use egui::{Painter, Stroke};
use shared::Data;

use crate::{math::world_to_screen, ui::app::AppState};

impl AppState {
    pub fn draw_nade_prediction(&self, painter: &Painter, data: &Data) {
        let config = &self.config.hud.nade_prediction;
        let prediction = &data.nade_prediction;
        if !config.enabled || prediction.path.len() < 2 {
            return;
        }

        let stroke = Stroke::new(self.config.hud.line_width, config.line_color);
        for window in prediction.path.windows(2) {
            let (Some(a), Some(b)) = (
                world_to_screen(&window[0], data),
                world_to_screen(&window[1], data),
            ) else {
                continue;
            };
            painter.line_segment([a, b], stroke);
        }

        for bounce in &prediction.bounces {
            if let Some(point) = world_to_screen(bounce, data) {
                painter.circle_filled(point, 3.0, config.bounce_color);
            }
        }

        if let Some(end) = prediction.end
            && let Some(point) = world_to_screen(&end, data)
        {
            painter.circle_stroke(point, 6.0, Stroke::new(2.0, config.end_color));
            painter.circle_filled(point, 2.0, config.end_color);
        }
    }
}
