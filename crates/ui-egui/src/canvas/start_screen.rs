use super::tabs::open_on_double_click;
use super::{Color32, PhotocraftApp, Pos2, Rect, Sense, Stroke, paint_dots, pos2, vec2};
use egui::CornerRadius;

/// Recent files listed on the Home screen.
const HOME_RECENT: usize = 12;

/// Paint a dashed rounded rectangle matching Adobe Photoshop's drop box.
fn paint_dashed_rect(painter: &egui::Painter, rect: Rect, radius: f32, stroke: Stroke) {
    let dash = 8.0;
    let gap = 5.0;

    // Top
    let mut x = rect.left() + radius;
    while x < rect.right() - radius {
        let x2 = (x + dash).min(rect.right() - radius);
        painter.line_segment([pos2(x, rect.top()), pos2(x2, rect.top())], stroke);
        x += dash + gap;
    }
    // Bottom
    let mut x = rect.left() + radius;
    while x < rect.right() - radius {
        let x2 = (x + dash).min(rect.right() - radius);
        painter.line_segment([pos2(x, rect.bottom()), pos2(x2, rect.bottom())], stroke);
        x += dash + gap;
    }
    // Left
    let mut y = rect.top() + radius;
    while y < rect.bottom() - radius {
        let y2 = (y + dash).min(rect.bottom() - radius);
        painter.line_segment([pos2(rect.left(), y), pos2(rect.left(), y2)], stroke);
        y += dash + gap;
    }
    // Right
    let mut y = rect.top() + radius;
    while y < rect.bottom() - radius {
        let y2 = (y + dash).min(rect.bottom() - radius);
        painter.line_segment([pos2(rect.right(), y), pos2(rect.right(), y2)], stroke);
        y += dash + gap;
    }
    // Four corner arcs
    painter.line_segment([pos2(rect.left() + radius, rect.top()), pos2(rect.left(), rect.top() + radius)], stroke);
    painter.line_segment([pos2(rect.right() - radius, rect.top()), pos2(rect.right(), rect.top() + radius)], stroke);
    painter.line_segment([pos2(rect.left() + radius, rect.bottom()), pos2(rect.left(), rect.bottom() - radius)], stroke);
    painter.line_segment([pos2(rect.right() - radius, rect.bottom()), pos2(rect.right(), rect.bottom() - radius)], stroke);
}

/// Paint the Photoshop-style image stack icon with a down arrow.
fn paint_image_stack_icon(painter: &egui::Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    let bg = Color32::from_rgb(30, 30, 30);
    let fw = 44.0;
    let fh = 30.0;

    // Back card (tilted/offset top-right)
    let back = Rect::from_center_size(center + vec2(4.0, -6.0), vec2(fw, fh));
    painter.rect_stroke(back, CornerRadius::same(3), stroke, egui::StrokeKind::Inside);

    // Front card (foreground, covers back card with fill)
    let front = Rect::from_center_size(center + vec2(-4.0, 2.0), vec2(fw, fh));
    painter.rect_filled(front, CornerRadius::same(3), bg);
    painter.rect_stroke(front, CornerRadius::same(3), stroke, egui::StrokeKind::Inside);

    // Sun / Circle in front frame
    painter.circle_stroke(front.left_top() + vec2(9.0, 8.0), 2.5, stroke);

    // Mountains in front frame
    let m1 = [
        front.left_bottom() + vec2(4.0, -3.0),
        front.left_bottom() + vec2(15.0, -14.0),
        front.left_bottom() + vec2(25.0, -3.0),
    ];
    painter.line_segment([m1[0], m1[1]], stroke);
    painter.line_segment([m1[1], m1[2]], stroke);

    let m2 = [
        front.left_bottom() + vec2(21.0, -7.0),
        front.left_bottom() + vec2(30.0, -16.0),
        front.left_bottom() + vec2(39.0, -3.0),
    ];
    painter.line_segment([m2[0], m2[1]], stroke);
    painter.line_segment([m2[1], m2[2]], stroke);

    // Downward arrow centered pointing into the frames
    let ax = center.x - 2.0;
    let ay_top = center.y - 20.0;
    let ay_bot = center.y + 10.0;
    let arrow_stroke = Stroke::new(2.0, color);
    painter.line_segment([pos2(ax, ay_top), pos2(ax, ay_bot)], arrow_stroke);
    painter.line_segment([pos2(ax - 5.0, ay_bot - 6.0), pos2(ax, ay_bot)], arrow_stroke);
    painter.line_segment([pos2(ax + 5.0, ay_bot - 6.0), pos2(ax, ay_bot)], arrow_stroke);
}

pub(super) fn start_screen(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let _t = crate::theme::Tokens::get(ui.ctx());
    let area = ui.available_rect_before_wrap();
    paint_dots(ui, area, app.session.prefs().interface.show_canvas_dots);
    open_on_double_click(app, ui, area);

    // File › Open Recent, newest first
    let show_recent = app.session.prefs().interface.show_home_recent_files;
    let recent: Vec<String> = if cfg!(target_arch = "wasm32") || !show_recent {
        Vec::new()
    } else {
        app.ui.recent_files.iter().take(HOME_RECENT).cloned().collect()
    };

    let new_label = crate::shortcuts::command_label(app, "New document…", "file.new");
    let open_label = crate::shortcuts::command_label(app, "Open…", "file.open");

    // Responsive sidebar width: standard 190.0, slightly scaled down on very narrow windows
    let sidebar_w = if area.width() < 500.0 {
        (area.width() * 0.42).clamp(130.0, 180.0)
    } else {
        190.0
    };

    let sidebar_rect = Rect::from_min_max(area.left_top(), pos2(area.left() + sidebar_w, area.bottom()));
    let main_rect = Rect::from_min_max(pos2(sidebar_rect.right(), area.top()), area.right_bottom());

    // Paint sidebar background & subtle right border
    let sidebar_bg = Color32::from_rgb(25, 25, 25);
    ui.painter().rect_filled(sidebar_rect, CornerRadius::ZERO, sidebar_bg);
    ui.painter().line_segment(
        [pos2(sidebar_rect.right(), sidebar_rect.top()), pos2(sidebar_rect.right(), sidebar_rect.bottom())],
        Stroke::new(1.0, Color32::from_rgb(42, 42, 42)),
    );

    // Paint main area background
    let main_bg = Color32::from_rgb(30, 30, 30);
    ui.painter().rect_filled(main_rect, CornerRadius::ZERO, main_bg);

    // -------------------------------------------------------------------------
    // 1. LEFT SIDEBAR (Photoshop-style navigation)
    // -------------------------------------------------------------------------
    ui.scope_builder(egui::UiBuilder::new().max_rect(sidebar_rect), |ui| {
        ui.add_space(20.0);
        let btn_w = (sidebar_w - 32.0).clamp(90.0, 140.0);
        let pad_x = ((sidebar_w - btn_w) / 2.0).max(10.0);

        // "New file" (Signature Photoshop Blue Pill Button)
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            let (r, resp) = ui.allocate_exact_size(vec2(btn_w, 32.0), Sense::click());
            let pill_color = if resp.is_pointer_button_down_on() {
                Color32::from_rgb(15, 95, 195)
            } else if resp.hovered() {
                Color32::from_rgb(38, 128, 235)
            } else {
                Color32::from_rgb(20, 115, 230)
            };
            ui.painter().rect_filled(r, CornerRadius::same(16), pill_color);
            let text_g = ui.painter().layout_no_wrap(new_label.clone(), crate::theme::semibold(13.0), Color32::WHITE);
            let text_pos = pos2(r.center().x - text_g.size().x / 2.0, r.center().y - text_g.size().y / 2.0);
            ui.painter().galley(text_pos, text_g, Color32::WHITE);
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &new_label));
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                let fields = app.new_document_fields();
                app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
            }
        });

        ui.add_space(10.0);

        // "Open" Button
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            let (r, resp) = ui.allocate_exact_size(vec2(btn_w, 30.0), Sense::click());
            let hover = resp.hovered();
            if hover {
                ui.painter().rect_filled(r, CornerRadius::same(15), Color32::from_white_alpha(15));
            }
            let text_col = if hover { Color32::WHITE } else { Color32::from_rgb(225, 225, 225) };
            let text_g = ui.painter().layout_no_wrap(open_label.clone(), egui::FontId::proportional(13.0), text_col);
            let text_pos = pos2(r.center().x - text_g.size().x / 2.0, r.center().y - text_g.size().y / 2.0);
            ui.painter().galley(text_pos, text_g, text_col);
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &open_label));
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                let _ = app.open_dialog_file();
            }
        });

        ui.add_space(26.0);

        // "Home" Navigation Tab (Active indicator)
        let (home_r, home_resp) = ui.allocate_exact_size(vec2(sidebar_w, 34.0), Sense::click());
        let home_hover = home_resp.hovered();
        let home_bg = if home_hover { Color32::from_white_alpha(18) } else { Color32::from_white_alpha(10) };
        ui.painter().rect_filled(home_r, CornerRadius::ZERO, home_bg);

        // Active indicator bar on left edge
        let bar_r = Rect::from_min_size(home_r.left_top() + vec2(0.0, 5.0), vec2(3.0, 24.0));
        ui.painter().rect_filled(bar_r, CornerRadius::ZERO, Color32::from_rgb(20, 115, 230));

        // House Icon
        let icon_r = Rect::from_center_size(pos2(home_r.left() + 24.0, home_r.center().y), vec2(16.0, 16.0));
        crate::icons::paint(ui, icon_r, "house", 15.0, Color32::WHITE);

        // Home Label
        let home_text = tl!("Home");
        let home_g = ui.painter().layout_no_wrap(home_text.to_string(), crate::theme::semibold(13.5), Color32::WHITE);
        ui.painter().galley(pos2(home_r.left() + 42.0, home_r.center().y - home_g.size().y / 2.0), home_g, Color32::WHITE);

        // Bottom footer links in sidebar
        let bottom_y = sidebar_rect.bottom() - 60.0;
        if ui.cursor().top() < bottom_y {
            ui.add_space((bottom_y - ui.cursor().top()).max(0.0));
            crate::links::discord_button(app, ui, btn_w);
            ui.add_space(6.0);
            crate::links::link_row(app, ui);
        }
    });

    // -------------------------------------------------------------------------
    // 2. MAIN AREA (Photoshop-style dashed drop box & recent files)
    // -------------------------------------------------------------------------
    ui.scope_builder(egui::UiBuilder::new().max_rect(main_rect), |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let avail_w = ui.available_width();
            let box_w = (avail_w - 64.0).clamp(280.0, 1100.0);
            let pad_x = ((avail_w - box_w) / 2.0).max(24.0);
            let box_h = if recent.is_empty() { 230.0 } else { 180.0 };

            if recent.is_empty() {
                let avail_h = ui.available_height();
                let top_pad = ((avail_h - box_h) * 0.32).clamp(36.0, 200.0);
                ui.add_space(top_pad);
            } else {
                ui.add_space(28.0);
            }

            // Dashed Drop Box
            ui.horizontal(|ui| {
                ui.add_space(pad_x);
                let (box_r, box_resp) = ui.allocate_exact_size(vec2(box_w, box_h), Sense::click());

                // Detect file drag-over or mouse hover
                let has_drag = ui.ctx().input(|i| !i.raw.hovered_files.is_empty()) || box_resp.hovered();
                let stroke_color = if has_drag {
                    Color32::from_rgb(20, 115, 230)
                } else {
                    Color32::from_rgb(68, 68, 68)
                };
                let stroke = Stroke::new(if has_drag { 1.5 } else { 1.0 }, stroke_color);

                if has_drag {
                    ui.painter().rect_filled(box_r, CornerRadius::same(8), Color32::from_rgba_unmultiplied(20, 115, 230, 14));
                }
                paint_dashed_rect(ui.painter(), box_r, 8.0, stroke);

                // Photoshop Image Stack Icon
                let icon_center = pos2(box_r.center().x, box_r.top() + 50.0);
                paint_image_stack_icon(
                    ui.painter(),
                    icon_center,
                    if has_drag { Color32::from_rgb(20, 115, 230) } else { Color32::from_rgb(145, 145, 145) },
                );

                // "Drag and drop a file" Title
                let title_text = tl!("Drag and drop a file");
                let title_g = ui.painter().layout_no_wrap(title_text.to_string(), crate::theme::semibold(17.5), Color32::from_rgb(240, 240, 240));
                let title_pos = pos2(box_r.center().x - title_g.size().x / 2.0, box_r.top() + 96.0);
                ui.painter().galley(title_pos, title_g, Color32::from_rgb(240, 240, 240));

                // Subtitle: "Or select one from your computer to get started."
                let prefix = tl!("Or ");
                let select_one = tl!("select one");
                let suffix = tl!(" from your computer to get started.");
                let sub_font = egui::FontId::proportional(13.0);
                let g_prefix = ui.painter().layout_no_wrap(prefix.to_string(), sub_font.clone(), Color32::from_rgb(150, 150, 150));
                let g_select = ui.painter().layout_no_wrap(select_one.to_string(), sub_font.clone(), Color32::from_rgb(59, 142, 246));
                let g_suffix = ui.painter().layout_no_wrap(suffix.to_string(), sub_font, Color32::from_rgb(150, 150, 150));

                let total_w = g_prefix.size().x + g_select.size().x + g_suffix.size().x;
                let mut start_x = box_r.center().x - total_w / 2.0;
                let sub_y = box_r.top() + 126.0;

                ui.painter().galley(pos2(start_x, sub_y), g_prefix.clone(), Color32::from_rgb(150, 150, 150));
                start_x += g_prefix.size().x;

                let link_rect = Rect::from_min_size(pos2(start_x, sub_y), g_select.size());
                let link_resp = ui.interact(link_rect, ui.scope_id().with("select_one_link"), Sense::click());
                let link_hover = link_resp.hovered();
                let link_color = if link_hover { Color32::from_rgb(96, 165, 250) } else { Color32::from_rgb(59, 142, 246) };
                ui.painter().galley(pos2(start_x, sub_y), g_select.clone(), link_color);
                if link_hover {
                    ui.painter().line_segment(
                        [pos2(link_rect.left(), link_rect.bottom()), pos2(link_rect.right(), link_rect.bottom())],
                        Stroke::new(1.0, link_color),
                    );
                }
                let link_clicked = link_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
                if link_clicked {
                    let _ = app.open_dialog_file();
                }

                start_x += g_select.size().x;
                ui.painter().galley(pos2(start_x, sub_y), g_suffix, Color32::from_rgb(150, 150, 150));

                // Click anywhere on dashed box to open file dialog
                if box_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !link_clicked {
                    let _ = app.open_dialog_file();
                }
            });

            // Recent Files Section
            if !recent.is_empty() {
                ui.add_space(28.0);
                home_recent(app, ui, &recent, pad_x, box_w);
            }
            ui.add_space(32.0);
        });
    });
}

/// Wayland has no native file drops (winit 0.30, #386), so the hint offers File › Open and paste.
#[cfg(test)]
pub(super) fn start_screen_drop_hint(app: &PhotocraftApp) -> std::borrow::Cow<'static, str> {
    if app.services.is_wayland {
        crate::i18n::fmt(tl!("Use File › Open, or paste a copied image with {paste}."), &[("paste", &crate::notices::paste_hint(app))]).into()
    } else {
        tl!("Drop an image or PSD anywhere to open it.").into()
    }
}

/// The Home screen's "Recent" list: file name, its folder on the right, click to open.
fn home_recent(app: &mut PhotocraftApp, ui: &mut egui::Ui, recent: &[String], pad_x: f32, width: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.add_space(pad_x + 4.0);
        ui.label(egui::RichText::new(tl!("Recent")).font(crate::theme::semibold(16.0)).color(Color32::from_rgb(240, 240, 240)));
    });
    ui.add_space(10.0);

    let mut open = None;
    let mut remove = None;
    let mut clear = false;

    for path in recent {
        let name = crate::file_open::display_name(path);
        let folder = std::path::Path::new(path).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();

        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            let (row, resp) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
            let hovered = ui.rect_contains_pointer(row);
            if hovered {
                ui.painter().rect_filled(row, CornerRadius::same(6), Color32::from_white_alpha(15));
            }

            let icon_r = Rect::from_center_size(pos2(row.left() + 20.0, row.center().y), vec2(16.0, 16.0));
            crate::icons::paint(ui, icon_r, "file", 16.0, if hovered { t.text } else { t.text_dim });

            let close = Rect::from_center_size(pos2(row.right() - 20.0, row.center().y), vec2(22.0, 22.0));
            let right = if hovered { close.left() - 6.0 } else { row.right() - 14.0 };

            let x = row.left() + 40.0;
            let name_max = (width * 0.42).clamp(260.0, 520.0);
            let name_g = crate::tab_strip::elided(ui, &name, egui::FontId::proportional(13.0), Color32::from_rgb(235, 235, 235), name_max);
            let name_w = name_g.size().x;
            ui.painter().galley(pos2(x, row.center().y - name_g.size().y / 2.0), name_g, Color32::from_rgb(235, 235, 235));

            let folder_max = (right - (x + name_w + 20.0)).max(0.0);
            if folder_max > 40.0 && !folder.is_empty() {
                let fg = crate::tab_strip::elided(ui, &folder, egui::FontId::proportional(12.0), Color32::from_rgb(130, 130, 130), folder_max);
                ui.painter().galley(pos2(right - fg.size().x, row.center().y - fg.size().y / 2.0), fg, Color32::from_rgb(130, 130, 130));
            }

            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &name));
            let resp = resp.on_hover_text(path).on_hover_cursor(egui::CursorIcon::PointingHand);

            if hovered {
                let x_btn = ui.interact(close, resp.id.with("remove"), Sense::click());
                let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &name)]);
                x_btn.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
                let color = crate::icons::button_chrome(ui, close, false, x_btn.hovered());
                crate::icons::paint(ui, close, "x", 12.0, color);
                if x_btn.on_hover_text(tl!("Remove from Recent")).clicked() {
                    remove = Some(path.clone());
                }
            }

            if resp.clicked() && remove.is_none() {
                open = Some(path.clone());
            }

            resp.context_menu(|ui| {
                if ui.button(tl!("Remove from Recent")).clicked() {
                    remove = Some(path.clone());
                    ui.close();
                }
                if ui.button(tl!("Clear Recent Files")).clicked() {
                    clear = true;
                    ui.close();
                }
            });
        });
        ui.add_space(2.0);
    }

    if clear {
        app.clear_recent();
    } else if let Some(path) = remove {
        app.remove_recent(&path);
    }
    if let Some(path) = open
        && let Err(e) = app.open_path(&path)
    {
        app.open_failed(&crate::file_open::display_name(&path), &e);
    }
}
