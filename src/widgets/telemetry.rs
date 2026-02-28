use eframe::egui;
use egui_plot::{Line, Plot, PlotBounds, PlotPoints, Points};
use std::collections::VecDeque;

pub fn telemetry_graph(
    ui: &mut egui::Ui,
    gas_history: &VecDeque<(f64, f64, bool)>,
    brake_history: &VecDeque<(f64, f64, bool)>,
    time_window: f64,
    t: f64,
    color_gas: egui::Color32,
    color_brake: egui::Color32,
) {
    Plot::new("telemetry_plot")
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .show_axes([false, false])
        .show_grid([false, false])
        .include_y(0.0)
        .include_y(1.0)
        .height(120.0)
        .width(450.0)
        .set_margin_fraction(egui::vec2(0.0, 0.05))
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds(PlotBounds::from_min_max([t - time_window, 0.0], [t, 1.0]));

            let gas_line: PlotPoints = gas_history
                .iter()
                .step_by(5)
                .map(|(t, v, _)| [*t, *v])
                .collect();

            let brake_line: PlotPoints = brake_history
                .iter()
                .step_by(5)
                .map(|(t, v, _)| [*t, *v])
                .collect();

            // FIX: Added name string as first argument
            plot_ui.line(Line::new("Gas", gas_line).color(color_gas).width(4.0));
            plot_ui.line(Line::new("Brake", brake_line).color(color_brake).width(4.0));

            let gas_tc_points: PlotPoints = gas_history
                .iter()
                .filter(|(_, _, tc)| *tc)
                .map(|(t, v, _)| [*t, *v])
                .collect();

            // FIX: Added name string
            plot_ui.points(
                Points::new("Gas TC", gas_tc_points)
                    .color(egui::Color32::YELLOW)
                    .radius(1.5),
            );

            let brake_abs_points: PlotPoints = brake_history
                .iter()
                .filter(|(_, _, abs)| *abs)
                .map(|(t, v, _)| [*t, *v])
                .collect();

            // FIX: Added name string
            plot_ui.points(
                Points::new("Brake ABS", brake_abs_points)
                    .color(egui::Color32::YELLOW)
                    .radius(1.5),
            );
        });
}

// ... (Rest of the file remains exactly the same as before)

pub fn pedal_bar(ui: &mut egui::Ui, value: f32, color: egui::Color32, in_action: bool, text: &str) {
    let bar_height = 120.0;
    let bar_width = 30.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());
    let painter = ui.painter();

    // Background
    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));

    // Fill
    if value > 0.0 {
        let fill_height = rect.height() * value;
        let fill_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - fill_height),
            egui::pos2(rect.right(), rect.bottom()),
        );

        let fill_color = if in_action {
            egui::Color32::from_rgb(255, 123, 0)
        } else {
            color
        };

        painter.rect_filled(fill_rect, 0.0, fill_color);
    }

    // Text
    let text_pos = egui::pos2(rect.center().x, rect.top() + 5.0);

    painter.text(
        text_pos,
        egui::Align2::CENTER_TOP,
        text,
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );
}
