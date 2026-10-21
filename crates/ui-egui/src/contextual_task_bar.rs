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
    // Modern floating island frame: dark charcoal background, subtle dual-tone border, soft ambient shadow
    let bar_bg = if t.dark() { Color32::from_rgb(34, 34, 36) } else { Color32::from_rgb(246, 246, 248) };
    let bar_border = if t.dark() { Color32::from_rgba_premultiplied(255, 255, 255, 28) } else { Color32::from_rgb(212, 212, 216) };
    let frame = egui::Frame::NONE
        .fill(bar_bg)
        .stroke(Stroke::new(1.0, bar_border))
        .corner_radius(CornerRadius::same(8))
        .shadow(egui::Shadow { offset: [0, 6], blur: 24, spread: 1, color: Color32::from_black_alpha(if t.dark() { 190 } else { 75 }) })
        .inner_margin(egui::Margin { left: 8, right: 8, top: 4, bottom: 4 });

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
                    if app.ui.text_edit.is_some() || tool.is_type() {
                        render_type_context(app, ui, &t);
                    } else if has_selection {
                        render_selection_context(app, ui, &t);
                    } else if is_text_layer {
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
    let height = 27.0;
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

    // Background & Stroke styling matching modern pro software (subtle glass highlight)
    let (bg, stroke_col) = if resp.is_pointer_button_down_on() {
        if t.dark() { (Color32::from_rgb(28, 28, 30), Color32::from_rgb(100, 100, 105)) } else { (Color32::from_rgb(205, 205, 208), Color32::from_rgb(160, 160, 160)) }
    } else if resp.hovered() {
        if t.dark() { (Color32::from_rgb(58, 58, 62), Color32::from_rgb(115, 115, 120)) } else { (Color32::from_rgb(228, 228, 232), Color32::from_rgb(175, 175, 175)) }
    } else {
        if t.dark() { (Color32::from_rgb(46, 46, 50), Color32::from_rgb(66, 66, 72)) } else { (Color32::from_rgb(240, 240, 244), Color32::from_rgb(215, 215, 218)) }
    };

    let painter = ui.painter();
    painter.rect(rect, CornerRadius::same(5), bg, Stroke::new(1.0, stroke_col), StrokeKind::Inside);

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

/// Floating Quick Color HUD chip and palette popup
fn render_color_hud_chip(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    let fg = app.session.tools.foreground;
    let bg = app.session.tools.background;
    let fg_c32 = Color32::from_rgba_premultiplied(
        (fg[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (fg[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (fg[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        255,
    );
    let bg_c32 = Color32::from_rgba_premultiplied(
        (bg[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (bg[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (bg[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        255,
    );

    let (chip_rect, chip_resp) = ui.allocate_exact_size(vec2(44.0, 28.0), Sense::click());
    let chip_hovered = chip_resp.hovered();

    let (frame_bg, frame_stroke) = if chip_resp.is_pointer_button_down_on() {
        if t.dark() { (Color32::from_rgb(28, 28, 30), Color32::from_rgb(100, 100, 105)) } else { (Color32::from_rgb(205, 205, 208), Color32::from_rgb(160, 160, 160)) }
    } else if chip_hovered {
        if t.dark() { (Color32::from_rgb(58, 58, 62), Color32::from_rgb(115, 115, 120)) } else { (Color32::from_rgb(228, 228, 232), Color32::from_rgb(175, 175, 175)) }
    } else {
        if t.dark() { (Color32::from_rgb(46, 46, 50), Color32::from_rgb(66, 66, 72)) } else { (Color32::from_rgb(240, 240, 244), Color32::from_rgb(215, 215, 218)) }
    };

    let painter = ui.painter();
    painter.rect(chip_rect, CornerRadius::same(5), frame_bg, Stroke::new(1.0, frame_stroke), StrokeKind::Inside);

    // Background color square (drawn behind, at right)
    let bg_rect = Rect::from_min_size(pos2(chip_rect.left() + 16.0, chip_rect.top() + 8.0), vec2(14.0, 14.0));
    painter.rect(bg_rect, CornerRadius::same(2), bg_c32, Stroke::new(1.0, if t.dark() { Color32::from_rgb(90, 90, 95) } else { Color32::from_rgb(160, 160, 165) }), StrokeKind::Inside);

    // Foreground color square (drawn in front, at left)
    let fg_rect = Rect::from_min_size(pos2(chip_rect.left() + 6.0, chip_rect.top() + 5.0), vec2(15.0, 15.0));
    painter.rect(fg_rect, CornerRadius::same(2), fg_c32, Stroke::new(1.0, if t.dark() { Color32::from_rgb(110, 110, 115) } else { Color32::from_rgb(140, 140, 145) }), StrokeKind::Inside);

    // Tiny down arrow indicator on far right
    let chevron_pos = pos2(chip_rect.left() + 36.0, chip_rect.center().y);
    painter.line_segment([pos2(chevron_pos.x - 2.5, chevron_pos.y - 1.5), pos2(chevron_pos.x, chevron_pos.y + 1.5)], Stroke::new(1.2, t.text_dim));
    painter.line_segment([pos2(chevron_pos.x, chevron_pos.y + 1.5), pos2(chevron_pos.x + 2.5, chevron_pos.y - 1.5)], Stroke::new(1.2, t.text_dim));

    let chip_resp = chip_resp.on_hover_text(tl!("Quick Color HUD (Foreground / Background)\nClick for Palette & Swatches"));

    egui::Popup::menu(&chip_resp).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(290.0);

        ui.horizontal(|ui| {
            ui.heading(tl!("🎨 Quick Color HUD"));
        });
        ui.add_space(3.0);

        // Active Colors preview + Hex readouts
        let fg_hex = crate::color_picker_ui::hex([fg[0], fg[1], fg[2]]);
        let bg_hex = crate::color_picker_ui::hex([bg[0], bg[1], bg[2]]);

        ui.horizontal(|ui| {
            // Foreground row
            ui.label(egui::RichText::new("FG:").small().strong());
            let (f_rect, f_resp) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());
            ui.painter().rect(f_rect, CornerRadius::same(3), fg_c32, Stroke::new(1.0, t.field_border), StrokeKind::Inside);
            if f_resp.on_hover_text(tl!("Click to open Full Color Picker for Foreground")).clicked() {
                crate::color_picker_ui::open(app, "foreground");
                ui.close();
            }
            ui.monospace(egui::RichText::new(&fg_hex).small());

            ui.add_space(8.0);

            // Background row
            ui.label(egui::RichText::new("BG:").small().strong());
            let (b_rect, b_resp) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());
            ui.painter().rect(b_rect, CornerRadius::same(3), bg_c32, Stroke::new(1.0, t.field_border), StrokeKind::Inside);
            if b_resp.on_hover_text(tl!("Click to open Full Color Picker for Background")).clicked() {
                crate::color_picker_ui::open(app, "background");
                ui.close();
            }
            ui.monospace(egui::RichText::new(&bg_hex).small());
        });

        ui.add_space(4.0);

        // Quick Tools: Swap (X) and Default (D)
        ui.horizontal(|ui| {
            if ui.button(tl!("🔄 Swap (X)")).on_hover_text(tl!("Swap Foreground & Background Colors (X)")).clicked() {
                let _ = crate::menus::invoke(app, ui.ctx(), "tools.swapColors", json!({}));
            }
            if ui.button(tl!("⚪ Default B/W (D)")).on_hover_text(tl!("Default Black & White Colors (D)")).clicked() {
                let _ = crate::menus::invoke(app, ui.ctx(), "tools.defaultColors", json!({}));
            }
        });

        ui.add_space(6.0);
        ui.separator();
        ui.label(egui::RichText::new(tl!("ផ្ទាំងពណ៌រហ័ស (Designer Quick Palette):")).small().strong());
        ui.label(egui::RichText::new(tl!("Left-click: Set FG • Right-click: Set BG")).weak().small());

        const SWATCHES: &[(&str, [f32; 3])] = &[
            ("Pure White", [1.0, 1.0, 1.0]),
            ("Light Gray", [0.88, 0.88, 0.90]),
            ("Slate Gray", [0.58, 0.64, 0.72]),
            ("Dark Slate", [0.20, 0.25, 0.33]),
            ("Deep Black", [0.0, 0.0, 0.0]),

            ("Crimson Red", [0.94, 0.27, 0.27]),
            ("Sunset Orange", [0.98, 0.45, 0.09]),
            ("Amber Gold", [0.96, 0.62, 0.04]),
            ("Royal Gold", [0.92, 0.70, 0.03]),
            ("Lime Green", [0.52, 0.80, 0.09]),

            ("Emerald Green", [0.06, 0.73, 0.51]),
            ("Bright Cyan", [0.02, 0.71, 0.83]),
            ("Sky Blue", [0.05, 0.65, 0.91]),
            ("Royal Blue", [0.23, 0.51, 0.96]),
            ("Indigo", [0.39, 0.40, 0.95]),

            ("Violet", [0.55, 0.36, 0.96]),
            ("Purple", [0.66, 0.33, 0.97]),
            ("Fuchsia", [0.85, 0.27, 0.94]),
            ("Hot Pink", [0.93, 0.28, 0.60]),
            ("Bronze Brown", [0.52, 0.30, 0.05]),
        ];

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
            for &(name, rgb) in SWATCHES {
                let (sw_rect, sw_resp) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::click());
                let sw_c32 = Color32::from_rgb(
                    (rgb[0] * 255.0).round() as u8,
                    (rgb[1] * 255.0).round() as u8,
                    (rgb[2] * 255.0).round() as u8,
                );
                let is_hovered = sw_resp.hovered();
                let border_stroke = if is_hovered {
                    Stroke::new(1.5, Color32::WHITE)
                } else {
                    Stroke::new(1.0, if t.dark() { Color32::from_rgb(60, 60, 65) } else { Color32::from_rgb(180, 180, 185) })
                };

                let painter = ui.painter();
                painter.rect(sw_rect, CornerRadius::same(4), sw_c32, border_stroke, StrokeKind::Inside);

                let hex_str = crate::color_picker_ui::hex(rgb);
                let tooltip = format!("{}\n{}\nLeft-click: Set FG • Right-click: Set BG", name, hex_str);
                let sw_resp = sw_resp.on_hover_text(tooltip);

                if sw_resp.clicked() {
                    app.session.tools.foreground = [rgb[0], rgb[1], rgb[2], 1.0];
                } else if sw_resp.secondary_clicked() {
                    app.session.tools.background = [rgb[0], rgb[1], rgb[2], 1.0];
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();

        if ui.button(tl!("🎨 Open Full Color Picker…")).clicked() {
            crate::color_picker_ui::open(app, "foreground");
            ui.close();
        }
    });
}

/// Context when there is an active selection
fn render_selection_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    if task_button(ui, Some("lasso-select"), Some(tl!("Select and Mask…")), tl!("Refine selection edge (Cmd+Alt+R)"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "select.selectAndMask", json!({}));
    }

    if task_button(ui, Some("arrow-left-right"), Some(tl!("Invert")), tl!("Inverse Selection (Cmd+Shift+I)"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "select.inverse", json!({}));
    }

    if task_button(ui, Some("paint-bucket"), Some(tl!("Fill…")), tl!("Fill Selection (Shift+F5)"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "edit.fill", json!({}));
    }

    if task_button(ui, Some("layer-mask"), Some(tl!("Mask")), tl!("Create Layer Mask from Selection"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "layer.layerMask.revealSelection", json!({}));
    }

    // Quick adjustment popup
    let adj_btn = task_button(ui, Some("sliders-horizontal"), Some(tl!("Adjust Colors")), tl!("Add Adjustment Layer"), t);
    egui::Popup::menu(&adj_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(170.0);

        if ui.button(tl!("Curves…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.curves", json!({}));
            ui.close();
        }
        if ui.button(tl!("Levels…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.levels", json!({}));
            ui.close();
        }
        if ui.button(tl!("Brightness/Contrast…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.brightnessContrast", json!({}));
            ui.close();
        }
        if ui.button(tl!("Hue/Saturation…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.hueSaturation", json!({}));
            ui.close();
        }
        if ui.button(tl!("Black & White…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.blackWhite", json!({}));
            ui.close();
        }
    });

    separator(ui, t);

    render_color_hud_chip(app, ui, t);

    separator(ui, t);

    if task_button(ui, Some("scaling"), None, tl!("Transform Selection"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "select.transformSelection", json!({}));
    }

    if task_button(ui, Some("x"), None, tl!("Deselect (Cmd+D)"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "select.deselect", json!({}));
    }
}

/// Context when Type tool or a Text Layer is active
fn render_type_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    let _ = task_button(ui, Some("type"), Some(tl!("Text Layer")), "", t);

    separator(ui, t);

    if task_button(ui, Some("align-left"), None, tl!("Align Left"), t).clicked() {
        crate::type_tool::set_align(app, "left");
    }
    if task_button(ui, Some("align-center"), None, tl!("Align Center"), t).clicked() {
        crate::type_tool::set_align(app, "center");
    }
    if task_button(ui, Some("align-right"), None, tl!("Align Right"), t).clicked() {
        crate::type_tool::set_align(app, "right");
    }

    separator(ui, t);

    render_color_hud_chip(app, ui, t);

    separator(ui, t);

    if task_button(ui, Some("check"), None, tl!("Commit Type"), t).clicked() {
        crate::type_tool::commit(app);
    }
    if task_button(ui, Some("x"), None, tl!("Cancel Type"), t).clicked() {
        crate::type_tool::cancel(app);
    }
}

/// Context when Crop tool is active
fn render_crop_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    let _ = task_button(ui, Some("crop"), Some(tl!("Crop")), "", t);

    separator(ui, t);

    if task_button(ui, Some("ruler"), Some(tl!("Straighten")), tl!("Straighten image"), t).clicked() {
        crate::crop_straighten::toggle(app);
    }

    separator(ui, t);

    if task_button(ui, Some("check"), None, tl!("Commit Crop (Enter)"), t).clicked() {
        crate::canvas::commit_crop(app);
    }
    if task_button(ui, Some("x"), None, tl!("Cancel Crop (Esc)"), t).clicked() {
        crate::crop_ui::cancel(app);
    }
}

/// Context when Shape layer is active
fn render_shape_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    let _ = task_button(ui, Some("pentagon"), Some(tl!("Shape Layer")), "", t);

    separator(ui, t);

    render_color_hud_chip(app, ui, t);

    separator(ui, t);

    if task_button(ui, Some("square-dashed"), Some(tl!("To Selection")), tl!("Convert Shape to Selection"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "path.toSelection", json!({}));
    }
    if task_button(ui, Some("brush"), Some(tl!("Rasterize")), tl!("Rasterize Shape Layer"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "layer.rasterize.shape", json!({}));
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PosterComposerState {
    style: String,
    headline: String,
    subhead: String,
    badge: String,
    cta: String,
}

impl Default for PosterComposerState {
    fn default() -> Self {
        Self {
            style: "summer_fresh".into(),
            headline: "FRESH SUMMER".into(),
            subhead: "100% Pure • Refreshing Taste".into(),
            badge: "NEW ARRIVAL • 50% OFF".into(),
            cta: "BUY NOW".into(),
        }
    }
}

/// Default context when idle on an image/raster layer with no active selection (matches Photoshop 2026)
fn render_default_context(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    // 1. Select Subject (wand-sparkles icon + label)
    if task_button(ui, Some("wand-sparkles"), Some(tl!("Select Subject")), tl!("Automatically detect and select subject"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "select.subject", json!({}));
    }

    // 2. Remove Background (with AI / Standard / Specialized method selection)
    let current_method = app.session.prefs().tools.remove_background_method;
    let is_glass = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Glass;
    let is_tree = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Tree;
    let is_cloud = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Cloud;
    let is_plastic = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Plastic;
    let is_special = is_glass || is_tree || is_cloud || is_plastic;
    let is_ai = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Ai;
    let bg_label = if is_glass {
        format!("{} ({})", tl!("Remove Background"), tl!("Glass"))
    } else if is_tree {
        format!("{} ({})", tl!("Remove Background"), tl!("Tree"))
    } else if is_cloud {
        format!("{} ({})", tl!("Remove Background"), tl!("Cloud"))
    } else if is_plastic {
        format!("{} ({})", tl!("Remove Background"), tl!("Plastic"))
    } else if is_ai {
        format!("{} (AI)", tl!("Remove Background"))
    } else {
        tl!("Remove Background").to_string()
    };
    let bg_btn = task_button(
        ui,
        Some(if is_ai || is_special { "wand-sparkles" } else { "eraser-background" }),
        Some(&bg_label),
        tl!("Automatically isolate subject and mask out background"),
        t,
    );
    if bg_btn.clicked() {
        let method_str = match current_method {
            photocraft_engine::prefs::RemoveBackgroundMethod::Glass => "glass",
            photocraft_engine::prefs::RemoveBackgroundMethod::Tree => "tree",
            photocraft_engine::prefs::RemoveBackgroundMethod::Cloud => "cloud",
            photocraft_engine::prefs::RemoveBackgroundMethod::Plastic => "plastic",
            photocraft_engine::prefs::RemoveBackgroundMethod::Ai => "ai",
            photocraft_engine::prefs::RemoveBackgroundMethod::Heuristic => "heuristic",
        };
        let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": method_str, "glass": is_glass }));
    }

    let opt_btn = task_button(ui, Some("chevron-down"), None, tl!("Choose Background Removal Engine"), t);
    egui::Popup::menu(&opt_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(280.0);

        if ui.selectable_label(current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Heuristic, tl!("⚡ Standard (Fast & Built-in)")).clicked() {
            app.session.edit_prefs(|p| p.tools.remove_background_method = photocraft_engine::prefs::RemoveBackgroundMethod::Heuristic);
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "heuristic" }));
            ui.close();
        }

        if ui.selectable_label(is_ai, tl!("✨ AI Neural Model (Auto Detect)")).clicked() {
            app.session.edit_prefs(|p| p.tools.remove_background_method = photocraft_engine::prefs::RemoveBackgroundMethod::Ai);
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "ai" }));
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new(tl!("Photoshop Pro Native Cutouts (រូបមន្ត PS ផ្ទាល់ លឿនភ្លាមៗ < 0.1s):")).small().strong());

        if ui.selectable_label(is_glass, tl!("🍸 PS Glass / Transparent (ដបកែវ/បំពង់ថ្លា - ភ្លាមៗ)")).clicked() {
            app.session.edit_prefs(|p| p.tools.remove_background_method = photocraft_engine::prefs::RemoveBackgroundMethod::Glass);
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "glass", "glass": true, "ai": false }));
            ui.close();
        }

        if ui.selectable_label(is_tree, tl!("🌳 PS Tree / Foliage (ដើមឈើ/ស្លឹកឈើ - Channel Masking)")).clicked() {
            app.session.edit_prefs(|p| p.tools.remove_background_method = photocraft_engine::prefs::RemoveBackgroundMethod::Tree);
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "tree", "ai": false }));
            ui.close();
        }

        if ui.selectable_label(is_cloud, tl!("☁️ PS Cloud / Sky (ពពក/មេឃ/ផ្សែង - Blend If)")).clicked() {
            app.session.edit_prefs(|p| p.tools.remove_background_method = photocraft_engine::prefs::RemoveBackgroundMethod::Cloud);
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "cloud", "ai": false }));
            ui.close();
        }

        if ui.selectable_label(is_plastic, tl!("🛍️ PS Plastic Wrap (ថង់ប្លាស្ទិក - Multi-Pass)")).clicked() {
            app.session.edit_prefs(|p| p.tools.remove_background_method = photocraft_engine::prefs::RemoveBackgroundMethod::Plastic);
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "plastic", "ai": false }));
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new(tl!("Model folder: models/rmbg_model.onnx")).weak().small());
    });

    // 2.1 Photoshop Pro Cutout Tips button with "!"
    let tip_btn = task_button(ui, Some("info"), Some("! Tips"), tl!("Photoshop Pro Cutout Tips (គន្លឹះ និងជំហានកាត់ក្នុង PS)"), t);
    egui::Popup::menu(&tip_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_max_width(560.0);

        ui.horizontal(|ui| {
            ui.heading(tl!("💡 Photoshop Pro Cutout Tips & Steps"));
        });
        ui.label(egui::RichText::new(tl!("រៀនពីបច្ចេកទេសជំនាញក្នុង Photoshop និងចុចកាត់ស្វ័យប្រវត្តិតាមរូបមន្តនីមួយៗ")).small().weak());
        ui.separator();

        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            // Section 1: Glass & Transparent Objects
            ui.collapsing(tl!("🍸 1. ដបកែវ និងបំពង់ថ្លា (Glass & Transparent Objects)"), |ui| {
                ui.label(egui::RichText::new(tl!("• បញ្ហា: កញ្ចក់ថ្លាគ្មានពណ៌ផ្ទាល់ខ្លួនទេ (ឆ្លុះតុឈើ និងជញ្ជាំងចាស់)។ ការកាត់ធម្មតានឹងជាប់រូបតុឈើ និងសាឡុង ឬរលុបកែវខាងក្នុងចោល។")).small());
                ui.label(egui::RichText::new(tl!("• ជំហានក្នុង Photoshop (Commercial Glass Retouching):")).strong().small());
                ui.label(egui::RichText::new(tl!("   1. Pen Tool (P) កាត់រាងគ្រោងដបខាងក្រៅ (Silhouette Mask)។\n   2. បំបែកជា ២ ស្រទាប់ក្នុង Folder Group:\n      - Layer ក្រោម (Multiply): Desaturate (Ctrl+Shift+U) លុបពណ៌តុឈើត្នោតចោល រួចទាញកម្រិតឱ្យផ្ទៃកណ្តាលដបទៅជាពណ៌សសុទ្ធ (ដើម្បីឱ្យ Multiply Mode លុបរូបតុឈើចាស់ចោល ១០០% ក្លាយជាថ្លា) ដោយរក្សាទុកតែគែមដប និងបាតដប។\n      - Layer លើ (Screen): Desaturate និងប្រើ Levels (Ctrl+L) ទាញខ្មៅជិត ០ ទុកតែពន្លឺចាំងសៗ (Specular Highlights)។\n   3. ពេលដាក់លើ Background ថ្មី ដបកែវនឹងថ្លាឆ្លុះ Background នោះ ១០០% ដោយគ្មានជាប់ពណ៌តុឈើ ឬជញ្ជាំងចាស់ឡើយ។")).small());
                if ui.button(tl!("⚡ អនុវត្តកាត់ដបកែវភ្លាមៗ (PS Native Glass Cutout - Instant)")).clicked() {
                    let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "glass", "glass": true, "ai": false }));
                    ui.close();
                }
            });

            ui.separator();

            // Section 2: Trees & Foliage
            ui.collapsing(tl!("🌳 2. ដើមឈើ និងស្លឹកស្មុគស្មាញ (Trees & Foliage against Sky)"), |ui| {
                ui.label(egui::RichText::new(tl!("• បញ្ហា: ស្លឹកឈើតូចៗរាប់ពាន់ជាប់មេឃខៀវ/ស។ AI ទូទៅតែងតែរលុបស្លឹកតូចៗ ឬសល់រង្វង់ខៀវ (Color Fringing) ជុំវិញស្លឹក។")).small());
                ui.label(egui::RichText::new(tl!("• ជំហានក្នុង Photoshop (Channel Masking):")).strong().small());
                ui.label(egui::RichText::new(tl!("   1. បើក Channels Panel រកមើល Blue Channel (ព្រោះមេឃខៀវភ្លឺខ្លាំង ឯស្លឹកឈើងងឹតខ្មៅ Contrast ដាច់ខ្លាំង)។\n   2. Duplicate Blue Channel រួចចុច Ctrl+L (Levels) ទាញឱ្យស្លឹកខ្មៅដាច់ និងមេឃសសុទ្ធ។\n   3. ចុច Ctrl+I (Invert) ឱ្យដើមឈើស (Reveal) និងមេឃខ្មៅ (Hide)។\n   4. Ctrl+Click លើ Channel នោះដើម្បីយក Selection មកបង្កើត Layer Mask លើរូបដើម។\n   5. ប្រើ Layer > Matting > Defringe ដើម្បីលុបពណ៌ខៀវមេឃដែលសល់តាមគែមស្លឹក។")).small());
                if ui.button(tl!("⚡ អនុវត្តកាត់ដើមឈើភ្លាមៗ (PS Native Tree Cutout - Instant)")).clicked() {
                    let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "tree", "ai": false }));
                    ui.close();
                }
            });

            ui.separator();

            // Section 3: Clouds & Smoke
            ui.collapsing(tl!("☁️ 3. ពពក មេឃ និងផ្សែង (Clouds, Sky & Smoke)"), |ui| {
                ui.label(egui::RichText::new(tl!("• បញ្ហា: ពពកមានសភាពទន់ និងថ្លាពាក់កណ្តាល (Semi-transparent) រលាយចូលមេឃ មិនអាចកាត់ដោយគែមរឹងបានឡើយ។")).small());
                ui.label(egui::RichText::new(tl!("• ជំហានក្នុង Photoshop (Blend If):")).strong().small());
                ui.label(egui::RichText::new(tl!("   1. Double click លើ Layer ដើម្បីបើក Layer Style > Blending Options។\n   2. ត្រង់ Blend If ជ្រើសរើស Blue Channel។\n   3. ចុច Alt + Drag គ្រាប់ស្លាយខាងស្តាំ (Split Slider) មកឆ្វេង ដើម្បីឱ្យមេឃខៀវរលាយបាត់បន្តិចម្តងៗ ដោយសល់ដុំពពកយ៉ាងរលូន។")).small());
                if ui.button(tl!("⚡ អនុវត្តកាត់ពពកភ្លាមៗ (PS Native Cloud Cutout - Instant)")).clicked() {
                    let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "cloud", "ai": false }));
                    ui.close();
                }
            });

            ui.separator();

            // Section 4: Plastic Bags
            ui.collapsing(tl!("🛍️ 4. ថង់ប្លាស្ទិកថ្លា (Plastic Bags & Wrap)"), |ui| {
                ui.label(egui::RichText::new(tl!("• បញ្ហា: ថង់ប្លាស្ទិកថ្លាមានផ្នត់ជ្រួញ និងពន្លឺជះ ព្រមទាំងបង្ហាញរបស់នៅខាងក្នុង។")).small());
                ui.label(egui::RichText::new(tl!("• ជំហានក្នុង Photoshop:")).strong().small());
                ui.label(egui::RichText::new(tl!("   1. Mask រាងគ្រោងថង់ខាងក្រៅដោយ Pen Tool។\n   2. Layer ខាងក្រោមដាក់ Blend Mode = Multiply សម្រាប់ស្រមោល និងរបស់ក្នុងថង់។\n   3. Layer ខាងលើប្រើ Filter > Other > High Pass រួចដាក់ Blend Mode = Screen សម្រាប់ពន្លឺចាំងផ្នត់ជ្រួញ។")).small());
                if ui.button(tl!("⚡ អនុវត្តកាត់ថង់ប្លាស្ទិកភ្លាមៗ (PS Native Plastic Cutout - Instant)")).clicked() {
                    let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "plastic", "ai": false }));
                    ui.close();
                }
            });
        });
    });

    // 2.2 Smart Poster Composer button
    let poster_btn = task_button(ui, Some("sparkles"), Some(tl!("🎨 Smart Poster")), tl!("Smart Poster Composer (រៀបចំ Poster ស្វ័យប្រវត្តពី Layers របស់អ្នក)"), t);
    egui::Popup::menu(&poster_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(380.0);

        ui.horizontal(|ui| {
            let (icon_r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
            ui.painter().circle_filled(icon_r.center(), 11.0, Color32::from_rgb(0, 120, 240));
            crate::icons::paint(ui, icon_r, "sparkles", 14.0, Color32::WHITE);
            ui.heading(tl!("🎨 Smart Poster Composer"));
        });
        ui.label(egui::RichText::new(tl!("រៀបចំ Poster ស្វ័យប្រវត្តពី Layers របស់អ្នក (Auto Composition & Typography)")).small().weak());
        ui.separator();

        let state_id = egui::Id::new("smart_poster_composer_state");
        let mut state = ui.data_mut(|d| d.get_temp_mut_or_default::<PosterComposerState>(state_id).clone());

        ui.label(egui::RichText::new(tl!("រចនាបទ Poster (Visual Styles):")).strong().small());

        // Visual Style Card Selector
        let styles: [(&str, &str, &str, Color32, &str, &str); 5] = [
            ("summer_fresh", "🌴 Summer Fresh", "Tropical", Color32::from_rgb(6, 182, 212), "FRESH SUMMER", "100% Pure • Refreshing Taste"),
            ("luxury_gold", "👑 Luxury Gold", "Premium", Color32::from_rgb(234, 179, 8), "PURE LUXURY", "Exclusive Edition • Premium Craftsmanship"),
            ("cyberpunk", "⚡ Cyberpunk", "Neon", Color32::from_rgb(236, 72, 153), "CYBER VIBES", "Next-Gen Experience • Limited Drop"),
            ("super_sale", "🛍️ Super Sale", "50% Off", Color32::from_rgb(239, 68, 68), "SUPER SALE", "Up to 50% Off • Limited Time Offer"),
            ("minimal_clean", "🌿 Minimal Clean", "Studio", Color32::from_rgb(16, 185, 129), "MINIMAL ART", "Simplicity & Elegance in Every Detail"),
        ];

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for (id, label, tag, color, def_h, def_s) in styles {
                let is_selected = state.style == id;
                let (card_r, card_resp) = ui.allocate_exact_size(vec2(178.0, 44.0), Sense::click());
                let hovered = card_resp.hovered();

                let bg = if is_selected {
                    color.gamma_multiply(0.18)
                } else if hovered {
                    Color32::from_rgba_premultiplied(255, 255, 255, 15)
                } else {
                    Color32::from_rgba_premultiplied(255, 255, 255, 8)
                };

                let stroke_col = if is_selected {
                    color
                } else if hovered {
                    Color32::from_rgba_premultiplied(255, 255, 255, 60)
                } else {
                    Color32::from_rgba_premultiplied(255, 255, 255, 20)
                };

                let painter = ui.painter();
                painter.rect_filled(card_r, CornerRadius::same(6), bg);
                painter.rect_stroke(card_r, CornerRadius::same(6), Stroke::new(if is_selected { 1.5 } else { 1.0 }, stroke_col), StrokeKind::Inside);

                // Left accent vertical bar
                let bar = Rect::from_min_max(pos2(card_r.left() + 2.0, card_r.top() + 4.0), pos2(card_r.left() + 6.0, card_r.bottom() - 4.0));
                painter.rect_filled(bar, CornerRadius::same(2), color);

                // Title
                let title_galley = painter.layout_no_wrap(label.to_string(), theme::medium(12.0), if is_selected { Color32::WHITE } else { t.text });
                painter.galley(pos2(card_r.left() + 12.0, card_r.top() + 6.0), title_galley, Color32::WHITE);

                // Tag badge
                let tag_galley = painter.layout_no_wrap(tag.to_string(), egui::FontId::proportional(10.0), color);
                let tag_w = tag_galley.size().x + 8.0;
                let tag_rect = Rect::from_min_size(pos2(card_r.left() + 12.0, card_r.top() + 23.0), vec2(tag_w, 15.0));
                painter.rect_filled(tag_rect, CornerRadius::same(3), color.gamma_multiply(0.2));
                painter.galley(pos2(tag_rect.left() + 4.0, tag_rect.top() + 1.0), tag_galley, color);

                if card_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    state.style = id.to_string();
                    state.headline = def_h.to_string();
                    state.subhead = def_s.to_string();
                }
            }
        });

        ui.add_space(8.0);
        ui.label(egui::RichText::new(tl!("ចំណងជើងធំ (Main Headline):")).small());
        ui.add(egui::TextEdit::singleline(&mut state.headline).desired_width(365.0));

        ui.add_space(4.0);
        ui.label(egui::RichText::new(tl!("ចំណងជើងរង & ពាក្យស្លោក (Sub-Headline):")).small());
        ui.add(egui::TextEdit::singleline(&mut state.subhead).desired_width(365.0));

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(tl!("Badge (បញ្ចុះតម្លៃ):")).small());
                ui.add(egui::TextEdit::singleline(&mut state.badge).desired_width(175.0));
            });
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(tl!("CTA (ប៊ូតុងសកម្មភាព):")).small());
                ui.add(egui::TextEdit::singleline(&mut state.cta).desired_width(182.0));
            });
        });

        ui.add_space(12.0);
        ui.separator();
        let (btn_r, btn_resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
        let btn_hovered = btn_resp.hovered();
        let btn_bg = if btn_hovered { Color32::from_rgb(28, 130, 248) } else { Color32::from_rgb(0, 110, 235) };
        let painter = ui.painter();
        painter.rect_filled(btn_r, CornerRadius::same(6), btn_bg);
        painter.rect_stroke(btn_r, CornerRadius::same(6), Stroke::new(1.0, Color32::from_white_alpha(50)), StrokeKind::Inside);
        let btn_text = tl!("🚀 Compose Poster Now (បង្កើត Poster ភ្លាមៗ)");
        let btn_g = painter.layout_no_wrap(btn_text.to_string(), theme::semibold(13.0), Color32::WHITE);
        painter.galley(pos2(btn_r.center().x - btn_g.size().x / 2.0, btn_r.center().y - btn_g.size().y / 2.0), btn_g, Color32::WHITE);

        if btn_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            let _ = crate::menus::invoke(
                app,
                ui.ctx(),
                "doc.composePoster",
                json!({
                    "style": state.style,
                    "headline": state.headline,
                    "subhead": state.subhead,
                    "badge": state.badge,
                    "cta": state.cta,
                }),
            );
            ui.close();
        }

        ui.data_mut(|d| d.insert_temp(state_id, state));
    });

    // 3. Adjust Colors (sliders-horizontal icon + label)
    let adj_btn = task_button(ui, Some("sliders-horizontal"), Some(tl!("Adjust Colors")), tl!("Add Adjustment Layer"), t);
    egui::Popup::menu(&adj_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(170.0);

        if ui.button(tl!("Curves…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.curves", json!({}));
            ui.close();
        }
        if ui.button(tl!("Levels…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.levels", json!({}));
            ui.close();
        }
        if ui.button(tl!("Brightness/Contrast…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.brightnessContrast", json!({}));
            ui.close();
        }
        if ui.button(tl!("Hue/Saturation…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.hueSaturation", json!({}));
            ui.close();
        }
        if ui.button(tl!("Black & White…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.blackWhite", json!({}));
            ui.close();
        }
    });

    separator(ui, t);

    render_color_hud_chip(app, ui, t);

    separator(ui, t);

    // 4. Free Transform icon button
    if task_button(ui, Some("scaling"), None, tl!("Free Transform (Cmd+T)"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "edit.freeTransform", json!({}));
    }

    // 5. Adjustment Layer icon button
    let adj_icon_btn = task_button(ui, Some("adjustment-layer"), None, tl!("New Adjustment Layer…"), t);
    egui::Popup::menu(&adj_icon_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(170.0);

        if ui.button(tl!("Curves…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.curves", json!({}));
            ui.close();
        }
        if ui.button(tl!("Levels…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.levels", json!({}));
            ui.close();
        }
        if ui.button(tl!("Brightness/Contrast…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.brightnessContrast", json!({}));
            ui.close();
        }
        if ui.button(tl!("Hue/Saturation…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.hueSaturation", json!({}));
            ui.close();
        }
        if ui.button(tl!("Black & White…")).clicked() {
            let _ = crate::menus::invoke(app, ui.ctx(), "layer.newAdjustmentLayer.blackWhite", json!({}));
            ui.close();
        }
    });

    // 6. Place Embedded / Add image icon button
    if task_button(ui, Some("image"), None, tl!("Place Embedded…"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "file.placeEmbedded", json!({}));
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

    #[test]
    fn type_commit_and_cancel_commands_succeed() {
        let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.session.execute("file.new", serde_json::json!({"width": 200, "height": 100})).unwrap();
        app.sync_views();
        assert!(app.run("type.commit", serde_json::json!({})).is_ok());
        assert!(app.run("type.cancel", serde_json::json!({})).is_ok());
        assert!(app.run("type.align.left", serde_json::json!({})).is_ok());
        assert!(app.run("type.align.center", serde_json::json!({})).is_ok());
        assert!(app.run("type.align.right", serde_json::json!({})).is_ok());
        assert!(app.run("image.crop.commit", serde_json::json!({})).is_ok());
        assert!(app.run("image.crop.cancel", serde_json::json!({})).is_ok());
        assert!(app.run("image.crop.straighten", serde_json::json!({})).is_ok());
    }

    #[test]
    fn contextual_task_bar_commands_exist() {
        let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.session.execute("file.new", serde_json::json!({"width": 200, "height": 100})).unwrap();
        app.sync_views();
        // Default context commands
        assert!(app.session.is_enabled("select.subject"));
        assert!(app.session.is_enabled("layer.removeBackground"));
        assert!(app.session.is_enabled("layer.newAdjustmentLayer.curves"));
        assert!(app.session.is_enabled("layer.newAdjustmentLayer.levels"));
        assert!(app.session.is_enabled("layer.newAdjustmentLayer.brightnessContrast"));
        assert!(app.session.is_enabled("layer.newAdjustmentLayer.hueSaturation"));
        assert!(app.session.is_enabled("layer.newAdjustmentLayer.blackWhite"));
        // Selection context commands
        assert!(photocraft_engine::commands::find("select.inverse").is_some());
        assert!(photocraft_engine::commands::find("edit.fill").is_some());
        assert!(photocraft_engine::commands::find("layer.layerMask.revealSelection").is_some());
        assert!(photocraft_engine::commands::find("select.deselect").is_some());
        assert!(photocraft_engine::commands::find("select.transformSelection").is_some());
        // Shape context commands
        assert!(photocraft_engine::commands::find("path.toSelection").is_some());
        assert!(photocraft_engine::commands::find("layer.rasterize.shape").is_some());
        // Color HUD commands
        assert!(photocraft_engine::commands::find("tools.swapColors").is_some());
        assert!(photocraft_engine::commands::find("tools.defaultColors").is_some());
    }

    #[test]
    fn color_hud_tools_swap_and_default() {
        let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        assert!(app.run("tools.defaultColors", serde_json::json!({})).is_ok());
        assert_eq!(app.session.tools.foreground, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(app.session.tools.background, [1.0, 1.0, 1.0, 1.0]);
        assert!(app.run("tools.swapColors", serde_json::json!({})).is_ok());
        assert_eq!(app.session.tools.foreground, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(app.session.tools.background, [0.0, 0.0, 0.0, 1.0]);
    }
}
