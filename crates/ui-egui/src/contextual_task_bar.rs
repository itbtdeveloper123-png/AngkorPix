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

    // Dynamic width tracking for exact horizontal centering
    let width_id = egui::Id::unique("contextual_bar_measured_width");
    let measured_width: f32 = ctx.data(|d| d.get_temp(width_id)).unwrap_or(470.0);

    let center_x = (content_rect.center().x - measured_width / 2.0)
        .clamp(content_rect.left() + 10.0, (content_rect.right() - measured_width - 10.0).max(content_rect.left() + 10.0));
    let bottom_y = (content_rect.bottom() - 48.0)
        .clamp(content_rect.top() + 10.0, (content_rect.bottom() - 44.0).max(content_rect.top() + 10.0));
    let default_pos = pos2(center_x, bottom_y);

    let initial_pos = if app.ui.contextual_task_bar_pinned {
        if let Some([x, y]) = app.ui.contextual_task_bar_pos {
            pos2(
                x.clamp(content_rect.left() + 10.0, (content_rect.right() - measured_width - 10.0).max(content_rect.left() + 10.0)),
                y.clamp(content_rect.top() + 10.0, (content_rect.bottom() - 44.0).max(content_rect.top() + 10.0)),
            )
        } else {
            default_pos
        }
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

    let area_resp = egui::Area::new(egui::Id::unique("contextual-task-bar-area"))
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
                        app.ui.contextual_task_bar_pinned = true;
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
                        ui.set_min_width(220.0);

                        ui.menu_button(tl!("⚡ 1-Click Pro Actions"), |ui| {
                            crate::widgets::style_spectrum_popup_menu(ui);
                            render_one_click_tools_popup_content(app, ui, &t);
                        });

                        ui.menu_button(tl!("🎨 Smart Poster Composer"), |ui| {
                            crate::widgets::style_spectrum_popup_menu(ui);
                            render_poster_composer_content(app, ui, &t);
                        });

                        ui.menu_button(tl!("💡 Pro Cutout Tips"), |ui| {
                            crate::widgets::style_spectrum_popup_menu(ui);
                            render_cutout_tips_content(app, ui);
                        });

                        ui.menu_button(tl!("Background Removal Engine"), |ui| {
                            crate::widgets::style_spectrum_popup_menu(ui);
                            render_bg_engine_menu_content(app, ui);
                        });

                        ui.separator();

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

    let bar_rect = area_resp.response.rect;
    if bar_rect.width() > 10.0 {
        ctx.data_mut(|d| d.insert_temp(width_id, bar_rect.width()));
    }

    if let Some(pos) = new_pos {
        app.ui.contextual_task_bar_pos = Some(pos);
    }
    if reset_pos {
        app.ui.contextual_task_bar_pos = None;
        app.ui.contextual_task_bar_pinned = false;
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

    // Feather selection popup
    let feather_btn = task_button(ui, None, Some(tl!("Feather…")), tl!("Feather Selection Edge"), t);
    egui::Popup::menu(&feather_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(120.0);
        for px in [1, 2, 5, 10, 20, 30, 50] {
            if ui.button(format!("{px} px")).clicked() {
                let _ = crate::menus::invoke(app, ui.ctx(), "select.modify.feather", json!({"radius": px}));
                ui.close();
            }
        }
    });

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

    // Font size selector popup
    let cur_size = app.ui.tool_options.type_size.round() as i32;
    let size_label = format!("{cur_size} pt");
    let size_btn = task_button(ui, None, Some(&size_label), tl!("Font Size"), t);
    egui::Popup::menu(&size_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(95.0);
        for sz in [10, 12, 14, 18, 24, 30, 36, 48, 60, 72, 96] {
            if ui.button(format!("{sz} pt")).clicked() {
                crate::type_tool::set_size(app, sz as f32);
                ui.close();
            }
        }
    });

    // Faux Bold & Italic quick buttons
    if task_button(ui, None, Some("B"), tl!("Toggle Faux Bold"), t).clicked() {
        crate::type_tool::toggle_bold(app);
    }
    if task_button(ui, None, Some("I"), tl!("Toggle Faux Italic"), t).clicked() {
        crate::type_tool::toggle_italic(app);
    }

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

    if task_button(ui, None, Some(tl!("Warp…")), tl!("Warp Text (Arc, Wave, Flag)"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "layer.type.warp", json!({}));
    }

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

    // Aspect ratio preset popup
    let ratio_str = match app.ui.tool_options.crop_ratio.as_str() {
        "" => tl!("Ratio"),
        "original" => tl!("Original"),
        "1:1" => "1:1",
        "4:5" => "4:5",
        "16:9" => "16:9",
        "4:3" => "4:3",
        "9:16" => "9:16",
        other => other,
    };
    let ratio_btn = task_button(ui, None, Some(ratio_str), tl!("Crop Aspect Ratio"), t);
    egui::Popup::menu(&ratio_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(140.0);
        if ui.button(tl!("Unconstrained")).clicked() {
            app.ui.tool_options.crop_ratio.clear();
            ui.close();
        }
        if ui.button(tl!("Original Ratio")).clicked() {
            app.ui.tool_options.crop_ratio = "original".into();
            ui.close();
        }
        if ui.button("1:1 (Square)").clicked() {
            app.ui.tool_options.crop_ratio = "1:1".into();
            ui.close();
        }
        if ui.button("4:5 (8:10)").clicked() {
            app.ui.tool_options.crop_ratio = "4:5".into();
            ui.close();
        }
        if ui.button("16:9 (Landscape)").clicked() {
            app.ui.tool_options.crop_ratio = "16:9".into();
            ui.close();
        }
        if ui.button("9:16 (Story / Reel)").clicked() {
            app.ui.tool_options.crop_ratio = "9:16".into();
            ui.close();
        }
        if ui.button("4:3").clicked() {
            app.ui.tool_options.crop_ratio = "4:3".into();
            ui.close();
        }
    });

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

    // Stroke width popup
    let stroke_btn = task_button(ui, None, Some(tl!("Stroke…")), tl!("Shape Stroke Width"), t);
    egui::Popup::menu(&stroke_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(110.0);
        for w in [1, 2, 3, 5, 8, 12, 20] {
            if ui.button(format!("{w} px")).clicked() {
                let _ = crate::menus::invoke(app, ui.ctx(), "shape.edit", json!({"strokeWidth": w}));
                ui.close();
            }
        }
    });

    // Corner radius popup
    let radius_btn = task_button(ui, None, Some(tl!("Radius…")), tl!("Corner Radius (Rectangles/Polygons)"), t);
    egui::Popup::menu(&radius_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(110.0);
        for r in [0, 4, 8, 12, 16, 24, 32, 48] {
            if ui.button(format!("{r} px")).clicked() {
                let _ = crate::menus::invoke(app, ui.ctx(), "shape.edit", json!({"cornerRadius": r}));
                ui.close();
            }
        }
    });

    if task_button(ui, Some("sparkles"), Some(tl!("Fx…")), tl!("Open Layer Styles"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "layer.layerStyle", json!({}));
    }

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

    // 2. Remove Background (clean icon + label)
    let current_method = app.session.prefs().tools.remove_background_method;
    let is_api = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Api;
    let is_glass = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Glass;
    let is_ai = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Ai;
    let bg_btn = task_button(
        ui,
        Some(if is_ai || is_glass || is_api { "wand-sparkles" } else { "eraser-background" }),
        Some(tl!("Remove Background")),
        if is_api {
            tl!("Remove Background (remove.bg Cloud API with 3-key failover)")
        } else {
            tl!("Automatically isolate subject and mask out background")
        },
        t,
    );
    if bg_btn.clicked() {
        let method_str = match current_method {
            photocraft_engine::prefs::RemoveBackgroundMethod::Api => "api",
            photocraft_engine::prefs::RemoveBackgroundMethod::Glass => "glass",
            photocraft_engine::prefs::RemoveBackgroundMethod::Tree => "tree",
            photocraft_engine::prefs::RemoveBackgroundMethod::Cloud => "cloud",
            photocraft_engine::prefs::RemoveBackgroundMethod::Plastic => "plastic",
            photocraft_engine::prefs::RemoveBackgroundMethod::Ai => "ai",
            photocraft_engine::prefs::RemoveBackgroundMethod::Heuristic => "heuristic",
        };
        let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": method_str, "glass": is_glass }));
    }

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

    // 4. Free Transform (Cmd+T) - icon only
    if task_button(ui, Some("scaling"), None, tl!("Free Transform (Cmd+T)"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "edit.freeTransform", json!({}));
    }

    // 5. New Adjustment Layer - icon only (half filled circle)
    let adj_icon_btn = task_button(ui, Some("adjustment-layer"), None, tl!("New Adjustment Layer…"), t);
    egui::Popup::menu(&adj_icon_btn).show(|ui| {
        crate::widgets::style_spectrum_popup_menu(ui);
        ui.set_min_width(180.0);

        let adjs = [
            ("layer.newAdjustmentLayer.brightnessContrast", tl!("Brightness/Contrast…")),
            ("layer.newAdjustmentLayer.levels", tl!("Levels…")),
            ("layer.newAdjustmentLayer.curves", tl!("Curves…")),
            ("layer.newAdjustmentLayer.exposure", tl!("Exposure…")),
            ("layer.newAdjustmentLayer.vibrance", tl!("Vibrance…")),
            ("layer.newAdjustmentLayer.hueSaturation", tl!("Hue/Saturation…")),
            ("layer.newAdjustmentLayer.colorBalance", tl!("Color Balance…")),
            ("layer.newAdjustmentLayer.blackWhite", tl!("Black & White…")),
            ("layer.newAdjustmentLayer.photoFilter", tl!("Photo Filter…")),
            ("layer.newAdjustmentLayer.channelMixer", tl!("Channel Mixer…")),
            ("layer.newAdjustmentLayer.colorLookup", tl!("Color Lookup…")),
            ("layer.newAdjustmentLayer.invert", tl!("Invert")),
            ("layer.newAdjustmentLayer.posterize", tl!("Posterize…")),
            ("layer.newAdjustmentLayer.threshold", tl!("Threshold…")),
            ("layer.newAdjustmentLayer.gradientMap", tl!("Gradient Map…")),
            ("layer.newAdjustmentLayer.selectiveColor", tl!("Selective Color…")),
        ];
        for (id, label) in adjs {
            if ui.button(label).clicked() {
                let _ = crate::menus::invoke(app, ui.ctx(), id, json!({}));
                ui.close();
            }
        }
    });

    // 6. Place Embedded / Add image - icon only
    if task_button(ui, Some("image"), None, tl!("Place Embedded…"), t).clicked() {
        let _ = crate::menus::invoke(app, ui.ctx(), "file.placeEmbedded", json!({}));
    }

    separator(ui, t);
}

fn render_bg_engine_menu_content(app: &mut PhotocraftApp, ui: &mut Ui) {
    ui.set_min_width(290.0);
    let current_method = app.session.prefs().tools.remove_background_method;
    let is_api = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Api;
    let is_glass = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Glass;
    let is_tree = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Tree;
    let is_cloud = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Cloud;
    let is_plastic = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Plastic;
    let is_ai = current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Ai;

    if ui.selectable_label(is_api, tl!("🌐 Cloud HD (remove.bg 3-Key Pool)")).clicked() {
        app.session.edit_prefs(|p| p.tools.remove_background_method = photocraft_engine::prefs::RemoveBackgroundMethod::Api);
        let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "api" }));
        ui.close();
    }

    if ui.selectable_label(current_method == photocraft_engine::prefs::RemoveBackgroundMethod::Heuristic, tl!("⚡ Standard (Fast & Built-in)")).clicked() {
        app.session.edit_prefs(|p| p.tools.remove_background_method = photocraft_engine::prefs::RemoveBackgroundMethod::Heuristic);
        let _ = crate::menus::invoke(app, ui.ctx(), "layer.removeBackground", json!({ "method": "heuristic" }));
        ui.close();
    }

    if ui.selectable_label(is_ai, tl!("✨ Local AI Model (Auto Detect)")).clicked() {
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
    ui.label(egui::RichText::new(tl!("Keys: 3 remove.bg active (Auto Failover)")).weak().small());
}

fn render_cutout_tips_content(app: &mut PhotocraftApp, ui: &mut Ui) {
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
}

fn render_poster_composer_content(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    ui.set_min_width(380.0);

    ui.horizontal(|ui| {
        let (icon_r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
        ui.painter().circle_filled(icon_r.center(), 11.0, Color32::from_rgb(0, 120, 240));
        crate::icons::paint(ui, icon_r, "sparkles", 14.0, Color32::WHITE);
        ui.heading(tl!("🎨 Smart Poster Composer"));
    });
    ui.label(egui::RichText::new(tl!("រៀបចំ Poster ស្វ័យប្រវត្តពី Layers របស់អ្នក (Auto Composition & Typography)")).small().weak());
    ui.separator();

    let state_id = egui::Id::unique("smart_poster_composer_state");
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
}

fn render_one_click_tools_popup_content(app: &mut PhotocraftApp, ui: &mut Ui, t: &Tokens) {
    ui.set_min_width(380.0);

    ui.horizontal(|ui| {
        let (icon_r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
        ui.painter().circle_filled(icon_r.center(), 11.0, Color32::from_rgb(245, 158, 11));
        crate::icons::paint(ui, icon_r, "sparkles", 14.0, Color32::WHITE);
        ui.heading(tl!("⚡ 1-Click Pro Quick Actions"));
    });
    ui.label(egui::RichText::new(tl!("ឧបករណ៍ជំនាញ ចុចមួយភ្លែតបានផលភ្លាមៗ ដោយមិនបាច់សារ៉េអីច្រើន")).small().weak());
    ui.separator();

    let actions: [(&str, &str, &str, &str, Color32); 8] = [
        ("quickAction.autoEnhance", "⚡ Auto Enhance", "កែពន្លឺ ពណ៌ និង Contrast ស្វ័យប្រវត្តិ", "Auto Tone + Contrast + Color", Color32::from_rgb(59, 130, 246)),
        ("layer.removeBackground", "✂️ Remove Background", "កាត់ផ្ទៃក្រោយភ្លាមៗ (< 0.1s)", "Auto subject detection & mask", Color32::from_rgb(16, 185, 129)),
        ("quickAction.blurBackground", "🌫️ Blur Background", "ព្រិលផ្ទៃក្រោយ Portrait Depth / Bokeh", "Sharp subject + creamy bokeh blur", Color32::from_rgb(139, 92, 246)),
        ("quickAction.blackWhiteBackground", "🎭 B&W Background", "ផ្ទៃក្រោយ ស-ខ្មៅ រំលេចមនុស្ស (Color Splash)", "Vivid subject + dramatic B&W backdrop", Color32::from_rgb(236, 72, 153)),
        ("quickAction.goldenHour", "🌅 Golden Hour Glow", "ពន្លឺថ្ងៃលិច ស្រទន់កក់ក្តៅ", "Warm amber glow & sunlight grade", Color32::from_rgb(245, 158, 11)),
        ("quickAction.cinematicTealOrange", "🎬 Cinematic Grade", "ពណ៌កុន Teal & Orange Blockbuster", "Hollywood split-toning movie grade", Color32::from_rgb(6, 182, 212)),
        ("quickAction.popColors", "✨ Pop Colors & Vibrance", "ពណ៌រស់រវើក HDR Pop", "Dynamic punch for social media", Color32::from_rgb(234, 179, 8)),
        ("quickAction.studioProductWhite", "🛍️ Studio Product White", "ផ្ទៃក្រោយសស្ទូឌីយោ + ស្រមោលជាន់ដី", "Pure white e-commerce + contact shadow", Color32::from_rgb(99, 102, 241)),
    ];

    egui::ScrollArea::vertical().max_height(460.0).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 5.0);
        for (cmd, title, khmer_desc, en_desc, accent) in actions {
            let (card_r, card_resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
            let hovered = card_resp.hovered();

            let bg = if hovered {
                accent.gamma_multiply(0.18)
            } else {
                Color32::from_rgba_premultiplied(255, 255, 255, if t.dark() { 8 } else { 200 })
            };

            let stroke_col = if hovered {
                accent
            } else {
                Color32::from_rgba_premultiplied(255, 255, 255, if t.dark() { 24 } else { 180 })
            };

            let painter = ui.painter();
            painter.rect_filled(card_r, CornerRadius::same(6), bg);
            painter.rect_stroke(card_r, CornerRadius::same(6), Stroke::new(if hovered { 1.5 } else { 1.0 }, stroke_col), StrokeKind::Inside);

            // Left accent vertical indicator
            let bar = Rect::from_min_max(pos2(card_r.left() + 2.0, card_r.top() + 4.0), pos2(card_r.left() + 6.0, card_r.bottom() - 4.0));
            painter.rect_filled(bar, CornerRadius::same(2), accent);

            // Title
            let title_galley = painter.layout_no_wrap(title.to_string(), theme::medium(12.5), if hovered { Color32::WHITE } else { t.text });
            painter.galley(pos2(card_r.left() + 14.0, card_r.top() + 6.0), title_galley, Color32::WHITE);

            // Description (Khmer + English)
            let desc_text = format!("{} • {}", khmer_desc, en_desc);
            let desc_galley = painter.layout_no_wrap(desc_text, egui::FontId::proportional(10.5), if hovered { accent } else { t.text_dim });
            painter.galley(pos2(card_r.left() + 14.0, card_r.top() + 24.0), desc_galley, t.text_dim);

            if card_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                let _ = crate::menus::invoke(app, ui.ctx(), cmd, json!({}));
                ui.close();
            }
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

    #[test]
    fn type_commit_and_cancel_commands_succeed() {
        let mut app = crate::PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.session.execute("file.new", serde_json::json!({"width": 200, "height": 100})).unwrap();
        app.sync_views();
        let _ = app.run("type.create", serde_json::json!({"text": "Hello", "size": 24, "x": 10, "y": 20}));
        let _ = crate::type_tool::edit_active(&mut app);
        crate::type_tool::commit(&mut app);
        let _ = crate::type_tool::edit_active(&mut app);
        crate::type_tool::cancel(&mut app);
        crate::type_tool::set_align(&mut app, "left");
        crate::type_tool::set_align(&mut app, "center");
        crate::type_tool::set_align(&mut app, "right");
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
        // 1-Click Pro Quick Action commands
        assert!(app.session.is_enabled("quickAction.autoEnhance"));
        assert!(app.session.is_enabled("quickAction.blurBackground"));
        assert!(app.session.is_enabled("quickAction.blackWhiteBackground"));
        assert!(app.session.is_enabled("quickAction.goldenHour"));
        assert!(app.session.is_enabled("quickAction.cinematicTealOrange"));
        assert!(app.session.is_enabled("quickAction.popColors"));
        assert!(app.session.is_enabled("quickAction.studioProductWhite"));
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
