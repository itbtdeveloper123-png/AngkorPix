//! Contextual Task Bar: modern Photoshop-style floating context-sensitive toolbar
//! positioned over the canvas to provide instant, high-frequency actions with clean icons
//! and crisp typography matching Adobe Photoshop 2026.

use egui::{Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use serde_json::json;

use crate::PhotocraftApp;
use crate::state::Tool;
use crate::theme::{self, Tokens};

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
        (content_rect.center().x - 240.0).clamp(content_rect.left() + 10.0, (content_rect.right() - 500.0).max(content_rect.left() + 10.0)),
        (content_rect.bottom() - 54.0).clamp(content_rect.top() + 10.0, (content_rect.bottom() - 48.0).max(content_rect.top() + 10.0)),
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
    // Photoshop Contextual Taskbar frame: dark charcoal background, subtle dark border, soft shadow
    let bar_bg = if t.dark() { Color32::from_rgb(42, 42, 42) } else { Color32::from_rgb(238, 238, 238) };
    let bar_border = if t.dark() { Color32::from_rgb(64, 64, 64) } else { Color32::from_rgb(205, 205, 205) };
    let frame = egui::Frame::NONE
        .fill(bar_bg)
        .stroke(Stroke::new(1.0, bar_border))
        .corner_radius(CornerRadius::same(6))
        .shadow(egui::Shadow { offset: [0, 4], blur: 18, spread: 0, color: Color32::from_black_alpha(if t.dark() { 160 } else { 70 }) })
        .inner_margin(egui::Margin { left: 6, right: 6, top: 4, bottom: 4 });

    let mut new_pos = None;
    let mut hide_bar = false;
    let mut reset_pos = false;

    egui::Area::new(egui::Id::new("contextual-task-bar-area"))
        .fixed_pos(initial_pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(5.0, 0.0);

                    // 1. Drag Grip Handle: 2 columns of 6 vertical dots (Photoshop style)
                    let grip_resp = drag_handle(ui, &t);
                    let grip_resp = grip_resp.on_hover_text(tl!("Drag to reposition Contextual Task Bar"));
                    if grip_resp.dragged() {
                        let delta = grip_resp.drag_delta();
                        let cur = initial_pos;
                        new_pos = Some([cur.x + delta.x, cur.y + delta.y]);
                    }

                    separator(ui, &t);

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

                    // 3. More Options Menu (3 horizontal dots vector icon)
                    let menu_btn = task_button(ui, Some("ellipsis"), None, tl!("Task Bar Options"), &t);
                    egui::Popup::menu(&menu_btn).show(|ui| {
                        crate::widgets::style_spectrum_popup_menu(ui);
                        ui.set_min_width(190.0);

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
}

/// 2-column by 6-row dot grid drag handle matching Photoshop's Contextual Task Bar grip.
fn drag_handle(ui: &mut Ui, t: &Tokens) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(13.0, 26.0), Sense::drag());
    let resp = resp.on_hover_cursor(egui::CursorIcon::Grab);
    let painter = ui.painter();
    let dot_r = 1.15;
    let dot_color = if resp.hovered() || resp.dragged() {
        if t.dark() { Color32::from_gray(225) } else { Color32::from_gray(80) }
    } else {
        if t.dark() { Color32::from_gray(140) } else { Color32::from_gray(155) }
    };
    let cx = rect.center().x;
    let cy = rect.center().y;
    for row in 0..6 {
        let y = cy - 8.75 + (row as f32) * 3.5;
        for col in 0..2 {
            let x = cx - 2.0 + (col as f32) * 4.0;
            painter.circle_filled(pos2(x, y), dot_r, dot_color);
        }
    }
    resp
}

/// Vertical hairline separator between button groups.
fn separator(ui: &mut Ui, t: &Tokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(6.0, 26.0), Sense::hover());
    let col = if t.dark() { Color32::from_rgb(64, 64, 64) } else { Color32::from_rgb(205, 205, 205) };
    ui.painter().line_segment(
        [pos2(rect.center().x, rect.center().y - 8.0), pos2(rect.center().x, rect.center().y + 8.0)],
        Stroke::new(1.0, col),
    );
}

/// Photoshop-style rounded rectangular button with optional icon and crisp typography.
fn task_button(ui: &mut Ui, icon: Option<&str>, label: Option<&str>, tooltip: &str, t: &Tokens) -> Response {
    let height = 26.0;
    let font = theme::medium(12.0);
    let fg_text = if t.dark() { Color32::from_rgb(248, 248, 248) } else { Color32::from_rgb(30, 30, 30) };
    let fg_icon = if t.dark() { Color32::from_rgb(238, 238, 238) } else { Color32::from_rgb(45, 45, 45) };

    let (galley, text_size) = if let Some(text) = label {
        let job = egui::text::LayoutJob::simple_singleline(text.to_string(), font, fg_text);
        let g = ui.painter().layout_job(job);
        let s = g.size();
        (Some(g), s)
    } else {
        (None, Vec2::ZERO)
    };

    let width = match (icon, label) {
        (Some(_), Some(_)) => 8.0 + 15.0 + 6.0 + text_size.x + 8.0,
        (Some(_), None) => 28.0,
        (None, Some(_)) => 10.0 + text_size.x + 10.0,
        (None, None) => 28.0,
    };

    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());

    // Background & Stroke styling matching Photoshop 2026 Contextual Task Bar
    let (bg, stroke_col) = if resp.is_pointer_button_down_on() {
        if t.dark() { (Color32::from_rgb(38, 38, 38), Color32::from_rgb(90, 90, 90)) } else { (Color32::from_rgb(200, 200, 200), Color32::from_rgb(160, 160, 160)) }
    } else if resp.hovered() {
        if t.dark() { (Color32::from_rgb(66, 66, 66), Color32::from_rgb(115, 115, 115)) } else { (Color32::from_rgb(222, 222, 222), Color32::from_rgb(175, 175, 175)) }
    } else {
        if t.dark() { (Color32::from_rgb(52, 52, 52), Color32::from_rgb(72, 72, 72)) } else { (Color32::from_rgb(245, 245, 245), Color32::from_rgb(215, 215, 215)) }
    };

    let painter = ui.painter();
    painter.rect(rect, CornerRadius::same(4), bg, Stroke::new(1.0, stroke_col), StrokeKind::Inside);

    // Draw Icon & Label
    match (icon, label) {
        (Some(icon_name), Some(_)) => {
            let icon_rect = Rect::from_center_size(pos2(rect.left() + 8.0 + 7.5, rect.center().y), Vec2::splat(15.0));
            crate::icons::image(icon_name, 15.0, fg_icon).paint_at(ui, icon_rect);
            if let Some(g) = galley {
                let text_pos = pos2(rect.left() + 8.0 + 15.0 + 6.0, rect.center().y - g.size().y / 2.0);
                painter.galley(text_pos, g, fg_text);
            }
        }
        (Some(icon_name), None) => {
            let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(16.0));
            crate::icons::image(icon_name, 16.0, fg_icon).paint_at(ui, icon_rect);
        }
        (None, Some(_)) => {
            if let Some(g) = galley {
                painter.galley(rect.center() - g.size() / 2.0, g, fg_text);
            }
        }
        (None, None) => {}
    }

    if !tooltip.is_empty() {
        resp.on_hover_text(tooltip)
    } else {
        resp
    }
}

/// Context when there is an active selection
fn render_selection_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    if task_button(ui, Some("lasso-select"), Some(tl!("Select and Mask…")), tl!("Refine selection edge (Cmd+Alt+R)"), t).clicked() {
        let _ = app.run("select.selectAndMask", json!({}));
    }

    if task_button(ui, Some("arrow-left-right"), Some(tl!("Invert")), tl!("Inverse Selection (Cmd+Shift+I)"), t).clicked() {
        let _ = app.run("select.inverse", json!({}));
    }

    if task_button(ui, Some("paint-bucket"), Some(tl!("Fill…")), tl!("Fill Selection (Shift+F5)"), t).clicked() {
        let _ = app.run("edit.fill", json!({}));
    }

    if task_button(ui, Some("layer-mask"), Some(tl!("Mask")), tl!("Create Layer Mask from Selection"), t).clicked() {
        let _ = app.run("layer.layerMask.revealSelection", json!({}));
    }

    // Quick adjustment popup
    let adj_btn = task_button(ui, Some("sliders-horizontal"), Some(tl!("Adjust Colors")), tl!("Add Adjustment Layer"), t);
    egui::Popup::menu(&adj_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(170.0);

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

    separator(ui, t);

    if task_button(ui, Some("scaling"), None, tl!("Transform Selection"), t).clicked() {
        let _ = app.run("select.transformSelection", json!({}));
    }

    if task_button(ui, Some("x"), None, tl!("Deselect (Cmd+D)"), t).clicked() {
        let _ = app.run("select.deselect", json!({}));
    }
}

/// Context when Type tool or a Text Layer is active
fn render_type_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    let _ = task_button(ui, Some("type"), Some(tl!("Text Layer")), "", t);

    separator(ui, t);

    if task_button(ui, Some("align-left"), None, tl!("Align Left"), t).clicked() {
        let _ = app.run("type.align.left", json!({}));
    }
    if task_button(ui, Some("align-center"), None, tl!("Align Center"), t).clicked() {
        let _ = app.run("type.align.center", json!({}));
    }
    if task_button(ui, Some("align-right"), None, tl!("Align Right"), t).clicked() {
        let _ = app.run("type.align.right", json!({}));
    }

    separator(ui, t);

    if task_button(ui, Some("check"), None, tl!("Commit Type"), t).clicked() {
        let _ = app.run("type.commit", json!({}));
    }
    if task_button(ui, Some("x"), None, tl!("Cancel Type"), t).clicked() {
        let _ = app.run("type.cancel", json!({}));
    }
}

/// Context when Crop tool is active
fn render_crop_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    let _ = task_button(ui, Some("crop"), Some(tl!("Crop")), "", t);

    separator(ui, t);

    if task_button(ui, Some("ruler"), Some(tl!("Straighten")), tl!("Straighten image"), t).clicked() {
        let _ = app.run("image.crop.straighten", json!({}));
    }

    separator(ui, t);

    if task_button(ui, Some("check"), None, tl!("Commit Crop (Enter)"), t).clicked() {
        let _ = app.run("image.crop.commit", json!({}));
    }
    if task_button(ui, Some("x"), None, tl!("Cancel Crop (Esc)"), t).clicked() {
        let _ = app.run("image.crop.cancel", json!({}));
    }
}

/// Context when Shape layer is active
fn render_shape_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    let _ = task_button(ui, Some("pentagon"), Some(tl!("Shape Layer")), "", t);

    separator(ui, t);

    if task_button(ui, Some("square-dashed"), Some(tl!("To Selection")), tl!("Convert Shape to Selection"), t).clicked() {
        let _ = app.run("select.fromPath", json!({}));
    }
    if task_button(ui, Some("brush"), Some(tl!("Rasterize")), tl!("Rasterize Shape Layer"), t).clicked() {
        let _ = app.run("layer.rasterize.shape", json!({}));
    }
}

/// Default context when idle on an image/raster layer with no active selection (matches Photoshop 2026)
fn render_default_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    // 1. Select Subject (wand-sparkles icon + label)
    if task_button(ui, Some("wand-sparkles"), Some(tl!("Select Subject")), tl!("Automatically detect and select subject"), t).clicked() {
        let _ = app.run("select.subject", json!({}));
    }

    // 2. Remove Background (eraser-background icon + label)
    if task_button(ui, Some("eraser-background"), Some(tl!("Remove Background")), tl!("Automatically isolate subject and mask out background"), t).clicked() {
        let _ = app.run("select.subject", json!({}));
        let _ = app.run("layer.layerMask.revealSelection", json!({}));
    }

    // 3. Adjust Colors (sliders-horizontal icon + label)
    let adj_btn = task_button(ui, Some("sliders-horizontal"), Some(tl!("Adjust Colors")), tl!("Add Adjustment Layer"), t);
    egui::Popup::menu(&adj_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(170.0);

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

    separator(ui, t);

    // 4. Free Transform icon button
    if task_button(ui, Some("scaling"), None, tl!("Free Transform (Cmd+T)"), t).clicked() {
        let _ = app.run("edit.freeTransform", json!({}));
    }

    // 5. Adjustment Layer icon button
    let adj_icon_btn = task_button(ui, Some("adjustment-layer"), None, tl!("New Adjustment Layer…"), t);
    egui::Popup::menu(&adj_icon_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(170.0);

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
    });

    // 6. Place Embedded / Add image icon button
    if task_button(ui, Some("image"), None, tl!("Place Embedded…"), t).clicked() {
        let _ = app.run("file.placeEmbedded", json!({}));
    }
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
