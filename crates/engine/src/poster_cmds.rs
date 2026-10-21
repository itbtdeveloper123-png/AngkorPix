//! Smart Poster Composer:
//! Automatically composes production-ready promotional posters from existing layers.
//! Integrates:
//! 1. Atmospheric backdrop tailored to style (Summer, Luxury Gold, Cyberpunk, Super Sale, Minimal)
//! 2. Ambient radial spotlight glow centered behind the hero subject
//! 3. Ground contact & directional cast shadow under the subject base
//! 4. Headline typography layer (scaled & positioned with high-impact styling)
//! 5. Sub-headline and promotional details text layer
//! 6. Promotional badge / CTA button layer
//! 7. Focus vignette frame
//!
//! Non-destructive: Every layer is editable and neatly grouped into a master poster group.

use photocraft_color::{BlendMode, Color, PixelFormat};
use photocraft_doc::text::{CharStyle, ParagraphRun, ParagraphStyle, TextAlign, TextRun, TextShape};
use photocraft_doc::{Document, Layer, LayerContent, TextLayer};
use photocraft_geom::{Affine, Rect};
use photocraft_raster::Surface;
use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::{EngineError, Result, Session};

pub const CMD: &str = "doc.composePoster";

fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no document open".into())
}

/// Create realistic contact shadow (tight & dark) + cast shadow (soft & wide)
fn build_shadow_layer(bounds: Rect, base_cx: i32, base_cy: i32, sub_w: i32, fmt: PixelFormat) -> Layer {
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

            // 1. Contact shadow (dark, thin line right where base touches floor)
            let dist_contact = (dx / rx_contact).powi(2) + (dy_contact / ry_contact).powi(2);
            let alpha_contact = if dist_contact <= 1.0 {
                (1.0 - dist_contact).powf(1.8) * 0.85
            } else {
                0.0
            };

            // 2. Cast shadow (softer, wider dispersion)
            let dist_cast = (dx / rx_cast).powi(2) + (dy_cast / ry_cast).powi(2);
            let alpha_cast = if dist_cast <= 1.0 {
                (1.0 - dist_cast).powf(1.2) * 0.45
            } else {
                0.0
            };

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
    let mut layer = Layer::new("🌑 Contact & Cast Shadow [Multiply]", LayerContent::Raster(surf));
    layer.blend = BlendMode::Multiply;
    layer.opacity = 0.85;
    layer
}

/// Create atmospheric backdrop surface matching chosen style
fn build_backdrop_layer(bounds: Rect, style: &str, fmt: PixelFormat, cx: i32, cy: i32) -> Layer {
    let w = bounds.width() as usize;
    let h = bounds.height() as usize;
    let total = w * h;
    let mut surf = Surface::new(fmt);
    let mut rgba = Vec::with_capacity(total * 4);

    let max_radius = ((w * w + h * h) as f32).sqrt() * 0.55;

    for y in 0..h {
        let vy = y as f32 / h.max(1) as f32;
        for x in 0..w {
            let dx = x as f32 - cx as f32;
            let dy = y as f32 - cy as f32;
            let dist_norm = ((dx * dx + dy * dy).sqrt() / max_radius).clamp(0.0, 1.0);

            let (r, g, b) = match style {
                "summer_fresh" | "tropical" => {
                    // Vibrant tropical cyan to deep marine blue
                    let top_r = 0.04; let top_g = 0.75; let top_b = 0.94;
                    let bot_r = 0.01; let bot_g = 0.20; let bot_b = 0.50;
                    let radial_boost = (1.0 - dist_norm).powf(2.0) * 0.25;
                    (
                        (top_r * (1.0 - vy) + bot_r * vy + radial_boost).clamp(0.0, 1.0),
                        (top_g * (1.0 - vy) + bot_g * vy + radial_boost).clamp(0.0, 1.0),
                        (top_b * (1.0 - vy) + bot_b * vy + radial_boost * 0.5).clamp(0.0, 1.0),
                    )
                }
                "luxury_gold" => {
                    // Obsidian black with rich warm bronze spotlight
                    let base_black = 0.04 + vy * 0.02;
                    let radial_gold = (1.0 - dist_norm).powf(1.6);
                    (
                        (base_black + radial_gold * 0.32).clamp(0.0, 1.0),
                        (base_black + radial_gold * 0.24).clamp(0.0, 1.0),
                        (base_black + radial_gold * 0.10).clamp(0.0, 1.0),
                    )
                }
                "cyberpunk" => {
                    // Deep dark violet with dual cyan/magenta neon falloff
                    let vx = x as f32 / w.max(1) as f32;
                    let neon_cyan = (1.0 - vx).powf(1.8) * 0.45;
                    let neon_mag = vx.powf(1.8) * 0.40;
                    (
                        (0.06 + neon_mag * 0.85).clamp(0.0, 1.0),
                        (0.02 + neon_cyan * 0.70).clamp(0.0, 1.0),
                        (0.14 + neon_cyan * 0.90 + neon_mag * 0.60).clamp(0.0, 1.0),
                    )
                }
                "super_sale" => {
                    // Energetic fiery red to dark burgundy gradient
                    let top_r = 0.92; let top_g = 0.16; let top_b = 0.22;
                    let bot_r = 0.35; let bot_g = 0.03; let bot_b = 0.07;
                    let radial_glow = (1.0 - dist_norm).powf(1.5) * 0.20;
                    (
                        (top_r * (1.0 - vy) + bot_r * vy + radial_glow).clamp(0.0, 1.0),
                        (top_g * (1.0 - vy) + bot_g * vy + radial_glow * 0.6).clamp(0.0, 1.0),
                        (top_b * (1.0 - vy) + bot_b * vy).clamp(0.0, 1.0),
                    )
                }
                _ => {
                    // Minimal clean studio grey spotlight
                    let radial = (1.0 - dist_norm).powf(1.4);
                    let val = 0.76 + radial * 0.22;
                    (val, val, (val * 1.01).clamp(0.0, 1.0))
                }
            };
            rgba.extend_from_slice(&[r, g, b, 1.0]);
        }
    }

    surf.write_region(bounds, &rgba);
    Layer::new("🌄 Atmospheric Backdrop [Normal]", LayerContent::Raster(surf))
}

/// Create radiant spotlight glow behind the subject
fn build_spotlight_glow(bounds: Rect, style: &str, fmt: PixelFormat, cx: i32, cy: i32, sub_h: i32) -> Layer {
    let w = bounds.width() as usize;
    let h = bounds.height() as usize;
    let total = w * h;
    let mut surf = Surface::new(fmt);
    let mut rgba = vec![0.0f32; total * 4];

    let radius = ((sub_h as f32) * 0.70).max(120.0);
    let min_y = (cy as f32 - radius).max(0.0) as usize;
    let max_y = (cy as f32 + radius).min(h as f32) as usize;
    let min_x = (cx as f32 - radius).max(0.0) as usize;
    let max_x = (cx as f32 + radius).min(w as f32) as usize;

    let (glow_r, glow_g, glow_b) = match style {
        "summer_fresh" | "tropical" => (0.60, 0.95, 1.0),
        "luxury_gold" => (0.95, 0.78, 0.40),
        "cyberpunk" => (0.25, 0.90, 1.0),
        "super_sale" => (1.0, 0.85, 0.35),
        _ => (1.0, 1.0, 1.0),
    };

    for y in min_y..max_y {
        for x in min_x..max_x {
            let dx = x as f32 - cx as f32;
            let dy = y as f32 - cy as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < radius {
                let falloff = (1.0 - dist / radius).powf(2.0) * 0.75;
                let idx = (y * w + x) * 4;
                if idx + 3 < rgba.len() {
                    rgba[idx] = glow_r;
                    rgba[idx + 1] = glow_g;
                    rgba[idx + 2] = glow_b;
                    rgba[idx + 3] = falloff;
                }
            }
        }
    }

    surf.write_region(bounds, &rgba);
    let mut layer = Layer::new("✨ Ambient Spotlight Glow [Screen]", LayerContent::Raster(surf));
    layer.blend = BlendMode::Screen;
    layer.opacity = 0.70;
    layer
}

/// Create focus vignette frame (darkens canvas corners)
fn build_vignette_layer(bounds: Rect, fmt: PixelFormat) -> Layer {
    let w = bounds.width() as usize;
    let h = bounds.height() as usize;
    let total = w * h;
    let mut surf = Surface::new(fmt);
    let mut rgba = Vec::with_capacity(total * 4);

    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let max_d = (cx * cx + cy * cy).sqrt();

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt() / max_d;
            // Smooth step from 0.55 to 1.0
            let v = if d <= 0.55 {
                0.0
            } else {
                ((d - 0.55) / 0.45).powf(1.6) * 0.55
            };
            rgba.extend_from_slice(&[0.0, 0.0, 0.0, v]);
        }
    }

    surf.write_region(bounds, &rgba);
    let mut layer = Layer::new("🌑 Vignette Focus Frame [Multiply]", LayerContent::Raster(surf));
    layer.blend = BlendMode::Multiply;
    layer.opacity = 0.35;
    layer
}

/// Helper to build an editable TextLayer
fn build_text_layer(
    doc: &Document,
    name: &str,
    text: &str,
    size_pt: f32,
    color: Color,
    x: f32,
    y: f32,
    align: TextAlign,
) -> Layer {
    let char_style = CharStyle {
        size_pt,
        color,
        tracking: 40.0,
        faux_bold: true,
        ..Default::default()
    };
    let para_style = ParagraphStyle {
        align,
        ..Default::default()
    };

    let mut t = TextLayer {
        runs: vec![TextRun { len: text.len(), style: char_style }],
        paragraphs: vec![ParagraphRun { len: text.len(), style: para_style }],
        text: text.to_string(),
        shape: TextShape::Point,
        transform: Affine::translate(f64::from(x), f64::from(y)),
        ..Default::default()
    };
    t.sync_summary();
    crate::type_cmds::refresh(doc, &mut t);

    Layer::new(name, LayerContent::Text(t))
}

fn run(s: &mut Session, p: &Value) -> Result<Value> {
    let active_id = s.active().and_then(|d| d.active_layer).ok_or(EngineError::NoDocument)?;
    let style = p.get("style").and_then(Value::as_str).unwrap_or("summer_fresh");

    let default_headline = match style {
        "luxury_gold" => "PURE LUXURY",
        "cyberpunk" => "CYBER VIBES",
        "super_sale" => "SUPER SALE",
        "minimal_clean" => "MINIMAL ART",
        _ => "FRESH SUMMER",
    };
    let headline = p.get("headline").and_then(Value::as_str).unwrap_or(default_headline);

    let default_subhead = match style {
        "luxury_gold" => "Exclusive Edition • Premium Craftsmanship",
        "cyberpunk" => "Next-Gen Experience • Limited Drop",
        "super_sale" => "Up to 50% Off • Limited Time Offer",
        "minimal_clean" => "Simplicity & Elegance in Every Detail",
        _ => "100% Natural • Refreshing Taste",
    };
    let subhead = p.get("subhead").and_then(Value::as_str).unwrap_or(default_subhead);

    let cta = p.get("cta").and_then(Value::as_str).unwrap_or("ORDER NOW");
    let badge = p.get("badge").and_then(Value::as_str).unwrap_or("SPECIAL -30%");

    s.edit("🎨 Smart Poster Composer", |doc, active| {
        let bounds = doc.bounds();
        let w = bounds.width();
        let h = bounds.height();
        let fmt = doc.pixel_format();

        if w < 100 || h < 100 {
            return Err(EngineError::Other("Document size too small for poster composition".into()));
        }

        // 1. Identify hero subject bounds
        let w_i = w as i32;
        let h_i = h as i32;
        let hero_layer = doc.layer(active_id).ok_or(EngineError::NoLayer(active_id))?;
        let hero_cb = hero_layer.surface().map(|s| s.content_bounds()).unwrap_or_else(|| {
            Rect::new(w_i / 4, h_i / 3, (w_i * 3) / 4, (h_i * 4) / 5)
        });

        let base_cx = (hero_cb.x0 + hero_cb.x1) / 2;
        let base_cy = hero_cb.y1;
        let sub_w = hero_cb.width() as i32;
        let sub_h = hero_cb.height() as i32;
        let mid_cy = base_cy - sub_h / 2;

        // 2. Build Poster layers
        // Bottom: Atmospheric Backdrop
        let backdrop = build_backdrop_layer(bounds, style, fmt, base_cx, mid_cy);

        // Behind subject: Ambient Spotlight Glow
        let spotlight = build_spotlight_glow(bounds, style, fmt, base_cx, mid_cy, sub_h);

        // Under subject: Realistic Contact & Cast Floor Shadow
        let shadow = build_shadow_layer(bounds, base_cx, base_cy, sub_w, fmt);

        // Top: Typography
        let headline_color = match style {
            "luxury_gold" => Color::rgb(0.96, 0.88, 0.65),
            "cyberpunk" => Color::rgb(0.0, 1.0, 1.0),
            "super_sale" => Color::WHITE,
            _ => Color::WHITE,
        };
        let headline_size = ((w as f32) * 0.085).clamp(32.0, 110.0);
        let headline_x = w as f32 / 2.0;
        let headline_y = (h as f32 * 0.16).max(headline_size * 1.2);
        let headline_layer = build_text_layer(
            doc,
            "🔤 Headline Text",
            headline,
            headline_size,
            headline_color,
            headline_x,
            headline_y,
            TextAlign::Center,
        );

        let subhead_color = match style {
            "luxury_gold" => Color::rgb(0.82, 0.75, 0.60),
            "cyberpunk" => Color::rgb(0.90, 0.40, 0.80),
            _ => Color::rgb(0.92, 0.94, 0.96),
        };
        let subhead_size = (headline_size * 0.30).clamp(12.0, 32.0);
        let subhead_y = headline_y + headline_size * 0.55 + 14.0;
        let subhead_layer = build_text_layer(
            doc,
            "📝 Sub-Headline & Details",
            subhead,
            subhead_size,
            subhead_color,
            headline_x,
            subhead_y,
            TextAlign::Center,
        );

        // CTA Badge Text
        let cta_color = match style {
            "luxury_gold" => Color::rgb(1.0, 0.85, 0.45),
            "cyberpunk" => Color::rgb(0.0, 1.0, 0.85),
            "super_sale" => Color::rgb(1.0, 0.90, 0.20),
            _ => Color::rgb(1.0, 0.92, 0.35),
        };
        let cta_size = (headline_size * 0.32).clamp(14.0, 34.0);
        let cta_y = (h as f32 * 0.90).min(h as f32 - 30.0);
        let cta_text = format!("{} • {}", badge, cta);
        let cta_layer = build_text_layer(
            doc,
            "🏷️ Promo Badge & CTA",
            &cta_text,
            cta_size,
            cta_color,
            headline_x,
            cta_y,
            TextAlign::Center,
        );

        // Vignette focus frame
        let vignette = build_vignette_layer(bounds, fmt);

        // 3. Assemble Master Group
        // Stack order (top to bottom):
        // - Vignette Focus Frame
        // - CTA Badge
        // - Sub-headline
        // - Headline
        // - [Hero Subject] (duplicated into group)
        // - Contact & Cast Shadow
        // - Ambient Spotlight Glow
        // - Atmospheric Backdrop
        let hero_clone = hero_layer.clone();
        let group_title = match style {
            "luxury_gold" => "📁 Smart Poster - Luxury Gold (រចនាបែបប្រណិត)",
            "cyberpunk" => "📁 Smart Poster - Cyberpunk Neon (រចនាបែបណេអុង)",
            "super_sale" => "📁 Smart Poster - Super Sale (រចនាបែបបញ្ចុះតម្លៃ)",
            "minimal_clean" => "📁 Smart Poster - Minimal Clean (រចនាបែបសាមញ្ញ)",
            _ => "📁 Smart Poster - Summer Fresh (រចនាបែបស្រស់ស្រាយ)",
        };

        let group = Layer::group(group_title, vec![
            backdrop,
            spotlight,
            shadow,
            hero_clone,
            headline_layer,
            subhead_layer,
            cta_layer,
            vignette,
        ]);

        let gid = doc.insert_above(Some(active_id), group);
        doc.remove(active_id);
        *active = Some(gid);

        Ok(json!({
            "group": gid.0,
            "style": style,
            "headline": headline,
            "subhead": subhead,
        }))
    })
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: CMD,
            label: "Smart Poster Composer",
            menu: &["Layer", "Smart Poster"],
            shortcut: None,
            params: r##"{"style":"summer_fresh"|"luxury_gold"|"cyberpunk"|"super_sale"|"minimal_clean"?,"headline":str?,"subhead":str?,"cta":str?,"badge":str?} → {group,style,headline}"##,
            enabled: has_doc,
            run,
            journal: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poster_disabled_when_no_doc() {
        assert!(!Session::new().is_enabled(CMD));
        assert!(Session::new().execute(CMD, json!({})).is_err());
    }

    #[test]
    fn poster_compose_success() {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 300, "height": 400})).unwrap();

        let res = s.execute(CMD, json!({
            "style": "summer_fresh",
            "headline": "SUMMER TROPICAL",
            "subhead": "Pure fruit juice",
            "badge": "50% OFF",
            "cta": "ORDER NOW"
        }));

        assert!(res.is_ok(), "composePoster must succeed: {:?}", res);

        let active_state = s.active().unwrap();
        let active_id = active_state.active_layer.unwrap();
        let group_layer = active_state.doc.layer(active_id).unwrap();

        // Must be a group
        assert!(group_layer.is_group());
        assert!(group_layer.name.contains("Smart Poster"));

        // Must contain 8 child layers (backdrop, spotlight, shadow, hero, headline, subhead, cta, vignette)
        let children = group_layer.children().expect("must have children");
        assert_eq!(children.len(), 8);

        // Verify layers in group
        let child_names: Vec<String> = children.iter().map(|l| l.name.clone()).collect();
        assert!(child_names.iter().any(|n| n.contains("Backdrop")));
        assert!(child_names.iter().any(|n| n.contains("Spotlight")));
        assert!(child_names.iter().any(|n| n.contains("Shadow")));
        assert!(child_names.iter().any(|n| n.contains("Headline")));
        assert!(child_names.iter().any(|n| n.contains("Sub-Headline")));
        assert!(child_names.iter().any(|n| n.contains("Badge")));
        assert!(child_names.iter().any(|n| n.contains("Vignette")));
    }

    #[test]
    fn poster_compose_all_styles() {
        let styles = ["summer_fresh", "luxury_gold", "cyberpunk", "super_sale", "minimal_clean"];
        for style in styles {
            let mut s = Session::new();
            s.execute("file.new", json!({"width": 200, "height": 200})).unwrap();
            let res = s.execute(CMD, json!({"style": style}));
            assert!(res.is_ok(), "Failed for style {}: {:?}", style, res);
        }
    }
}

