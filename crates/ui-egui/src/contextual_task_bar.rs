//! Contextual Task Bar: modern Photoshop-style floating context-sensitive toolbar
//! positioned over the canvas to provide instant, high-frequency actions.

use egui::{Color32, CornerRadius, Sense, Stroke, pos2, vec2};
use serde_json::json;

use crate::PhotocraftApp;
use crate::state::Tool;
use crate::theme::Tokens;
use crate::widgets;

/// Renders the floating Contextual Task Bar over the canvas when enabled.
pub fn window(app: &mut PhotocraftApp, ctx: &egui::Context) {
    if !app.ui.panels.contextual_task_bar {
        return;
    }

    // Only render when a document is open and active
    let (has_selection, is_text_layer, is_shape_layer) = {
        let Some(st) = app.session.active() else {
            return;
        };
        let sel = st.doc.selection.is_some();
        let active_layer = st.active_layer.and_then(|id| st.doc.layer(id));
        let text = active_layer.is_some_and(|l| matches!(l.content, photocraft_doc::LayerContent::Text(_)));
        let shape = active_layer.is_some_and(|l| matches!(l.content, photocraft_doc::LayerContent::Shape(_)));
        (sel, text, shape)
    };

    let content_rect = crate::rulers::content_rect(app, app.last_canvas_rect);
    if content_rect.width() < 100.0 || content_rect.height() < 60.0 {
        return;
    }

    let default_pos = pos2(
        (content_rect.center().x - 220.0).clamp(content_rect.left() + 10.0, (content_rect.right() - 480.0).max(content_rect.left() + 10.0)),
        (content_rect.bottom() - 60.0).clamp(content_rect.top() + 10.0, (content_rect.bottom() - 48.0).max(content_rect.top() + 10.0)),
    );

    let initial_pos = if let Some([x, y]) = app.ui.contextual_task_bar_pos {
        pos2(
            x.clamp(content_rect.left(), (content_rect.right() - 100.0).max(content_rect.left())),
            y.clamp(content_rect.top(), (content_rect.bottom() - 40.0).max(content_rect.top())),
        )
    } else {
        default_pos
    };

    let t = Tokens::get(ctx);
    let frame = egui::Frame::NONE
        .fill(t.card)
        .stroke(Stroke::new(1.0, t.card_border))
        .corner_radius(CornerRadius::same(t.radius_lg as u8))
        .shadow(egui::Shadow { offset: [0, 4], blur: 16, spread: 0, color: t.shadow })
        .inner_margin(egui::Margin { left: 8, right: 8, top: 4, bottom: 4 });

    let mut new_pos = None;
    let mut hide_bar = false;
    let mut reset_pos = false;

    let area_res = egui::Area::new(egui::Id::new("contextual-task-bar-area"))
        .fixed_pos(initial_pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);

                    // 1. Drag Grip Handle
                    let grip_resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new("⠿")
                                .size(14.0)
                                .color(if t.pro { Color32::from_gray(140) } else { Color32::from_gray(100) }),
                        )
                        .sense(Sense::drag()),
                    );
                    let grip_resp = grip_resp.on_hover_text(tl!("Drag to reposition Contextual Task Bar"));
                    if grip_resp.dragged() {
                        let delta = grip_resp.drag_delta();
                        let cur = initial_pos;
                        new_pos = Some([cur.x + delta.x, cur.y + delta.y]);
                    }

                    widgets::vline(ui, 18.0);

                    // 2. Context-Sensitive Action Buttons
                    let tool = app.ui.tool;
                    if has_selection {
                        render_selection_context(app, ui, &t);
                    } else if tool == Tool::Type || is_text_layer {
                        render_type_context(app, ui, &t);
                    } else if tool == Tool::Crop || tool == Tool::PerspectiveCrop {
                        render_crop_context(app, ui, &t);
                    } else if is_shape_layer {
                        render_shape_context(app, ui, &t);
                    } else {
                        render_default_context(app, ui, &t);
                    }

                    widgets::vline(ui, 18.0);

                    // 3. More Options Menu (3 dots)
                    let menu_btn = ui.add(
                        egui::Button::new(egui::RichText::new("⋮").size(15.0).strong())
                            .frame(false)
                            .min_size(vec2(18.0, 24.0)),
                    ).on_hover_text(tl!("Task Bar Options"));

                    egui::Popup::menu(&menu_btn).show(|ui| {
                        widgets::style_spectrum_popup_menu(ui);
                        ui.set_min_width(180.0);

                        let mut pinned = app.ui.contextual_task_bar_pinned;
                        if ui.checkbox(&mut pinned, tl!("Pin bar position")).clicked() {
                            app.ui.contextual_task_bar_pinned = pinned;
                            ui.close();
                        }

                        if ui.button(tl!("Reset bar position")).clicked() {
                            reset_pos = true;
                            ui.close();
                        }

                        ui.separator();

                        if ui.button(tl!("Hide Contextual Task Bar")).clicked() {
                            hide_bar = true;
                            ui.close();
                        }
                    });
                });
            });
        });

    if let Some(pos) = new_pos {
        app.ui.contextual_task_bar_pos = Some(pos);
    }
    if reset_pos {
        app.ui.contextual_task_bar_pos = None;
    }
    if hide_bar {
        app.ui.panels.contextual_task_bar = false;
    }

    let _ = area_res;
}

fn task_button(ui: &mut Ui, label: &str, tooltip: &str) -> egui::Response {
    let resp = widgets::compact_button(ui, label, 24.0);
    if !tooltip.is_empty() {
        resp.on_hover_text(tooltip)
    } else {
        resp
    }
}

use egui::Ui;

/// Context when there is an active selection
fn render_selection_context(app: &mut PhotocraftApp, ui: &mut Ui, _t: &Tokens) {
    ui.label(egui::RichText::new("⬚").size(13.0));

    if task_button(ui, tl!("Fill…"), tl!("Fill Selection (Shift+F5)")).clicked() {
        let _ = app.run("edit.fill", json!({}));
    }

    if task_button(ui, tl!("Select and Mask…"), tl!("Refine selection edge (Cmd+Alt+R)")).clicked() {
        let _ = app.run("select.selectAndMask", json!({}));
    }

    if task_button(ui, tl!("Invert"), tl!("Inverse Selection (Cmd+Shift+I)")).clicked() {
        let _ = app.run("select.inverse", json!({}));
    }

    if task_button(ui, tl!("Transform"), tl!("Transform Selection")).clicked() {
        let _ = app.run("select.transformSelection", json!({}));
    }

    if task_button(ui, tl!("Mask"), tl!("Create Layer Mask from Selection")).clicked() {
        let _ = app.run("layer.layerMask.revealSelection", json!({}));
    }

    // Quick adjustment popup
    let adj_btn = ui.add(
        egui::Button::new(format!("{} ▾", tl!("Adjust")))
            .min_size(vec2(20.0, 24.0)),
    ).on_hover_text(tl!("Add Adjustment Layer"));

    egui::Popup::menu(&adj_btn).show(|ui| {
        widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(160.0);

        if ui.button(tl!("Curves…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.curves", json!({}));
            ui.close();
        }
        if ui.button(tl!("Levels…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.levels", json!({}));
            ui.close();
        }
        if ui.button(tl!("Brightness/Contrast…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.brightnessContrast", json!({}));
            ui.close();
        }
        if ui.button(tl!("Hue/Saturation…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.hueSaturation", json!({}));
            ui.close();
        }
        if ui.button(tl!("Black & White…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.blackAndWhite", json!({}));
            ui.close();
        }
    });

    if task_button(ui, tl!("Deselect"), tl!("Deselect (Cmd+D)")).clicked() {
        let _ = app.run("select.deselect", json!({}));
    }
}

/// Context when Type tool or a Text Layer is active
fn render_type_context(app: &mut PhotocraftApp, ui: &mut Ui, _t: &Tokens) {
    ui.label(egui::RichText::new("T").size(14.0).strong());

    ui.label(tl!("Text Layer"));

    widgets::vline(ui, 16.0);

    if task_button(ui, "Left", tl!("Align Left")).clicked() {
        let _ = app.run("type.align.left", json!({}));
    }
    if task_button(ui, "Center", tl!("Align Center")).clicked() {
        let _ = app.run("type.align.center", json!({}));
    }
    if task_button(ui, "Right", tl!("Align Right")).clicked() {
        let _ = app.run("type.align.right", json!({}));
    }

    widgets::vline(ui, 16.0);

    if task_button(ui, "✓", tl!("Commit Type")).clicked() {
        let _ = app.run("type.commit", json!({}));
    }
    if task_button(ui, "✕", tl!("Cancel Type")).clicked() {
        let _ = app.run("type.cancel", json!({}));
    }
}

/// Context when Crop tool is active
fn render_crop_context(app: &mut PhotocraftApp, ui: &mut Ui, _t: &Tokens) {
    ui.label(egui::RichText::new("⯐").size(13.0));
    ui.label(tl!("Crop"));

    widgets::vline(ui, 16.0);

    if task_button(ui, tl!("Straighten"), tl!("Straighten image")).clicked() {
        let _ = app.run("image.crop.straighten", json!({}));
    }

    widgets::vline(ui, 16.0);

    if task_button(ui, "✓", tl!("Commit Crop (Enter)")).clicked() {
        let _ = app.run("image.crop.commit", json!({}));
    }
    if task_button(ui, "✕", tl!("Cancel Crop (Esc)")).clicked() {
        let _ = app.run("image.crop.cancel", json!({}));
    }
}

/// Context when Shape layer is active
fn render_shape_context(app: &mut PhotocraftApp, ui: &mut Ui, _t: &Tokens) {
    ui.label(egui::RichText::new("⬟").size(13.0));
    ui.label(tl!("Shape Layer"));

    widgets::vline(ui, 16.0);

    if task_button(ui, tl!("To Selection"), tl!("Convert Shape to Selection")).clicked() {
        let _ = app.run("select.fromPath", json!({}));
    }
    if task_button(ui, tl!("Rasterize"), tl!("Rasterize Shape Layer")).clicked() {
        let _ = app.run("layer.rasterize.shape", json!({}));
    }
}

/// Default context when idle on an image/raster layer with no active selection
fn render_default_context(app: &mut PhotocraftApp, ui: &mut Ui, _t: &Tokens) {
    ui.label(egui::RichText::new("✦").size(13.0));

    if task_button(ui, tl!("Select Subject"), tl!("Automatically detect and select subject")).clicked() {
        let _ = app.run("select.subject", json!({}));
    }

    if task_button(ui, tl!("Remove Background"), tl!("Automatically isolate subject and mask out background")).clicked() {
        // Photoshop Remove Background: Select Subject followed by Layer Mask Reveal Selection
        let _ = app.run("select.subject", json!({}));
        let _ = app.run("layer.layerMask.revealSelection", json!({}));
    }

    if task_button(ui, tl!("Select Sky"), tl!("Automatically detect and select sky")).clicked() {
        let _ = app.run("select.sky", json!({}));
    }

    widgets::vline(ui, 16.0);

    if task_button(ui, tl!("Transform"), tl!("Free Transform (Cmd+T)")).clicked() {
        let _ = app.run("edit.freeTransform", json!({}));
    }

    // Quick adjustment popup
    let adj_btn = ui.add(
        egui::Button::new(format!("{} ▾", tl!("Adjust")))
            .min_size(vec2(20.0, 24.0)),
    ).on_hover_text(tl!("Add Adjustment Layer"));

    egui::Popup::menu(&adj_btn).show(|ui| {
        widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(160.0);

        if ui.button(tl!("Curves…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.curves", json!({}));
            ui.close();
        }
        if ui.button(tl!("Levels…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.levels", json!({}));
            ui.close();
        }
        if ui.button(tl!("Brightness/Contrast…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.brightnessContrast", json!({}));
            ui.close();
        }
        if ui.button(tl!("Hue/Saturation…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.hueSaturation", json!({}));
            ui.close();
        }
        if ui.button(tl!("Black & White…")).clicked() {
            let _ = app.run("layer.newAdjustmentLayer.blackAndWhite", json!({}));
            ui.close();
        }
    });
}

#[cfg(test)]
mod tests {

    #[test]
    fn contextual_task_bar_panel_state() {
        let panels = crate::state::Panels::default();
        assert!(panels.contextual_task_bar, "Contextual Task Bar should be enabled by default");
    }

    #[test]
    fn contextual_task_bar_menu_alias() {
        assert_eq!(crate::menus::panel_alias("window.panel.contextualTaskBar"), Some("window.toggle.contextualTaskBar"));
    }
}
