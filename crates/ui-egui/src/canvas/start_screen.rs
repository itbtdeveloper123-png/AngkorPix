use super::tabs::{open_on_double_click, paint_dots};
use super::{PhotocraftApp, Rect, Sense};

pub(super) fn start_screen(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let area = ui.available_rect_before_wrap();
    paint_dots(ui, area, app.session.prefs().interface.show_canvas_dots);
    open_on_double_click(app, ui, area);

    // File › Open Recent, newest first (on the web there are no paths to reopen).
    let show_recent = app.session.prefs().interface.show_home_recent_files;
    let recent: Vec<String> =
        if cfg!(target_arch = "wasm32") || !show_recent { Vec::new() } else { app.ui.recent_files.iter().take(HOME_RECENT).cloned().collect() };

    let is_narrow = area.width() < 640.0;
    let pad_x = if is_narrow { 16.0 } else { 36.0 };
    let pad_y = if is_narrow { 24.0 } else { 54.0 };
    let rail_w = 170.0;

    if is_narrow {
        let card_w = (area.width() - 32.0).clamp(0.0, 520.0);
        let card = Rect::from_center_size(area.center(), egui::vec2(card_w, (area.height() - 40.0).max(0.0)));
        ui.scope_builder(egui::UiBuilder::new().max_rect(card), |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                render_start_actions_row(app, ui, card.width());
                ui.add_space(16.0);
                render_start_content(app, ui, &recent, card.width());
            });
        });
    } else {
        let rail_rect = Rect::from_min_size(egui::pos2(area.left() + pad_x, area.top() + pad_y), egui::vec2(rail_w, area.height() - pad_y * 2.0));
        let content_x = rail_rect.right() + 36.0;
        let content_rect = Rect::from_min_max(egui::pos2(content_x, area.top() + pad_y), egui::pos2(area.right() - pad_x, area.bottom() - 16.0));

        // 1. Left Navigation Rail
        ui.scope_builder(egui::UiBuilder::new().max_rect(rail_rect), |ui| {
            render_start_rail(app, ui, rail_rect.width());
        });

        // 2. Right Content Area (scrollable)
        ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                render_start_content(app, ui, &recent, content_rect.width());
            });
        });
    }
}

fn render_start_actions_row(app: &mut PhotocraftApp, ui: &mut egui::Ui, avail_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let btn_w = ((avail_w - 12.0) / 2.0).clamp(0.0, 150.0);

    ui.horizontal(|ui| {
        // [ New file ]
        let new_label = tl!("New file");
        let (new_rect, new_resp) = ui.allocate_exact_size(egui::vec2(btn_w, 32.0), Sense::click());
        new_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, new_label));
        let new_resp = new_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
        let new_bg = if new_resp.hovered() { egui::Color32::from_rgb(41, 131, 245) } else { egui::Color32::from_rgb(26, 115, 232) };
        ui.painter().rect_filled(new_rect, 4.0, new_bg);
        ui.painter().text(new_rect.center(), egui::Align2::CENTER_CENTER, new_label, crate::theme::semibold(13.0), egui::Color32::WHITE);
        if new_resp.clicked() {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        }

        ui.add_space(8.0);

        // [ Open… ]
        let open_label = tl!("Open…");
        let (open_rect, open_resp) = ui.allocate_exact_size(egui::vec2(btn_w, 32.0), Sense::click());
        open_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, open_label));
        let open_resp = open_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
        if open_resp.hovered() {
            ui.painter().rect_filled(open_rect, 4.0, t.hover);
        }
        ui.painter().rect_stroke(open_rect, 4.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
        ui.painter().text(open_rect.center(), egui::Align2::CENTER_CENTER, open_label, crate::theme::semibold(13.0), t.text);
        if open_resp.clicked() {
            let _ = app.open_dialog_file();
        }
    });
}

fn render_start_rail(app: &mut PhotocraftApp, ui: &mut egui::Ui, rail_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());

    ui.add_space(8.0);

    // 1. [ New file ] button
    let new_label = tl!("New file");
    let (new_rect, new_resp) = ui.allocate_exact_size(egui::vec2(rail_w.min(130.0), 32.0), Sense::click());
    new_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, new_label));
    let new_resp = new_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let new_bg = if new_resp.hovered() { egui::Color32::from_rgb(41, 131, 245) } else { egui::Color32::from_rgb(26, 115, 232) };
    ui.painter().rect_filled(new_rect, 4.0, new_bg);
    ui.painter().text(new_rect.center(), egui::Align2::CENTER_CENTER, new_label, crate::theme::semibold(13.0), egui::Color32::WHITE);
    if new_resp.clicked() {
        let fields = app.new_document_fields();
        app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
    }

    ui.add_space(10.0);

    // 2. [ Open… ] button
    let open_label = tl!("Open…");
    let (open_rect, open_resp) = ui.allocate_exact_size(egui::vec2(rail_w.min(130.0), 32.0), Sense::click());
    open_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, open_label));
    let open_resp = open_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if open_resp.hovered() {
        ui.painter().rect_filled(open_rect, 4.0, t.hover);
    }
    ui.painter().rect_stroke(open_rect, 4.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    ui.painter().text(open_rect.center(), egui::Align2::CENTER_CENTER, open_label, crate::theme::semibold(13.0), t.text);
    if open_resp.clicked() {
        let _ = app.open_dialog_file();
    }

    ui.add_space(24.0);

    // 3. Navigation items: Home
    let (home_rect, home_resp) = ui.allocate_exact_size(egui::vec2(rail_w, 28.0), Sense::hover());
    ui.painter().rect_filled(home_rect, 4.0, t.hover);
    let icon_box = Rect::from_center_size(egui::pos2(home_rect.left() + 20.0, home_rect.center().y), egui::vec2(14.0, 14.0));
    crate::icons::paint(ui, icon_box, "house", 14.0, t.text);
    ui.painter().text(egui::pos2(home_rect.left() + 34.0, home_rect.center().y), egui::Align2::LEFT_CENTER, tl!("Home"), crate::theme::semibold(12.0), t.text);
    let _ = home_resp;
}

fn render_start_content(app: &mut PhotocraftApp, ui: &mut egui::Ui, recent: &[String], content_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    ui.add_space(8.0);

    // Top Row: [App Icon] AngkorPix
    ui.horizontal(|ui| {
        let logo_size = egui::vec2(26.0, 26.0);
        let (logo_rect, _) = ui.allocate_exact_size(logo_size, Sense::hover());
        crate::brand::paint_mark(ui, logo_rect);

        ui.add_space(10.0);

        let app_name = crate::menus::native_menu::APP_NAME;
        let title_galley = ui.painter().layout_no_wrap(app_name.into(), crate::theme::semibold(20.0), t.text);
        let (title_r, _) = ui.allocate_exact_size(title_galley.size(), Sense::hover());
        ui.painter().galley(title_r.min, title_galley, t.text);
    });

    ui.add_space(18.0);

    // Hairline separator under header
    let (sep_r, _) = ui.allocate_exact_size(egui::vec2(content_w, 1.0), Sense::hover());
    ui.painter().line_segment([sep_r.min, egui::pos2(sep_r.max.x, sep_r.min.y)], egui::Stroke::new(1.0, t.separator));

    ui.add_space(24.0);

    if recent.is_empty() {
        render_first_run_content(app, ui, content_w);
    } else {
        crate::home_recent_ui::show_home_recent(app, ui, recent, content_w);
    }

    ui.add_space(24.0);
}

fn render_first_run_content(app: &mut PhotocraftApp, ui: &mut egui::Ui, content_w: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());

    let drop_w = 540.0f32.min(content_w - 24.0);
    let drop_h = 200.0;

    // Center vertically in available content area
    let avail_h = ui.available_height();
    let top_space = ((avail_h - drop_h) / 2.0 - 50.0).max(40.0);
    ui.add_space(top_space);

    // Center horizontally across content_w
    let pad_x = ((content_w - drop_w) / 2.0).max(0.0);
    ui.horizontal(|ui| {
        ui.add_space(pad_x);

        let (drop_r, drop_resp) = ui.allocate_exact_size(egui::vec2(drop_w, drop_h), Sense::click());
        let drop_resp = drop_resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tl!("Open…"));
        let is_hov = drop_resp.hovered();

        let border_color = if is_hov { t.text_dim } else { t.separator };
        let bg_color = if is_hov { t.hover } else { egui::Color32::TRANSPARENT };
        ui.painter().rect_filled(drop_r, 8.0, bg_color);
        ui.painter().rect_stroke(drop_r, 8.0, egui::Stroke::new(1.0, border_color), egui::StrokeKind::Inside);

        // Centered icon & message
        let icon_box = Rect::from_center_size(egui::pos2(drop_r.center().x, drop_r.center().y - 18.0), egui::vec2(40.0, 40.0));
        crate::icons::paint(ui, icon_box, "image", 34.0, if is_hov { t.text } else { t.text_dim });

        let drop_msg = start_screen_drop_hint(app);
        ui.painter().text(
            egui::pos2(drop_r.center().x, drop_r.center().y + 20.0),
            egui::Align2::CENTER_CENTER,
            drop_msg,
            crate::theme::medium(14.0),
            if is_hov { t.text } else { t.text_faint },
        );

        if drop_resp.clicked() {
            let _ = app.open_dialog_file();
        }
    });
}

/// Wayland has no native file drops (winit 0.30, #386), so the hint offers File › Open and paste.
pub(crate) fn start_screen_drop_hint(app: &PhotocraftApp) -> std::borrow::Cow<'static, str> {
    if app.services.is_wayland {
        crate::i18n::fmt(tl!("Use File › Open, or paste a copied image with {paste}."), &[("paste", &crate::notices::paste_hint(app))]).into()
    } else {
        tl!("Drop an image or PSD anywhere to open it.").into()
    }
}

/// Recent files listed on the Home screen.
const HOME_RECENT: usize = 11;
