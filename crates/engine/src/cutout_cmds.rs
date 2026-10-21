//! Remove Background, Photoshop's Quick Action for a pixel layer (Properties › Quick Actions):
//! Select Subject with a light edge refinement, turned into a layer mask, in one history step.
//! Like Photoshop it is non-destructive (the pixels stay; the mask hides the background), turns
//! the Background layer into a normal layer first and has no menu item.

use photocraft_algo::matting::{self, RefineParams};
use photocraft_algo::segment::subject;
use photocraft_algo::selection::Region;
use photocraft_doc::{Document, Layer, LayerContent, LayerId, LayerMask};
use photocraft_geom::Rect;
use serde_json::{Value, json};

use crate::commands::{CommandSpec, layer_param};
use crate::smartselect_cmds::with_doc_sampler;
use crate::{EngineError, Result, Session};

const CMD: &str = "layer.removeBackground";

/// Edge refinement after Select Subject: a narrow smart radius and a little smoothing, so soft
/// edges and hair get partial coverage instead of a hard cut.
const REFINE: RefineParams = RefineParams { radius: 2.0, smart_radius: true, smooth: 10.0, feather: 0.5, contrast: 10.0, shift_edge: 0.0 };

/// Remove Background needs an unlocked pixel layer.
fn check(doc: &Document, l: &Layer) -> std::result::Result<(), String> {
    if !matches!(l.content, LayerContent::Raster(_)) {
        return Err(format!("the layer is {} {} layer, not a pixel layer", l.content.article(), l.content.kind_name()));
    }
    let locks = doc.effective_locks(l.id);
    if locks.pixels || locks.all {
        return Err(format!("Could not complete your request because the layer \"{}\" is locked", l.name));
    }
    Ok(())
}

fn enabled(s: &Session) -> std::result::Result<(), String> {
    let d = s.active().ok_or("no document open")?;
    check(&d.doc, crate::active_layer_of(s)?)
}

#[cfg(not(target_arch = "wasm32"))]
struct AiCutoutResult {
    region: Region,
    rgba_pixels: Option<Vec<f32>>,
}

#[cfg(not(target_arch = "wasm32"))]
fn run_ai_cutout(doc: &Document, layer_id: LayerId, bounds: Rect, sample_all: bool, mode: &str) -> std::result::Result<AiCutoutResult, String> {
    let w = bounds.width();
    let h = bounds.height();
    if w < 4 || h < 4 {
        return Err("layer bounds too small".into());
    }

    // Sample layer or composite pixels in bounds
    let rgba_vec: Vec<[f32; 4]> = with_doc_sampler(doc, Some(layer_id), sample_all, |smp, _| smp.rgba(bounds));
    let mut u8_data = Vec::with_capacity((w * h * 4) as usize);
    for px in &rgba_vec {
        u8_data.push((px[0].clamp(0.0, 1.0) * 255.0).round() as u8);
        u8_data.push((px[1].clamp(0.0, 1.0) * 255.0).round() as u8);
        u8_data.push((px[2].clamp(0.0, 1.0) * 255.0).round() as u8);
        u8_data.push((px[3].clamp(0.0, 1.0) * 255.0).round() as u8);
    }

    let img = photocraft_codecs::Image::from_u8(w, h, photocraft_codecs::ChannelLayout::Rgba, u8_data)
        .map_err(|e| format!("failed to create image: {e}"))?;
    let png_bytes = photocraft_codecs::encode(&img, photocraft_codecs::Format::Png, &photocraft_codecs::EncodeOptions::default())
        .map_err(|e| format!("failed to encode image: {e}"))?;

    let in_file = tempfile::Builder::new()
        .prefix("angkorpix_in_")
        .suffix(".png")
        .tempfile()
        .map_err(|e| format!("failed to create temp file: {e}"))?;

    let time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let out_path = std::env::temp_dir().join(format!("angkorpix_mask_{}_{}.png", std::process::id(), time_ms));

    std::fs::write(in_file.path(), &png_bytes)
        .map_err(|e| format!("failed to write input image: {e}"))?;

    // Find script path relative to current exe or workspace
    let mut script_candidates = vec![
        std::path::PathBuf::from("scripts/rmbg_infer.py"),
    ];
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        script_candidates.push(parent.join("scripts/rmbg_infer.py"));
        script_candidates.push(parent.join("../../scripts/rmbg_infer.py"));
    }
    let script_path = script_candidates.into_iter().find(|p| p.exists())
        .ok_or_else(|| "scripts/rmbg_infer.py not found".to_string())?;

    let py_cmd = if cfg!(windows) { "python" } else { "python3" };
    let mut cmd = std::process::Command::new(py_cmd);
    cmd.arg(script_path)
        .arg("--input").arg(in_file.path())
        .arg("--output").arg(&out_path)
        .arg("--mode").arg(mode);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().map_err(|e| format!("failed to execute Python ({py_cmd}): {e}. Please ensure Python with onnxruntime is installed."))?;
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        let err_trimmed = err_msg.trim();
        return Err(if err_trimmed.is_empty() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            err_trimmed.to_string()
        });
    }

    if !out_path.exists() {
        return Err("AI script completed but output mask was not found".into());
    }

    let mask_bytes = std::fs::read(&out_path).map_err(|e| format!("failed to read mask: {e}"))?;
    let _ = std::fs::remove_file(&out_path);

    let mask_img = photocraft_codecs::decode(&mask_bytes).map_err(|e| format!("failed to decode mask: {e}"))?;
    let ch = mask_img.layout().channels().max(1);
    let raw = mask_img.data();
    let total_px = (w * h) as usize;
    let mut mask = Vec::with_capacity(total_px);
    let mut rgba_pixels = None;

    if ch >= 4 {
        let mut rgba = Vec::with_capacity(total_px * 4);
        for i in 0..total_px {
            let offset = i * ch;
            let r = raw.get(offset).copied().unwrap_or(0);
            let g = raw.get(offset + 1).copied().unwrap_or(0);
            let b = raw.get(offset + 2).copied().unwrap_or(0);
            let a = raw.get(offset + 3).copied().unwrap_or(0);
            rgba.push(r as f32 / 255.0);
            rgba.push(g as f32 / 255.0);
            rgba.push(b as f32 / 255.0);
            rgba.push(a as f32 / 255.0);
            mask.push(a);
        }
        rgba_pixels = Some(rgba);
    } else {
        for i in 0..total_px {
            if let Some(&b) = raw.get(i * ch) {
                mask.push(b);
            } else {
                mask.push(0);
            }
        }
    }

    Ok(AiCutoutResult {
        region: Region {
            bbox: bounds,
            mask,
        },
        rgba_pixels,
    })
}

fn run(s: &mut Session, p: &Value) -> Result<Value> {
    let id = layer_param(s, p)?;
    let doc = &s.active().ok_or(EngineError::NoDocument)?.doc;
    check(doc, doc.layer(id).ok_or(EngineError::NoLayer(id))?).map_err(EngineError::Other)?;
    let sample_all = p.get("sampleAllLayers").and_then(Value::as_bool).unwrap_or(false);
    let refine = p.get("refine").and_then(Value::as_bool).unwrap_or(true);
    let method = p.get("method")
        .and_then(Value::as_str)
        .and_then(crate::prefs::RemoveBackgroundMethod::parse)
        .unwrap_or(s.prefs().tools.remove_background_method);
    let method_param = p.get("method").and_then(Value::as_str);
    let is_glass = p.get("glass").and_then(Value::as_bool).unwrap_or(false)
        || method == crate::prefs::RemoveBackgroundMethod::Glass
        || method_param == Some("glass");
    let is_tree = method == crate::prefs::RemoveBackgroundMethod::Tree || method_param == Some("tree");
    let is_cloud = method == crate::prefs::RemoveBackgroundMethod::Cloud || method_param == Some("cloud");
    let is_plastic = method == crate::prefs::RemoveBackgroundMethod::Plastic || method_param == Some("plastic");
    let is_specialized = is_glass || is_tree || is_cloud || is_plastic;
    let is_ai = method == crate::prefs::RemoveBackgroundMethod::Ai || is_specialized;
    let explicit_ai = method_param == Some("ai") || is_specialized;

    let mode_str = if is_glass {
        "glass"
    } else if is_tree {
        "tree"
    } else if is_cloud {
        "cloud"
    } else if is_plastic {
        "plastic"
    } else {
        "auto"
    };

    let job_label = if is_glass {
        "Remove Background (Glass)"
    } else if is_tree {
        "Remove Background (Tree)"
    } else if is_cloud {
        "Remove Background (Cloud)"
    } else if is_plastic {
        "Remove Background (Plastic)"
    } else {
        "Remove Background"
    };

    crate::jobs::edit_job(
        s,
        job_label,
        move |doc, _, ctx| {
            let bounds = if !sample_all && let Some(surf) = doc.layer(id).and_then(Layer::surface) {
                let cb = surf.content_bounds().intersect(&doc.bounds());
                if cb.width() >= 4 && cb.height() >= 4 { cb } else { doc.bounds() }
            } else {
                doc.bounds()
            };

            let mut region_opt = None;
            let mut rgba_opt = None;
            if is_ai {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let progress_msg = match mode_str {
                        "glass" => "Running AI Glass model (rmbg_model.onnx)",
                        "tree" => "Running AI Tree Channel model (rmbg_model.onnx)",
                        "cloud" => "Running AI Cloud Blend model (rmbg_model.onnx)",
                        "plastic" => "Running AI Plastic Wrap model (rmbg_model.onnx)",
                        _ => "Running AI model (rmbg_model.onnx)",
                    };
                    ctx.progress(0.1, progress_msg);
                    match run_ai_cutout(doc, id, bounds, sample_all, mode_str) {
                        Ok(res) => {
                            region_opt = Some(res.region);
                            rgba_opt = res.rgba_pixels;
                        }
                        Err(e) => {
                            if explicit_ai {
                                return Err(EngineError::Other(e));
                            }
                            eprintln!("AI background removal failed, falling back to heuristic: {e}");
                        }
                    }
                }
            }

            let mut region = match region_opt {
                Some(r) => r,
                None => {
                    // Cancel reaches the min cuts inside Select Subject, so Esc doesn't wait for them.
                    let found = ctx
                        .stage(0.1, 0.6, "Finding subject", |ctl| {
                            with_doc_sampler(doc, Some(id), sample_all, |smp, _| subject::select_subject_with(smp, bounds, ctl))
                        })
                        .map_err(|_| EngineError::Cancelled)?;
                    found.ok_or_else(|| EngineError::Other("no subject found".into()))?
                }
            };

            if ctx.cancelled() {
                return Err(EngineError::Cancelled);
            }
            if refine && !is_ai {
                ctx.progress(0.6, "Refining edge");
                let refined = with_doc_sampler(doc, Some(id), sample_all, |smp, _| {
                    matting::refine_mask(smp, &matting::region_reader(&region), region.bbox, bounds, &REFINE)
                });
                region = refined.unwrap_or(region);
                if ctx.cancelled() {
                    return Err(EngineError::Cancelled);
                }
            }
            crate::extra_cmds::background_to_layer_for_mask(doc, id);
            if let Some(rgba) = &rgba_opt {
                if let Some(surf) = doc.layer_mut(id).and_then(Layer::surface_mut) {
                    if surf.channels() == 4 && rgba.len() == (bounds.width() as usize * bounds.height() as usize * 4) {
                        surf.write_region(bounds, rgba);
                    }
                }
            }
            doc.layer_mut(id).ok_or(EngineError::NoLayer(id))?.mask = Some(LayerMask { surface: matting::region_surface(&region), ..LayerMask::reveal_all() });
            doc.selection = None;
            let b = region.bbox;
            Ok([b.x0, b.y0, b.width() as i32, b.height() as i32])
        },
        move |bounds| json!({ "layer": id.0, "bounds": bounds }),
    )
}

/// Remove Background command spec (no menu path: a Properties Quick Action in Photoshop).
pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: CMD,
            label: "Remove Background",
            menu: &[],
            shortcut: None,
            params: r##"{"layer":id?,"sampleAllLayers":bool=false,"refine":bool=true,"method":"heuristic"|"ai"|"glass"|"tree"|"cloud"|"plastic","glass":bool=false} → {layer,bounds} (adds a layer mask from Select Subject or AI model; the Background becomes a normal layer)"##,
            enabled,
            run,
            journal: true,
        },
        CommandSpec {
            id: "layer.cutoutGlass",
            label: "Remove Background (Glass / Transparent)",
            menu: &[],
            shortcut: None,
            params: r##"{"layer":id?} → {layer,bounds} (auto AI cutout with glass transparency for transparent objects)"##,
            enabled,
            run: |s, p| {
                let mut params = p.clone();
                if let Some(obj) = params.as_object_mut() {
                    obj.insert("method".into(), json!("glass"));
                    obj.insert("glass".into(), json!(true));
                } else {
                    params = json!({"method": "glass", "glass": true});
                }
                run(s, &params)
            },
            journal: true,
        },
        CommandSpec {
            id: "layer.cutoutTree",
            label: "Remove Background (Tree / Foliage)",
            menu: &[],
            shortcut: None,
            params: r##"{"layer":id?} → {layer,bounds} (Photoshop blue-channel sky masking for fine foliage)"##,
            enabled,
            run: |s, p| {
                let mut params = p.clone();
                if let Some(obj) = params.as_object_mut() {
                    obj.insert("method".into(), json!("tree"));
                } else {
                    params = json!({"method": "tree"});
                }
                run(s, &params)
            },
            journal: true,
        },
        CommandSpec {
            id: "layer.cutoutCloud",
            label: "Remove Background (Cloud / Sky)",
            menu: &[],
            shortcut: None,
            params: r##"{"layer":id?} → {layer,bounds} (Photoshop soft blend-if matting for clouds and smoke)"##,
            enabled,
            run: |s, p| {
                let mut params = p.clone();
                if let Some(obj) = params.as_object_mut() {
                    obj.insert("method".into(), json!("cloud"));
                } else {
                    params = json!({"method": "cloud"});
                }
                run(s, &params)
            },
            journal: true,
        },
        CommandSpec {
            id: "layer.cutoutPlastic",
            label: "Remove Background (Plastic Bag)",
            menu: &[],
            shortcut: None,
            params: r##"{"layer":id?} → {layer,bounds} (Photoshop wrinkle highlights and multiply shadow matting)"##,
            enabled,
            run: |s, p| {
                let mut params = p.clone();
                if let Some(obj) = params.as_object_mut() {
                    obj.insert("method".into(), json!("plastic"));
                } else {
                    params = json!({"method": "plastic"});
                }
                run(s, &params)
            },
            journal: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use photocraft_algo::segment::Rng;
    use photocraft_doc::LayerId;
    use photocraft_geom::Rect;

    fn session_with(w: u32, h: u32, depth: u32, px: impl Fn(u32, u32) -> [f32; 3]) -> Session {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": w, "height": h, "depth": depth})).unwrap();
        s.edit("paint", |doc, _| {
            let bg = doc.layers[0].surface_mut().unwrap();
            let n = bg.channels();
            let mut data = Vec::with_capacity((w * h) as usize * n);
            for y in 0..h {
                for x in 0..w {
                    let c = px(x, y);
                    data.extend_from_slice(&c[..n.min(3)]);
                    if n == 4 {
                        data.push(1.0);
                    }
                }
            }
            bg.write_region(Rect::new(0, 0, w as i32, h as i32), &data);
            Ok(())
        })
        .unwrap();
        s
    }

    /// Textured disc (centre (100, 80), radius 45) on a noisy background.
    fn disc(depth: u32) -> (Session, impl Fn(i32, i32) -> bool) {
        let mut rng = Rng::new(2);
        let nz: Vec<f32> = (0..200 * 160).map(|_| rng.normal()).collect();
        let inside = |x: i32, y: i32| {
            let (dx, dy) = (x as f32 + 0.5 - 100.0, y as f32 + 0.5 - 80.0);
            dx * dx + dy * dy <= 45.0 * 45.0
        };
        let s = session_with(200, 160, depth, move |x, y| {
            let n = 0.05 * nz[(y * 200 + x) as usize];
            if inside(x as i32, y as i32) {
                let t = ((x + y) as f32 * 0.35).sin() * 0.5 + 0.5;
                [0.85 + 0.05 * t + n, 0.2 + 0.3 * t + n, 0.15 + n]
            } else {
                [0.25 + n, 0.45 + n, 0.7 + n]
            }
            .map(|v| v.clamp(0.0, 1.0))
        });
        (s, inside)
    }

    fn active_id(s: &Session) -> LayerId {
        s.active().unwrap().active_layer.unwrap()
    }

    fn layer(s: &Session, id: LayerId) -> &Layer {
        s.active().unwrap().doc.layer(id).unwrap()
    }

    fn mask_cov(s: &Session, x: i32, y: i32) -> f32 {
        layer(s, active_id(s)).mask.as_ref().map_or(0.0, |m| m.value(x, y))
    }

    fn mask_iou(s: &Session, w: i32, h: i32, truth: impl Fn(i32, i32) -> bool) -> f32 {
        let (mut i, mut u) = (0, 0);
        for y in 0..h {
            for x in 0..w {
                let (a, b) = (mask_cov(s, x, y) >= 0.5, truth(x, y));
                i += (a && b) as i32;
                u += (a || b) as i32;
            }
        }
        i as f32 / u.max(1) as f32
    }

    #[test]
    fn remove_background_masks_the_subject_at_every_depth_and_undoes() {
        for depth in [8, 16, 32] {
            let (mut s, inside) = disc(depth);
            assert!(s.is_enabled(CMD));
            let id = active_id(&s);
            let pixels = layer(&s, id).surface().unwrap().read_region(Rect::new(0, 0, 200, 160));
            let steps = s.active().unwrap().history.past_len();
            let r = s.execute(CMD, json!({})).unwrap();
            assert_eq!(r["layer"], id.0);
            assert_eq!(s.active().unwrap().history.past_len(), steps + 1, "depth {depth}: one history step");
            assert!(layer(&s, id).mask.is_some(), "depth {depth}: expected a layer mask");
            let v = mask_iou(&s, 200, 160, &inside);
            assert!(v >= 0.75, "depth {depth}: IoU {v}");
            assert!(mask_cov(&s, 0, 0) < 0.2, "depth {depth}: background corner still revealed");
            assert!(mask_cov(&s, 100, 80) >= 0.8, "depth {depth}: disc centre hidden");
            // Non-destructive: the pixels are untouched, only masked.
            assert_eq!(layer(&s, id).surface().unwrap().read_region(Rect::new(0, 0, 200, 160)), pixels, "depth {depth}");
            let corner = photocraft_compose::render(&s.active().unwrap().doc, Rect::new(0, 0, 1, 1)).px[0][3];
            assert!(corner < 0.2, "depth {depth}: composite corner alpha {corner}");
            assert!(s.active().unwrap().doc.selection.is_none());
            s.execute("edit.undo", json!({})).unwrap();
            assert!(layer(&s, id).mask.is_none(), "depth {depth}: mask remained after undo");
            s.execute("edit.redo", json!({})).unwrap();
            assert!(layer(&s, id).mask.is_some(), "depth {depth}: mask missing after redo");
        }
    }

    #[test]
    fn remove_background_turns_the_background_into_a_layer() {
        let (mut s, _) = disc(8);
        let id = active_id(&s);
        assert_eq!(layer(&s, id).name, "Background");
        s.execute(CMD, json!({})).unwrap();
        let l = layer(&s, id);
        assert_eq!(l.name, "Layer 0");
        assert!(!l.locks.transparency && !l.locks.position);
        assert!(l.mask.is_some());
    }

    #[test]
    fn remove_background_cancelled_while_finding_the_subject_changes_nothing() {
        let (mut s, _) = disc(8);
        let id = active_id(&s);
        let before = s.active().unwrap().revision;
        // The job's cancel reaches Select Subject, which stops at its first check.
        let ctx = crate::jobs::JobCtx::new();
        ctx.cancel();
        s.set_inline_job_ctx(Some(ctx.clone()));
        assert!(matches!(s.execute(CMD, json!({})), Err(EngineError::Cancelled)));
        assert_eq!(ctx.message(), "Finding subject");
        assert_eq!(s.active().unwrap().revision, before);
        assert!(layer(&s, id).mask.is_none());
        s.set_inline_job_ctx(None);
        s.execute(CMD, json!({})).unwrap();
        assert!(layer(&s, id).mask.is_some());
    }

    #[test]
    fn remove_background_without_a_subject_leaves_the_document() {
        let mut s = session_with(32, 32, 8, |_, _| [0.5, 0.5, 0.5]);
        let before = s.active().unwrap().revision;
        let id = active_id(&s);
        assert!(s.execute(CMD, json!({})).is_err());
        assert_eq!(s.active().unwrap().revision, before);
        assert!(layer(&s, id).mask.is_none());
    }

    #[test]
    fn remove_background_disabled_states_and_bad_params() {
        assert!(!Session::new().is_enabled(CMD));
        assert!(Session::new().execute(CMD, json!({})).is_err());

        let (mut s, _) = disc(8);
        s.execute("type.create", json!({"text": "Hi", "size": 24, "x": 10, "y": 20})).unwrap();
        assert!(!s.is_enabled(CMD));
        assert!(s.execute(CMD, json!({})).is_err());

        let (mut s, _) = disc(8);
        s.execute("shape.create", json!({"kind": "rect", "rect": [10, 10, 40, 20], "fill": "#ff0000"})).unwrap();
        assert!(!s.is_enabled(CMD));
        assert!(s.execute(CMD, json!({})).is_err());

        let (mut s, _) = disc(8);
        s.execute("layer.setProps", json!({"locks": {"pixels": true}})).unwrap();
        assert!(!s.is_enabled(CMD));
        assert!(s.execute(CMD, json!({})).is_err());

        let (mut s, _) = disc(8);
        let before = s.active().unwrap().revision;
        assert!(s.execute(CMD, json!({"layer": u64::MAX})).is_err());
        // A `layer` string names a layer; one that isn't there is an error, not the active layer.
        assert!(s.execute(CMD, json!({"layer": "x"})).is_err());
        assert_eq!(s.active().unwrap().revision, before);
        // Wrong types for the options fall back to the defaults (layer-only sampling, refined).
        assert!(s.execute(CMD, json!({"sampleAllLayers": 3, "refine": "no"})).is_ok());
        assert!(s.active().unwrap().revision > before);
    }

    #[test]
    fn remove_background_looks_at_the_layer_alone_by_default() {
        let (mut s, _) = disc(8);
        let bottom = active_id(&s);
        s.execute("layer.new.layer", json!({"name": "Empty"})).unwrap();
        let top = active_id(&s);
        let before = s.active().unwrap().revision;
        // The empty layer has no subject of its own, though the photo shows through it.
        assert!(s.execute(CMD, json!({})).is_err());
        assert_eq!(s.active().unwrap().revision, before);
        assert!(layer(&s, top).mask.is_none() && layer(&s, bottom).mask.is_none());
        // Sampling all layers finds the subject in the composite.
        s.execute(CMD, json!({"sampleAllLayers": true})).unwrap();
        assert!(layer(&s, top).mask.is_some() && layer(&s, bottom).mask.is_none());
    }

    #[test]
    fn remove_background_twice_replaces_the_mask() {
        let (mut s, _) = disc(8);
        s.execute(CMD, json!({})).unwrap();
        s.execute(CMD, json!({"refine": false})).unwrap();
        let masks = s.active().unwrap().doc.walk().into_iter().filter(|(_, _, l)| l.mask.is_some()).count();
        assert_eq!(masks, 1);
    }
}
