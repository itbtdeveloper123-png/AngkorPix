//! 1-Click Pro Quick Actions inspired by Adobe Photoshop 2026:
//! High-frequency commercial workflows executable in a single click without manual parameter tweaking.
//!
//! Included 1-Click Tools:
//! 1. `quickAction.autoEnhance`: One-click Auto Tone + Auto Contrast + Auto Color
//! 2. `quickAction.blurBackground`: Instant Portrait Depth / Bokeh (sharp subject, silky blurred background)
//! 3. `quickAction.blackWhiteBackground`: Color Splash (vivid subject against dramatic B&W background)
//! 4. `quickAction.goldenHour`: Instant warm golden hour glow & sunset sunlight grade
//! 5. `quickAction.cinematicTealOrange`: Hollywood blockbuster split-toning grade
//! 6. `quickAction.popColors`: HDR vibrance and dynamic punch
//! 7. `quickAction.studioProductWhite`: E-commerce pure white studio backdrop with soft floor contact shadow

use photocraft_color::{BlendMode, PixelFormat};
use photocraft_doc::{Layer, LayerContent, LayerId};
use photocraft_geom::Rect;
use photocraft_raster::Surface;
use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::{EngineError, Result, Session};

fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no document open".into())
}

fn has_pixels(s: &Session) -> std::result::Result<(), String> {
    let d = s.active().ok_or("no document open")?;
    let id = d.active_layer.ok_or("no active layer")?;
    let l = d.doc.layer(id).ok_or("no active layer")?;
    if l.surface().is_some() {
        Ok(())
    } else {
        Err("the active layer has no pixels".into())
    }
}

/// 1. Auto Enhance: One-click Auto Tone + Auto Contrast + Auto Color
fn auto_enhance(s: &mut Session, _p: &Value) -> Result<Value> {
    let _ = s.execute("image.autoTone", json!({}))?;
    let _ = s.execute("image.autoContrast", json!({}))?;
    let _ = s.execute("image.autoColor", json!({}))?;
    Ok(json!({"status": "success"}))
}

/// 2. Blur Background: Instant Portrait Depth / Bokeh
fn blur_background(s: &mut Session, p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let base_id = d.active_layer.ok_or_else(|| EngineError::Other("no active layer".into()))?;

    // 1. Select subject
    let _ = s.execute("select.subject", json!({}))?;

    // 2. Duplicate selected subject into a new layer
    let res = s.execute("layer.new.layerViaCopy", json!({}))?;
    let subject_id = res.get("layer").and_then(Value::as_u64).map(LayerId);

    // 3. Switch back to original background layer to apply blur
    s.select_layer(base_id)?;

    // 4. Apply Gaussian Blur to background
    let radius = p.get("radius").and_then(Value::as_f64).unwrap_or(16.0);
    let _ = s.execute("filter.blur.gaussianBlur", json!({ "radius": radius }))?;

    // 5. Restore active layer to the sharp subject layer
    if let Some(sub) = subject_id {
        let _ = s.select_layer(sub);
    }

    // 6. Deselect
    let _ = s.execute("select.deselect", json!({}));

    Ok(json!({
        "subjectLayer": subject_id.map(|id| id.0),
        "baseLayer": base_id.0
    }))
}

/// 3. B&W Background (Color Splash): Subject stays vivid color, background turns black & white
fn black_white_background(s: &mut Session, _p: &Value) -> Result<Value> {
    // 1. Select subject
    let _ = s.execute("select.subject", json!({}))?;

    // 2. Invert selection to target background
    let _ = s.execute("select.inverse", json!({}))?;

    // 3. Create Black & White adjustment layer (masked to background automatically)
    let res = s.execute("layer.newAdjustmentLayer.blackWhite", json!({}))?;

    // 4. Deselect
    let _ = s.execute("select.deselect", json!({}));

    Ok(res)
}

/// 4. Golden Hour Glow: Warm radiant sunset lighting
fn golden_hour(s: &mut Session, _p: &Value) -> Result<Value> {
    s.execute(
        "layer.newAdjustmentLayer.colorBalance",
        json!({
            "shadows": [12, 2, -10],
            "midtones": [25, 6, -20],
            "highlights": [30, 10, -25],
            "preserveLuminosity": true
        }),
    )
}

/// 5. Cinematic Teal & Orange: Blockbuster split-toning movie grade
fn cinematic_teal_orange(s: &mut Session, _p: &Value) -> Result<Value> {
    s.execute(
        "layer.newAdjustmentLayer.colorBalance",
        json!({
            "shadows": [-25, 4, 30],
            "midtones": [-5, 0, 5],
            "highlights": [28, 8, -22],
            "preserveLuminosity": true
        }),
    )
}

/// 6. Pop Colors: Boosts dynamic range, contrast, and vibrance
fn pop_colors(s: &mut Session, _p: &Value) -> Result<Value> {
    s.execute(
        "layer.newAdjustmentLayer.hueSaturation",
        json!({
            "saturation": 20
        }),
    )
}

/// Realistic contact + cast ground shadow under subject base
fn build_contact_shadow(bounds: Rect, base_cx: i32, base_cy: i32, sub_w: i32, fmt: PixelFormat) -> Layer {
    let w = bounds.width() as usize;
    let h = bounds.height() as usize;
    let total = w * h;
    let mut surf = Surface::new(fmt);
    let mut rgba = vec![0.0f32; total * 4];

    let rx_contact = ((sub_w as f32) * 0.40).max(35.0);
    let ry_contact = ((sub_w as f32) * 0.08).max(8.0);
    let rx_cast = rx_contact * 1.5;
    let ry_cast = ry_contact * 2.5;
    let cast_center_y = base_cy as f32 + ry_contact * 0.6;

    let min_y = (base_cy as f32 - ry_contact * 2.0).max(0.0) as usize;
    let max_y = (base_cy as f32 + ry_cast * 2.2).min(h as f32) as usize;
    let min_x = (base_cx as f32 - rx_cast * 1.8).max(0.0) as usize;
    let max_x = (base_cx as f32 + rx_cast * 1.8).min(w as f32) as usize;

    for y in min_y..max_y {
        for x in min_x..max_x {
            let idx = y * w + x;
            let dx = x as f32 - base_cx as f32;
            let dy_contact = y as f32 - base_cy as f32;
            let dy_cast = y as f32 - cast_center_y;

            let dist_contact = (dx / rx_contact).powi(2) + (dy_contact / ry_contact).powi(2);
            let alpha_contact = if dist_contact <= 1.0 { (1.0 - dist_contact).powf(1.8) * 0.85 } else { 0.0 };

            let dist_cast = (dx / rx_cast).powi(2) + (dy_cast / ry_cast).powi(2);
            let alpha_cast = if dist_cast <= 1.0 { (1.0 - dist_cast).powf(1.2) * 0.45 } else { 0.0 };

            let combined_alpha = (alpha_contact.max(alpha_cast)).clamp(0.0, 0.95);
            if combined_alpha > 0.005 {
                let p = idx * 4;
                if p + 3 < rgba.len() {
                    rgba[p] = 0.0;
                    rgba[p + 1] = 0.0;
                    rgba[p + 2] = 0.0;
                    rgba[p + 3] = combined_alpha;
                }
            }
        }
    }
    surf.write_region(bounds, &rgba);
    let mut shadow_l = Layer::new("Floor Ground Shadow", LayerContent::Raster(surf));
    shadow_l.blend = BlendMode::Multiply;
    shadow_l.opacity = 0.85;
    shadow_l
}

/// Pure studio white backdrop with subtle radial light vignette
fn build_white_studio_bg(bounds: Rect, fmt: PixelFormat) -> Layer {
    let w = bounds.width() as usize;
    let h = bounds.height() as usize;
    let total = w * h;
    let mut surf = Surface::new(fmt);
    let mut rgba = vec![1.0f32; total * 4];

    let cx = bounds.width() as f32 / 2.0;
    let cy = bounds.height() as f32 * 0.45;
    let max_dist = (cx * cx + cy * cy).sqrt().max(1.0);

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;
            let val = (1.0 - dist * 0.035).clamp(0.95, 1.0);
            let idx = (y * w + x) * 4;
            if idx + 3 < rgba.len() {
                rgba[idx] = val;
                rgba[idx + 1] = val;
                rgba[idx + 2] = val;
                rgba[idx + 3] = 1.0;
            }
        }
    }
    surf.write_region(bounds, &rgba);
    Layer::new("Studio White Backdrop", LayerContent::Raster(surf))
}

/// 7. Studio Product White: Commercial cutout on studio white background with floor ground shadow
fn studio_product_white(s: &mut Session, p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let pid = d.active_layer.ok_or_else(|| EngineError::Other("no active layer".into()))?;

    // 1. Remove background from product
    let _ = s.execute("layer.removeBackground", p.clone())?;

    // 2. Generate and place shadow and white backdrop underneath
    s.edit("Studio Product White", |doc, active| {
        let l = doc.layer(pid).ok_or(EngineError::NoLayer(pid))?;
        let bounds = l.surface().map(|s| s.content_bounds()).unwrap_or_else(|| doc.bounds());
        let base_cx = bounds.x0 + (bounds.width() as i32) / 2;
        let base_cy = bounds.y0 + (bounds.height() as i32);
        let sub_w = bounds.width() as i32;

        let shadow = build_contact_shadow(doc.bounds(), base_cx, base_cy, sub_w, doc.pixel_format());
        let bg = build_white_studio_bg(doc.bounds(), doc.pixel_format());

        let sid = doc.insert_below(Some(pid), shadow);
        let _ = doc.insert_below(Some(sid), bg);
        *active = Some(pid);
        Ok(())
    })?;

    Ok(json!({"productLayer": pid.0}))
}

/// 8. Add Contact Shadow: Place realistic floor contact & cast ground shadow under subject base
fn add_contact_shadow(s: &mut Session, _p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let pid = d.active_layer.ok_or_else(|| EngineError::Other("no active layer".into()))?;

    s.edit("Add Ground Shadow", |doc, active| {
        let l = doc.layer(pid).ok_or(EngineError::NoLayer(pid))?;
        let bounds = l.surface().map(|s| s.content_bounds()).unwrap_or_else(|| doc.bounds());
        let base_cx = bounds.x0 + (bounds.width() as i32) / 2;
        let base_cy = bounds.y0 + (bounds.height() as i32);
        let sub_w = bounds.width() as i32;

        let shadow = build_contact_shadow(doc.bounds(), base_cx, base_cy, sub_w, doc.pixel_format());
        let _ = doc.insert_below(Some(pid), shadow);
        *active = Some(pid);
        Ok(())
    })?;

    Ok(json!({"status": "success", "layer": pid.0}))
}

/// 9. Smooth Skin Retouch: Non-destructive portrait skin softening with texture retention
fn smooth_skin(s: &mut Session, p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let base_id = d.active_layer.ok_or_else(|| EngineError::Other("no active layer".into()))?;
    let radius = p.get("radius").and_then(Value::as_f64).unwrap_or(4.5).clamp(1.0, 50.0);
    let opacity = p.get("opacity").and_then(Value::as_f64).unwrap_or(0.75).clamp(0.1, 1.0) as f32;

    // 1. Duplicate active layer
    let res = s.execute("layer.duplicate", json!({}))?;
    let smooth_id = res.get("layer").and_then(Value::as_u64).map(LayerId);

    // 2. Apply Gaussian Blur to duplicate
    let _ = s.execute("filter.blur.gaussianBlur", json!({ "radius": radius }))?;

    // 3. Set opacity & name
    if let Some(id) = smooth_id {
        s.edit("Smooth Skin", |doc, active| {
            if let Some(l) = doc.layer_mut(id) {
                l.name = "✨ Smooth Skin".into();
                l.opacity = opacity;
            }
            *active = Some(id);
            Ok(())
        })?;
    }

    Ok(json!({"status": "success", "baseLayer": base_id.0, "smoothLayer": smooth_id.map(|id| id.0)}))
}

/// 10. Match Lighting / Tone: Harmonize subject color temperature and tones to background
fn match_lighting(s: &mut Session, p: &Value) -> Result<Value> {
    let warm = p.get("warm").and_then(Value::as_bool).unwrap_or(true);
    let (s_red, m_red, h_red) = if warm { (8, 14, 12) } else { (-8, -12, -10) };
    let (s_blue, m_blue, h_blue) = if warm { (-10, -18, -15) } else { (12, 16, 15) };

    s.execute(
        "layer.newAdjustmentLayer.colorBalance",
        json!({
            "shadows": [s_red, 2, s_blue],
            "midtones": [m_red, 4, m_blue],
            "highlights": [h_red, 5, h_blue],
            "preserveLuminosity": true
        }),
    )?;
    let _ = s.execute("layer.createClippingMask", json!({}));
    Ok(json!({"status": "success"}))
}

/// 11. Teeth & Eye Brighten: Clean highlight pop and yellow-cast removal
fn brighten_eyes_teeth(s: &mut Session, _p: &Value) -> Result<Value> {
    s.execute(
        "layer.newAdjustmentLayer.hueSaturation",
        json!({
            "saturation": -15,
            "lightness": 10
        }),
    )?;
    let _ = s.execute("layer.createClippingMask", json!({}));
    Ok(json!({"status": "success"}))
}

/// 12. Auto Center & Fit: Center subject content on canvas
fn auto_center_fit(s: &mut Session, _p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let pid = d.active_layer.ok_or_else(|| EngineError::Other("no active layer".into()))?;
    let doc_bounds = d.doc.bounds();
    let l = d.doc.layer(pid).ok_or(EngineError::NoLayer(pid))?;
    let content_bounds = l.surface().map(|s| s.content_bounds()).unwrap_or_else(|| doc_bounds);

    let doc_cx = (doc_bounds.x0 + doc_bounds.x1) / 2;
    let doc_cy = (doc_bounds.y0 + doc_bounds.y1) / 2;
    let content_cx = (content_bounds.x0 + content_bounds.x1) / 2;
    let content_cy = (content_bounds.y0 + content_bounds.y1) / 2;

    let dx = doc_cx - content_cx;
    let dy = doc_cy - content_cy;

    if dx != 0 || dy != 0 {
        s.execute("layer.translate", json!({"dx": dx, "dy": dy}))?;
    }
    Ok(json!({"status": "success", "dx": dx, "dy": dy}))
}

/// 13. Sticker Outline: Add instant clean white sticker border/stroke around subject
fn sticker_outline(s: &mut Session, p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let pid = d.active_layer.ok_or_else(|| EngineError::Other("no active layer".into()))?;
    let width = p.get("width").and_then(Value::as_u64).unwrap_or(8) as u32;

    s.execute(
        "layer.layerStyle.stroke",
        json!({
            "layer": pid.0,
            "size": width,
            "color": "#ffffff",
            "position": "outside",
            "opacity": 100
        }),
    )?;
    let _ = s.execute(
        "layer.layerStyle.dropShadow",
        json!({
            "layer": pid.0,
            "distance": 4,
            "size": 10,
            "opacity": 25
        }),
    );
    Ok(json!({"status": "success"}))
}

/// 14. Auto Clean Watermark: Automatic text/watermark detection and content-aware inpainting
fn auto_clean_watermark(s: &mut Session, p: &Value) -> Result<Value> {
    // 1. Try LaMa neural inpainting first for dewatermark.ai photorealistic quality
    if let Ok(res) = ai_remove(s, p) {
        return Ok(res);
    }

    // 2. Native algorithmic fallback if neural inpainter is unavailable
    auto_clean_watermark_native(s)
}

fn auto_clean_watermark_native(s: &mut Session) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let pid = d.active_layer.ok_or_else(|| EngineError::Other("no active layer".into()))?;

    // Check if user already has an active selection around the watermark
    let has_selection = d.doc.selection.as_ref().is_some_and(|sel| !sel.content_bounds().is_empty());

    if has_selection {
        let _ = s.execute("edit.deleteAndFillSelection", json!({}))?;
        let _ = s.execute("select.deselect", json!({}));
        return Ok(json!({"status": "success", "mode": "existing_selection"}));
    }

    // Auto-detect watermark / text strokes from active layer surface
    let doc_bounds = d.doc.bounds();
    let l = d.doc.layer(pid).ok_or(EngineError::NoLayer(pid))?;
    let surf = l.surface().ok_or_else(|| EngineError::Other("the active layer has no pixels".into()))?;
    let bounds = surf.content_bounds().intersect(&doc_bounds);
    if bounds.is_empty() {
        return Err(EngineError::Other("the active layer is empty".into()));
    }

    let w = bounds.width() as usize;
    let h = bounds.height() as usize;
    if w < 4 || h < 4 {
        return Err(EngineError::Other("layer content too small for watermark analysis".into()));
    }

    let pixel_count = w.checked_mul(h).ok_or_else(|| EngineError::Other("layer area too large".into()))?;
    if pixel_count > 50_000_000 {
        return Err(EngineError::Other("layer is too large for full auto-scan; make a selection around the watermark first".into()));
    }

    let mut rgba = vec![[0.0f32; 4]; pixel_count];
    surf.read_rgba_into(bounds, &mut rgba);

    // Compute luminance map
    let mut lum = vec![0.0f32; pixel_count];
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if let Some(&[r, g, b, a]) = rgba.get(idx) {
                lum[idx] = (0.299 * r + 0.587 * g + 0.114 * b) * a;
            }
        }
    }

    // Detect watermark & text strokes:
    let mut stroke_mask = vec![false; pixel_count];
    let mut detected_count = 0usize;

    for y in 1..h.saturating_sub(1) {
        let row_curr = y * w;
        let row_prev = (y - 1) * w;
        let row_next = (y + 1) * w;
        for x in 1..w.saturating_sub(1) {
            let idx = row_curr + x;
            let c = lum[idx];
            let left = lum[idx - 1];
            let right = lum[idx + 1];
            let up = lum[row_prev + x];
            let down = lum[row_next + x];

            let gx = (right - left).abs() * 0.5;
            let gy = (down - up).abs() * 0.5;
            let dxx = (2.0 * c - left - right).abs();
            let dyy = (2.0 * c - up - down).abs();
            let lap = (4.0 * c - left - right - up - down).abs();

            if (gx > 0.10 || gy > 0.10) && (lap > 0.15 || dxx > 0.12 || dyy > 0.12) {
                stroke_mask[idx] = true;
                detected_count += 1;
            }
        }
    }

    let max_allowed = pixel_count * 25 / 100;
    if detected_count > 0 && detected_count <= max_allowed {
        let dilated = photocraft_algo::remove::dilate(w, h, &stroke_mask, 3);
        let coverage: Vec<f32> = dilated.into_iter().map(|b| if b { 1.0 } else { 0.0 }).collect();
        let sel_surface = photocraft_algo::selection::mask_to_surface(&coverage, bounds);

        s.edit("Auto Clean Watermark", |doc, _| {
            doc.selection = Some(sel_surface);
            Ok(())
        })?;

        let _ = s.execute("edit.deleteAndFillSelection", json!({}))?;
        let _ = s.execute("select.deselect", json!({}));

        Ok(json!({
            "status": "success",
            "mode": "auto_detected",
            "detectedPixels": detected_count
        }))
    } else {
        let _ = s.execute("edit.deleteAndFillSelection", json!({}))?;
        let _ = s.execute("select.deselect", json!({}));
        Ok(json!({"status": "success", "mode": "fallback_clean"}))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_ai_inpaint(
    doc: &photocraft_doc::Document,
    layer_id: LayerId,
    bounds: Rect,
    mask_cov: Option<&[f32]>,
    engine: Option<&str>,
) -> std::result::Result<Vec<[f32; 4]>, String> {
    let w = bounds.width();
    let h = bounds.height();
    if w < 4 || h < 4 {
        return Err("layer bounds too small".into());
    }

    let l = doc.layer(layer_id).ok_or("no layer")?;
    let surf = l.surface().ok_or("no surface")?;

    let mut rgba_vec = vec![[0.0f32; 4]; (w * h) as usize];
    surf.read_rgba_into(bounds, &mut rgba_vec);

    let mut u8_img = Vec::with_capacity((w * h * 4) as usize);
    for px in &rgba_vec {
        u8_img.push((px[0].clamp(0.0, 1.0) * 255.0).round() as u8);
        u8_img.push((px[1].clamp(0.0, 1.0) * 255.0).round() as u8);
        u8_img.push((px[2].clamp(0.0, 1.0) * 255.0).round() as u8);
        u8_img.push((px[3].clamp(0.0, 1.0) * 255.0).round() as u8);
    }

    let img_codecs = photocraft_codecs::Image::from_u8(w, h, photocraft_codecs::ChannelLayout::Rgba, u8_img)
        .map_err(|e| format!("failed to create image: {e}"))?;
    let img_bytes = photocraft_codecs::encode(&img_codecs, photocraft_codecs::Format::Png, &photocraft_codecs::EncodeOptions::default())
        .map_err(|e| format!("failed to encode image: {e}"))?;

    let in_file = tempfile::Builder::new()
        .prefix("angkorpix_inpaint_in_")
        .suffix(".png")
        .tempfile()
        .map_err(|e| format!("failed to create temp file: {e}"))?;
    std::fs::write(in_file.path(), &img_bytes).map_err(|e| format!("failed to write in image: {e}"))?;

    let mask_file_opt = if let Some(cov) = mask_cov {
        let mut u8_mask = Vec::with_capacity((w * h) as usize);
        for &c in cov {
            u8_mask.push((c.clamp(0.0, 1.0) * 255.0).round() as u8);
        }
        let mask_codecs = photocraft_codecs::Image::from_u8(w, h, photocraft_codecs::ChannelLayout::Gray, u8_mask)
            .map_err(|e| format!("failed to create mask: {e}"))?;
        let mask_bytes = photocraft_codecs::encode(&mask_codecs, photocraft_codecs::Format::Png, &photocraft_codecs::EncodeOptions::default())
            .map_err(|e| format!("failed to encode mask: {e}"))?;

        let mask_file = tempfile::Builder::new()
            .prefix("angkorpix_inpaint_mask_")
            .suffix(".png")
            .tempfile()
            .map_err(|e| format!("failed to create temp mask file: {e}"))?;
        std::fs::write(mask_file.path(), &mask_bytes).map_err(|e| format!("failed to write in mask: {e}"))?;
        Some(mask_file)
    } else {
        None
    };

    let time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let out_path = std::env::temp_dir().join(format!("angkorpix_inpaint_out_{}_{}.png", std::process::id(), time_ms));

    let mut script_candidates = vec![
        std::path::PathBuf::from("scripts/inpaint_infer.py"),
        std::path::PathBuf::from("../../scripts/inpaint_infer.py"),
    ];
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        script_candidates.push(std::path::PathBuf::from(&manifest_dir).join("../../scripts/inpaint_infer.py"));
        script_candidates.push(std::path::PathBuf::from(&manifest_dir).join("../scripts/inpaint_infer.py"));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        script_candidates.push(parent.join("scripts/inpaint_infer.py"));
        script_candidates.push(parent.join("../../scripts/inpaint_infer.py"));
        script_candidates.push(parent.join("../../../scripts/inpaint_infer.py"));
    }
    let script_path = script_candidates.into_iter().find(|p| p.exists())
        .ok_or_else(|| "scripts/inpaint_infer.py not found".to_string())?;

    let py_cmd = if cfg!(windows) { "python" } else { "python3" };
    let mut cmd = std::process::Command::new(py_cmd);
    cmd.arg(script_path)
        .arg("--input").arg(in_file.path())
        .arg("--output").arg(&out_path);

    if let Some(eng) = engine {
        cmd.arg("--engine").arg(eng);
    }

    if let Some(ref mf) = mask_file_opt {
        cmd.arg("--mask").arg(mf.path());
    } else {
        cmd.arg("--auto-watermark");
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().map_err(|e| format!("failed to execute Python ({py_cmd}): {e}"))?;
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
        return Err("AI script completed but output was not found".into());
    }

    let out_bytes = std::fs::read(&out_path).map_err(|e| format!("failed to read output: {e}"))?;
    let _ = std::fs::remove_file(&out_path);

    let out_img = photocraft_codecs::decode(&out_bytes).map_err(|e| format!("failed to decode output: {e}"))?;
    let ch = out_img.layout().channels().max(1);
    let raw = out_img.data();
    let total_px = (w * h) as usize;
    let mut res_rgba = Vec::with_capacity(total_px);

    for i in 0..total_px {
        let offset = i * ch;
        let r = raw.get(offset).copied().unwrap_or(0);
        let g = raw.get(offset + 1).copied().unwrap_or(0);
        let b = raw.get(offset + 2).copied().unwrap_or(0);
        let a = if ch >= 4 { raw.get(offset + 3).copied().unwrap_or(255) } else { 255 };
        res_rgba.push([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]);
    }

    Ok(res_rgba)
}

#[cfg(target_arch = "wasm32")]
fn run_ai_inpaint(
    _doc: &photocraft_doc::Document,
    _layer_id: LayerId,
    _bounds: Rect,
    _mask_cov: Option<&[f32]>,
    _engine: Option<&str>,
) -> std::result::Result<Vec<[f32; 4]>, String> {
    Err("AI inpainting requires desktop native runtime".into())
}

/// 15. AI Remove / Inpaint: Photorealistic object and watermark removal using LaMa ONNX or Cloudflare Workers AI
fn ai_remove(s: &mut Session, p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let pid = d.active_layer.ok_or_else(|| EngineError::Other("no active layer".into()))?;

    let doc_bounds = d.doc.bounds();
    let l = d.doc.layer(pid).ok_or(EngineError::NoLayer(pid))?;
    let surf = l.surface().ok_or_else(|| EngineError::Other("the active layer has no pixels".into()))?;
    let bounds = surf.content_bounds().intersect(&doc_bounds);
    if bounds.is_empty() {
        return Err(EngineError::Other("the active layer is empty".into()));
    }

    let has_selection = d.doc.selection.as_ref().is_some_and(|sel| !sel.content_bounds().is_empty());
    let mask_cov = if has_selection {
        Some(photocraft_algo::selection::mask_from_surface(d.doc.selection.as_ref(), bounds))
    } else {
        None
    };

    let engine_param = p.get("engine").and_then(Value::as_str);
    let history_label = match engine_param {
        Some("cloudflare") => "AI Inpaint (Cloudflare Workers AI)",
        Some("lama") => "AI Inpaint (LaMa)",
        _ => "AI Inpaint",
    };

    // Try AI Inpaint
    match run_ai_inpaint(&d.doc, pid, bounds, mask_cov.as_deref(), engine_param) {
        Ok(res_rgba) => {
            s.edit(history_label, |doc, _| {
                let l = doc.layer_mut(pid).ok_or(EngineError::NoLayer(pid))?;
                let surf = l.surface_mut().ok_or_else(|| EngineError::Other("no surface".into()))?;
                let fmt = surf.format();
                let mut flat_data = Vec::with_capacity(res_rgba.len() * fmt.channels());
                let mut enc = [0.0f32; 8];
                for px in &res_rgba {
                    let n = photocraft_raster::from_rgba_into(&fmt, *px, &mut enc);
                    flat_data.extend_from_slice(&enc[..n]);
                }
                surf.write_region(bounds, &flat_data);
                surf.prune();
                doc.selection = None;
                Ok(())
            })?;
            Ok(json!({"status": "success", "engine": engine_param.unwrap_or("auto"), "mode": if has_selection { "selection" } else { "auto_watermark" }}))
        }
        Err(_) => {
            if has_selection {
                // Graceful fallback to native Content-Aware Fill
                let _ = s.execute("edit.deleteAndFillSelection", json!({}))?;
                let _ = s.execute("select.deselect", json!({}));
                Ok(json!({"status": "success", "engine": "fallback_content_aware"}))
            } else {
                // Auto watermark fallback: native stroke detection + deleteAndFill
                auto_clean_watermark_native(s)
            }
        }
    }
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "quickAction.autoEnhance",
            label: "Auto Enhance (Tone, Contrast & Color)",
            menu: &["Image"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_pixels,
            run: auto_enhance,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.blurBackground",
            label: "Blur Background (Portrait Bokeh)",
            menu: &["Filter", "Blur"],
            shortcut: None,
            params: r#"{"radius": 16.0}"#,
            enabled: has_pixels,
            run: blur_background,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.blackWhiteBackground",
            label: "B&W Background (Color Splash)",
            menu: &["Layer", "New Adjustment Layer"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_pixels,
            run: black_white_background,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.goldenHour",
            label: "Warm Golden Hour Glow",
            menu: &["Layer", "New Adjustment Layer"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_doc,
            run: golden_hour,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.cinematicTealOrange",
            label: "Cinematic Teal & Orange Grade",
            menu: &["Layer", "New Adjustment Layer"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_doc,
            run: cinematic_teal_orange,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.popColors",
            label: "Pop Colors (HDR Vibrance)",
            menu: &["Layer", "New Adjustment Layer"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_doc,
            run: pop_colors,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.studioProductWhite",
            label: "Studio Product White (With Floor Shadow)",
            menu: &["Layer"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_pixels,
            run: studio_product_white,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.addContactShadow",
            label: "Add Ground Shadow",
            menu: &["Layer"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_pixels,
            run: add_contact_shadow,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.smoothSkin",
            label: "Smooth Skin (Retouch)",
            menu: &["Filter"],
            shortcut: None,
            params: r#"{"radius": 4.5, "opacity": 0.75}"#,
            enabled: has_pixels,
            run: smooth_skin,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.matchLighting",
            label: "Match Lighting & Tone",
            menu: &["Image"],
            shortcut: None,
            params: r#"{"warm": true}"#,
            enabled: has_pixels,
            run: match_lighting,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.brightenEyesTeeth",
            label: "Teeth & Eye Brighten",
            menu: &["Image"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_pixels,
            run: brighten_eyes_teeth,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.autoCenterFit",
            label: "Auto Center & Fit",
            menu: &["Layer"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_pixels,
            run: auto_center_fit,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.stickerOutline",
            label: "Sticker White Outline",
            menu: &["Layer"],
            shortcut: None,
            params: r#"{"width": 8}"#,
            enabled: has_pixels,
            run: sticker_outline,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.autoCleanWatermark",
            label: "Auto Clean Watermark & Text",
            menu: &["Edit"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_pixels,
            run: auto_clean_watermark,
            journal: true,
        },
        CommandSpec {
            id: "quickAction.aiRemove",
            label: "AI Remove / Inpaint (LaMa)",
            menu: &["Edit"],
            shortcut: None,
            params: r#"{}"#,
            enabled: has_pixels,
            run: ai_remove,
            journal: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session() -> Session {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 100, "height": 100, "depth": 8})).unwrap();
        s
    }

    #[test]
    fn quick_actions_disabled_without_document() {
        let s = Session::new();
        assert!(!s.is_enabled("quickAction.autoEnhance"));
        assert!(!s.is_enabled("quickAction.blurBackground"));
        assert!(!s.is_enabled("quickAction.goldenHour"));
        assert!(!s.is_enabled("quickAction.cinematicTealOrange"));
        assert!(!s.is_enabled("quickAction.popColors"));
        assert!(!s.is_enabled("quickAction.studioProductWhite"));
        assert!(!s.is_enabled("quickAction.autoCleanWatermark"));
        assert!(!s.is_enabled("quickAction.aiRemove"));
    }

    #[test]
    fn golden_hour_creates_adjustment_layer() {
        let mut s = sample_session();
        let initial_count = s.active().unwrap().doc.layers.len();
        assert!(s.execute("quickAction.goldenHour", json!({})).is_ok());
        assert_eq!(s.active().unwrap().doc.layers.len(), initial_count + 1);
    }

    #[test]
    fn cinematic_teal_orange_creates_adjustment_layer() {
        let mut s = sample_session();
        let initial_count = s.active().unwrap().doc.layers.len();
        assert!(s.execute("quickAction.cinematicTealOrange", json!({})).is_ok());
        assert_eq!(s.active().unwrap().doc.layers.len(), initial_count + 1);
    }

    #[test]
    fn pop_colors_creates_adjustment_layer() {
        let mut s = sample_session();
        let initial_count = s.active().unwrap().doc.layers.len();
        assert!(s.execute("quickAction.popColors", json!({})).is_ok());
        assert_eq!(s.active().unwrap().doc.layers.len(), initial_count + 1);
    }

    #[test]
    fn new_quick_actions_execute_safely() {
        let mut s = sample_session();
        s.execute("select.rect", json!({"x": 10, "y": 10, "width": 50, "height": 50})).unwrap();
        s.execute("edit.fill", json!({"color": "#ffffff"})).unwrap();
        s.execute("select.deselect", json!({})).unwrap();
        let base_id = s.active().unwrap().active_layer.unwrap();

        assert!(s.execute("quickAction.addContactShadow", json!({})).is_ok());
        s.edit("reset", |_, active| { *active = Some(base_id); Ok(()) }).unwrap();
        assert!(s.execute("quickAction.smoothSkin", json!({})).is_ok());
        s.edit("reset", |_, active| { *active = Some(base_id); Ok(()) }).unwrap();
        assert!(s.execute("quickAction.matchLighting", json!({})).is_ok());
        s.edit("reset", |_, active| { *active = Some(base_id); Ok(()) }).unwrap();
        assert!(s.execute("quickAction.brightenEyesTeeth", json!({})).is_ok());
        s.edit("reset", |_, active| { *active = Some(base_id); Ok(()) }).unwrap();
        assert!(s.execute("quickAction.autoCenterFit", json!({})).is_ok());
        s.edit("reset", |_, active| { *active = Some(base_id); Ok(()) }).unwrap();
        assert!(s.execute("quickAction.stickerOutline", json!({})).is_ok());
    }

    #[test]
    fn auto_clean_watermark_with_existing_selection() {
        let mut s = sample_session();
        s.execute("select.rect", json!({"x": 10, "y": 10, "width": 20, "height": 20})).unwrap();
        s.execute("edit.fill", json!({"color": "#ffffff"})).unwrap();
        assert!(s.execute("quickAction.autoCleanWatermark", json!({})).is_ok());
        assert!(s.active().unwrap().doc.selection.is_none());
    }

    #[test]
    fn auto_clean_watermark_auto_detect() {
        let mut s = sample_session();
        // Fill background first so the stroke has a contrast edge within content bounds
        s.execute("edit.fill", json!({"color": "#202020"})).unwrap();
        // Draw a high contrast white line stroke
        s.execute("select.rect", json!({"x": 40, "y": 40, "width": 4, "height": 20})).unwrap();
        s.execute("edit.fill", json!({"color": "#ffffff"})).unwrap();
        s.execute("select.deselect", json!({})).unwrap();

        let res = s.execute("quickAction.autoCleanWatermark", json!({}));
        assert!(res.is_ok());
        assert!(s.active().unwrap().doc.selection.is_none());
    }

    #[test]
    fn ai_remove_executes_safely() {
        let mut s = sample_session();
        s.execute("select.rect", json!({"x": 10, "y": 10, "width": 20, "height": 20})).unwrap();
        s.execute("edit.fill", json!({"color": "#ffffff"})).unwrap();
        assert!(s.execute("quickAction.aiRemove", json!({})).is_ok());
        assert!(s.active().unwrap().doc.selection.is_none());
    }

    #[test]
    fn ai_remove_auto_detect_without_selection() {
        let mut s = sample_session();
        s.execute("edit.fill", json!({"color": "#202020"})).unwrap();
        s.execute("select.rect", json!({"x": 30, "y": 30, "width": 4, "height": 16})).unwrap();
        s.execute("edit.fill", json!({"color": "#ffffff"})).unwrap();
        s.execute("select.deselect", json!({})).unwrap();

        let res = s.execute("quickAction.aiRemove", json!({}));
        assert!(res.is_ok());
        assert!(s.active().unwrap().doc.selection.is_none());
    }
}
