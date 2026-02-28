use eframe::egui;

pub fn rev_strip(
    ui: &mut egui::Ui,
    percent: f32,
    is_yellow: bool,
    is_strobe: bool,
    text: &str,
    ignition: bool,
) {
    let bar_height = 30.0;
    let bar_width = 415.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));

    let color = if is_strobe {
        let time = ui.input(|i| i.time);
        // Flash between Red and Yellow (Strobe Effect)
        if (time * 20.0).sin() > 0.0 {
            egui::Color32::RED
        } else {
            egui::Color32::from_rgb(255, 255, 0)
        }
    } else if is_yellow {
        egui::Color32::from_rgb(255, 255, 0) // Yellow
    } else {
        egui::Color32::WHITE
    };

    let font_color = if percent > 0.5 {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    };

    if percent > 0.0 {
        let fill_width = rect.width() * percent;
        let fill_rect = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.left() + fill_width, rect.bottom()),
        );
        painter.rect_filled(fill_rect, 0.0, color);
    } else if !ignition {
        painter.rect_filled(rect, 0.0, egui::Color32::RED);
    }
    else {
        painter.rect_filled(rect, 0.0, egui::Color32::DARK_GREEN);
    }

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(16.0),
        font_color,
    );
}

pub fn gear_indicator(ui: &mut egui::Ui, gear_text: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 90.0), egui::Sense::hover());
    let painter = ui.painter();

    painter.text(
        rect.center() + egui::vec2(0.0, 12.0),
        egui::Align2::CENTER_CENTER,
        gear_text,
        egui::FontId::proportional(100.0),
        egui::Color32::WHITE,
    );
}

pub fn speedometer(ui: &mut egui::Ui, speed_text: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(90.0, 50.0), egui::Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(70));

    painter.text(
        rect.center() + egui::vec2(0.0, 3.0),
        egui::Align2::CENTER_CENTER,
        speed_text,
        egui::FontId::proportional(30.0),
        egui::Color32::WHITE,
    );
}

pub fn stat_box(ui: &mut egui::Ui, heading: &str, value: &str) {
    let width = 73.0;
    let height = 50.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter();

    let header_height = 20.0;
    let split_y = rect.top() + header_height;

    painter.rect_filled(
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), split_y)),
        0.0,
        egui::Color32::from_gray(50),
    );

    painter.rect_filled(
        egui::Rect::from_min_max(egui::pos2(rect.left(), split_y), rect.max),
        0.0,
        egui::Color32::from_gray(70),
    );

    painter.text(
        egui::pos2(rect.center().x, rect.top() + 10.0 + 1.5),
        egui::Align2::CENTER_CENTER,
        heading.to_uppercase(),
        egui::FontId::proportional(12.0),
        egui::Color32::LIGHT_GRAY,
    );

    painter.text(
        egui::pos2(rect.center().x, split_y + 15.0 + 1.5),
        egui::Align2::CENTER_CENTER,
        value,
        egui::FontId::proportional(18.0),
        egui::Color32::WHITE,
    );
}
