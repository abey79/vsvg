use egui::Color32;
use std::path::PathBuf;

pub(crate) struct MultiViewer {
    paths: Vec<PathBuf>,
    selected_path: usize,
}

impl MultiViewer {
    pub(crate) fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            paths,
            selected_path: 0,
        }
    }
}

impl eframe::App for MultiViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // handle input
        //if !ctx.wants_keyboard_input() {}

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowUp) {
                    self.selected_path = self.selected_path.saturating_sub(1);
                } else if i.key_pressed(egui::Key::ArrowDown) {
                    self.selected_path = (self.selected_path + 1).min(self.paths.len() - 1);
                }
            });
            for (i, path) in self.paths.iter().enumerate() {
                ui.selectable_value(&mut self.selected_path, i, path.to_string_lossy());
            }
        });

        let panel_frame = egui::Frame::central_panel(&ctx.style())
            .inner_margin(egui::style::Margin::same(0.))
            .fill(Color32::from_rgb(242, 242, 242));
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| ui.label("hello world"));
    }
}
