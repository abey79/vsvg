use egui::util::History;
use egui::Label;

pub(crate) struct FrameHistory {
    frame_times: History<f32>,
}

impl Default for FrameHistory {
    fn default() -> Self {
        let max_age: f32 = 1.0;
        let max_len = (max_age * 300.0).round() as usize;
        Self {
            frame_times: History::new(0..max_len, max_age),
        }
    }
}

impl FrameHistory {
    // Called first
    pub fn on_new_frame(&mut self, now: f64, previous_frame_time: Option<f32>) {
        let previous_frame_time = previous_frame_time.unwrap_or_default();
        if let Some(latest) = self.frame_times.latest_mut() {
            *latest = previous_frame_time; // rewrite history now that we know
        }
        self.frame_times.add(now, previous_frame_time); // projected
    }

    pub fn mean_frame_time(&self) -> f32 {
        self.frame_times.average().unwrap_or_default()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled(
            false,
            Label::new(format!("Frames #: {}", self.frame_times.total_count())),
        );

        ui.add_enabled(
            false,
            Label::new(format!(
                "Mean CPU: {:.2} ms / frame",
                1e3 * self.mean_frame_time()
            )),
        );
    }
}
