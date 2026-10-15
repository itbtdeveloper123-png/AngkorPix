//! Photoshop Adjustment Presets: one-click curated photo adjustment recipes (Portraits,
//! Landscape, Photo Repair, Creative, Black & White).
//!
//! Each preset adds one or more adjustment layers to the document with curated parameters,
//! optionally grouped into a named Adjustment Group (Photoshop 2024–2026).

use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::{EngineError, Result, Session};

fn bad(cmd: &str, msg: impl Into<String>) -> EngineError {
    EngineError::BadParams {
        cmd: cmd.into(),
        msg: msg.into(),
    }
}

fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no document open".into())
}

/// A curated adjustment preset definition.
#[derive(Clone, Copy, Debug)]
pub struct PresetDef {
    pub id: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub badge_colors: [[u8; 3]; 2],
    pub steps: &'static [(&'static str, &'static str)],
}

/// Master list of built-in modern adjustment presets.
pub const PRESETS: &[PresetDef] = &[
    // --- Portraits ---
    PresetDef {
        id: "portrait_warm_glow",
        label: "Warm Glow",
        category: "Portraits",
        description: "Warm, radiant cast that enhances skin tones and portraits",
        badge_colors: [[255, 180, 100], [235, 120, 50]],
        steps: &[
            ("photoFilter", r#"{"filter":"warming85","density":30}"#),
            ("brightnessContrast", r#"{"brightness":5,"contrast":10}"#),
        ],
    },
    PresetDef {
        id: "portrait_cool_light",
        label: "Cool Light",
        category: "Portraits",
        description: "Crisp, modern cool-toned lighting with clean contrast",
        badge_colors: [[120, 190, 255], [60, 110, 200]],
        steps: &[
            ("photoFilter", r#"{"filter":"cooling80","density":25}"#),
            ("brightnessContrast", r#"{"contrast":15}"#),
        ],
    },
    PresetDef {
        id: "portrait_soft_bright",
        label: "Soft Bright",
        category: "Portraits",
        description: "Flattering gentle bright tones with soft contrast",
        badge_colors: [[255, 225, 200], [240, 170, 160]],
        steps: &[
            ("brightnessContrast", r#"{"brightness":12,"contrast":-5}"#),
            ("vibrance", r#"{"vibrance":15,"saturation":5}"#),
        ],
    },
    PresetDef {
        id: "portrait_moody",
        label: "Moody Portrait",
        category: "Portraits",
        description: "Subtle cyan-shadow tint with warm highlights for editorial drama",
        badge_colors: [[80, 100, 120], [180, 140, 110]],
        steps: &[
            ("colorBalance", r#"{"shadows":[-8,0,8],"highlights":[10,4,-8]}"#),
            ("brightnessContrast", r#"{"contrast":15}"#),
        ],
    },

    // --- Landscape ---
    PresetDef {
        id: "landscape_pop",
        label: "Landscape Pop",
        category: "Landscape",
        description: "Punches up colors, skies, and foliage with balanced contrast",
        badge_colors: [[40, 180, 120], [20, 120, 220]],
        steps: &[
            ("vibrance", r#"{"vibrance":35,"saturation":10}"#),
            ("brightnessContrast", r#"{"contrast":15}"#),
        ],
    },
    PresetDef {
        id: "landscape_sunset",
        label: "Golden Sunset",
        category: "Landscape",
        description: "Rich amber highlights and warm magic hour glow",
        badge_colors: [[255, 120, 40], [180, 30, 80]],
        steps: &[
            ("photoFilter", r#"{"filter":"warming85","density":45}"#),
            ("colorBalance", r#"{"highlights":[15,-5,-12],"shadows":[5,0,-5]}"#),
        ],
    },
    PresetDef {
        id: "landscape_forest",
        label: "Deep Forest",
        category: "Landscape",
        description: "Deepens foliage greens and intensifies natural shadows",
        badge_colors: [[20, 100, 50], [50, 180, 90]],
        steps: &[
            ("hueSaturation", r#"{"yellows":{"hue":5,"saturation":15},"greens":{"hue":0,"saturation":20}}"#),
            ("brightnessContrast", r#"{"contrast":15}"#),
        ],
    },
    PresetDef {
        id: "landscape_vivid_sky",
        label: "Vivid Sky",
        category: "Landscape",
        description: "Enhances blue saturation and cloud definition in open skies",
        badge_colors: [[0, 150, 255], [0, 80, 190]],
        steps: &[
            ("hueSaturation", r#"{"cyans":{"saturation":25},"blues":{"saturation":30}}"#),
            ("brightnessContrast", r#"{"contrast":10}"#),
        ],
    },

    // --- Photo Repair ---
    PresetDef {
        id: "restore_fade_fix",
        label: "Fade Fix",
        category: "Photo Repair",
        description: "Restores washed-out, faded scans and lifts deep shadow detail",
        badge_colors: [[120, 120, 120], [20, 20, 20]],
        steps: &[
            ("levels", r#"{"inBlack":20,"inWhite":240,"gamma":1.05}"#),
            ("brightnessContrast", r#"{"contrast":15}"#),
        ],
    },
    PresetDef {
        id: "restore_contrast_boost",
        label: "Contrast Boost",
        category: "Photo Repair",
        description: "Reclaims dynamic range on flat or underexposed photos",
        badge_colors: [[90, 90, 100], [200, 200, 220]],
        steps: &[
            ("brightnessContrast", r#"{"brightness":5,"contrast":30}"#),
        ],
    },
    PresetDef {
        id: "restore_auto_tone",
        label: "Auto Tone Balance",
        category: "Photo Repair",
        description: "Equalizes midtone distribution for natural lighting balance",
        badge_colors: [[160, 180, 200], [100, 120, 140]],
        steps: &[
            ("levels", r#"{"gamma":1.15}"#),
            ("vibrance", r#"{"vibrance":15}"#),
        ],
    },

    // --- Creative ---
    PresetDef {
        id: "creative_teal_orange",
        label: "Teal & Orange",
        category: "Creative",
        description: "Iconic cinematic color grading with teal shadows and warm orange skin tones",
        badge_colors: [[0, 160, 180], [240, 120, 40]],
        steps: &[
            ("colorLookup", r#"{"lut":"tealOrange"}"#),
        ],
    },
    PresetDef {
        id: "creative_faded_film",
        label: "Faded Film",
        category: "Creative",
        description: "Muted analog tones with lifted matte blacks and nostalgic softness",
        badge_colors: [[190, 160, 140], [110, 90, 80]],
        steps: &[
            ("colorLookup", r#"{"lut":"fadedFilm"}"#),
        ],
    },
    PresetDef {
        id: "creative_bleach_bypass",
        label: "Bleach Bypass",
        category: "Creative",
        description: "High contrast silver-halide film look with reduced color saturation",
        badge_colors: [[150, 150, 140], [70, 70, 65]],
        steps: &[
            ("colorLookup", r#"{"lut":"bleachBypass"}"#),
        ],
    },
    PresetDef {
        id: "creative_cyberpunk",
        label: "Cyberpunk",
        category: "Creative",
        description: "Electric neon styling with saturated blues, violets, and magenta highlights",
        badge_colors: [[30, 20, 80], [240, 0, 180]],
        steps: &[
            ("colorBalance", r#"{"shadows":[-15,0,25],"highlights":[20,-10,15]}"#),
            ("vibrance", r#"{"vibrance":25,"saturation":15}"#),
        ],
    },

    // --- Black & White ---
    PresetDef {
        id: "bw_high_contrast",
        label: "High Contrast B&W",
        category: "Black & White",
        description: "Dramatic black and white with deep crushed blacks and brilliant highlights",
        badge_colors: [[20, 20, 20], [240, 240, 240]],
        steps: &[
            ("blackWhite", r#"{"weights":[40,60,40,60,20,80]}"#),
            ("brightnessContrast", r#"{"contrast":25}"#),
        ],
    },
    PresetDef {
        id: "bw_sepia",
        label: "Sepia Tone",
        category: "Black & White",
        description: "Vintage warm monochrome tint evocative of 19th-century photography",
        badge_colors: [[172, 122, 51], [235, 200, 150]],
        steps: &[
            ("blackWhite", r#"{}"#),
            ("photoFilter", r#"{"filter":"sepia","density":50}"#),
        ],
    },
    PresetDef {
        id: "bw_silver_gelatin",
        label: "Silver Gelatin",
        category: "Black & White",
        description: "Fine-art darkroom finish with silky smooth tonal gradations",
        badge_colors: [[60, 65, 70], [180, 185, 190]],
        steps: &[
            ("blackWhite", r#"{"weights":[30,45,30,50,30,70]}"#),
            ("brightnessContrast", r#"{"contrast":10}"#),
        ],
    },
    PresetDef {
        id: "bw_moody_dark",
        label: "Moody Dark B&W",
        category: "Black & White",
        description: "Brooding monochrome with dense shadows and restrained midtones",
        badge_colors: [[10, 10, 10], [100, 100, 100]],
        steps: &[
            ("blackWhite", r#"{"weights":[15,30,15,30,10,50]}"#),
            ("brightnessContrast", r#"{"brightness":-10,"contrast":20}"#),
        ],
    },
];

/// Find a preset definition by its stable identifier.
pub fn find_preset(id: &str) -> Option<&'static PresetDef> {
    PRESETS.iter().find(|p| p.id.eq_ignore_ascii_case(id) || p.label.eq_ignore_ascii_case(id))
}

/// Distinct categories in display order.
pub const CATEGORIES: &[&str] = &[
    "Portraits",
    "Landscape",
    "Photo Repair",
    "Creative",
    "Black & White",
];

/// Apply a preset to the active document.
pub fn apply_preset(s: &mut Session, p: &Value) -> Result<Value> {
    let preset_id = p.get("preset").and_then(Value::as_str).ok_or_else(|| bad("adjustment.presets.apply", "missing `preset`"))?;
    let preset = find_preset(preset_id).ok_or_else(|| bad("adjustment.presets.apply", format!("unknown preset `{preset_id}`")))?;

    // Make sure there is an active document before proceeding.
    if s.active().is_none() {
        return Err(EngineError::NoDocument);
    }

    let mut created_layers = Vec::with_capacity(preset.steps.len());

    for &(kind, params_str) in preset.steps {
        let params: Value = serde_json::from_str(params_str)
            .map_err(|e| bad("adjustment.presets.apply", format!("corrupt preset definition: {e}")))?;
        let res = s.execute(&format!("layer.newAdjustmentLayer.{kind}"), params)?;
        if let Some(lid) = res.get("layer").and_then(Value::as_u64) {
            created_layers.push(lid);
        }
    }

    // If multiple adjustment layers were created and grouping is not disabled, group them together.
    let should_group = p.get("group").and_then(Value::as_bool).unwrap_or(true);
    let group_id = if created_layers.len() > 1 && should_group {
        for &lid in &created_layers {
            let _ = s.execute("layer.select", json!({ "layer": lid, "mode": "add" }));
        }
        let group_res = s.execute("layer.groupLayers", json!({ "name": format!("Preset - {}", preset.label) }))?;
        group_res.get("layer").and_then(Value::as_u64)
    } else {
        None
    };

    Ok(json!({
        "preset": preset.id,
        "label": preset.label,
        "category": preset.category,
        "layers": created_layers,
        "group": group_id,
    }))
}

/// List all available adjustment presets and categories.
pub fn list_presets(_: &mut Session, p: &Value) -> Result<Value> {
    let filter_category = p.get("category").and_then(Value::as_str);
    let filter_search = p.get("search").and_then(Value::as_str).map(str::to_lowercase);

    let items: Vec<Value> = PRESETS
        .iter()
        .filter(|p| {
            if let Some(cat) = filter_category
                && !cat.eq_ignore_ascii_case("all")
                && !p.category.eq_ignore_ascii_case(cat)
            {
                return false;
            }
            if let Some(ref q) = filter_search
                && !p.label.to_lowercase().contains(q)
                && !p.description.to_lowercase().contains(q)
                && !p.category.to_lowercase().contains(q)
            {
                return false;
            }
            true
        })
        .map(|p| {
            json!({
                "id": p.id,
                "label": p.label,
                "category": p.category,
                "description": p.description,
                "badge": [
                    format!("#{:02x}{:02x}{:02x}", p.badge_colors[0][0], p.badge_colors[0][1], p.badge_colors[0][2]),
                    format!("#{:02x}{:02x}{:02x}", p.badge_colors[1][0], p.badge_colors[1][1], p.badge_colors[1][2]),
                ],
                "layers": p.steps.len(),
            })
        })
        .collect();

    Ok(json!({
        "categories": CATEGORIES,
        "presets": items,
    }))
}

/// Register engine commands.
pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "adjustment.presets.apply",
            label: "Apply Adjustment Preset",
            menu: &[],
            shortcut: None,
            params: r##"{"preset":"portrait_warm_glow|landscape_pop|creative_teal_orange|bw_sepia|...","group":bool=true}"##,
            enabled: has_doc,
            run: apply_preset,
            journal: true,
        },
        CommandSpec {
            id: "adjustment.presets.list",
            label: "List Adjustment Presets",
            menu: &[],
            shortcut: None,
            params: r##"{"category":str?,"search":str?}"##,
            enabled: |_: &Session| Ok(()),
            run: list_presets,
            journal: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_session() -> Session {
        let mut s = Session::new();
        s.execute("file.new", json!({ "width": 64, "height": 64, "name": "test.psd" })).unwrap();
        s
    }

    #[test]
    fn presets_list_returns_all_presets() {
        let mut s = Session::new();
        let list = s.execute("adjustment.presets.list", json!({})).unwrap();
        let presets = list["presets"].as_array().unwrap();
        assert_eq!(presets.len(), PRESETS.len());
        let categories = list["categories"].as_array().unwrap();
        assert_eq!(categories.len(), CATEGORIES.len());
    }

    #[test]
    fn presets_list_filters_by_category() {
        let mut s = Session::new();
        let list = s.execute("adjustment.presets.list", json!({ "category": "Creative" })).unwrap();
        let presets = list["presets"].as_array().unwrap();
        assert!(presets.iter().all(|p| p["category"].as_str() == Some("Creative")));
        assert!(!presets.is_empty());
    }

    #[test]
    fn apply_preset_single_adjustment() {
        let mut s = test_session();
        let before_count = s.active().unwrap().doc.layers.len();
        let res = s.execute("adjustment.presets.apply", json!({ "preset": "creative_teal_orange" })).unwrap();
        assert_eq!(res["preset"].as_str(), Some("creative_teal_orange"));
        assert_eq!(s.active().unwrap().doc.layers.len(), before_count + 1);
    }

    #[test]
    fn apply_preset_grouped_multi_adjustment() {
        let mut s = test_session();
        let before_count = s.active().unwrap().doc.layers.len();
        let res = s.execute("adjustment.presets.apply", json!({ "preset": "portrait_warm_glow", "group": true })).unwrap();
        assert_eq!(res["preset"].as_str(), Some("portrait_warm_glow"));
        assert!(res["group"].as_u64().is_some());
        // Added 1 group containing 2 adjustment layers
        assert_eq!(s.active().unwrap().doc.layers.len(), before_count + 1);
    }

    #[test]
    fn apply_preset_graceful_failures() {
        let mut s = test_session();
        // Missing preset
        assert!(s.execute("adjustment.presets.apply", json!({})).is_err());
        // Unknown preset
        assert!(s.execute("adjustment.presets.apply", json!({ "preset": "unknown_nonsense" })).is_err());
        // No document
        let mut empty_session = Session::new();
        assert!(empty_session.execute("adjustment.presets.apply", json!({ "preset": "creative_teal_orange" })).is_err());
    }
}
