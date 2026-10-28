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
}
