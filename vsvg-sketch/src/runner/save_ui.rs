use crate::Sketch;
use std::fs::canonicalize;
use std::path::PathBuf;

pub(super) struct SaveUI {
    /// The destination directory as typed/displayed in the UI by the user.
    destination_dir_str: String,

    /// The converted destination directory, if it exists.
    destination_dir: Option<PathBuf>,

    /// The output file base name.
    pub(super) base_name: String,

    /// The last save result, if any.
    ///
    /// Used to display status.
    last_error: Option<anyhow::Result<String>>,
}

impl Default for SaveUI {
    fn default() -> Self {
        let target_dir = canonicalize("./").ok().filter(|p| p.is_dir());
        let target_dir_str = target_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or("./".to_string());

        Self {
            destination_dir_str: target_dir_str,
            destination_dir: target_dir,
            base_name: String::from("output"),
            last_error: None,
        }
    }
}

impl SaveUI {
    pub(super) fn ui(&mut self, ui: &mut egui::Ui, sketch: Option<&Sketch>) {
        ui.strong("Save");
        ui.spacing_mut().text_edit_width = 250.0;

        egui::Grid::new("sketch_save_ui")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("destination:");

                ui.horizontal(|ui| {
                    ui.set_width(ui.spacing().text_edit_width);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::LEFT), |ui| {
                        if ui
                            .button("…")
                            .on_hover_text("Select destination directory")
                            .clicked()
                        {
                            // let the user select a directory
                            if let Some(path) = rfd::FileDialog::new()
                                .set_directory(&self.destination_dir_str)
                                .pick_folder()
                            {
                                self.destination_dir_str = path.to_string_lossy().to_string();
                                self.destination_dir = canonicalize(&self.destination_dir_str)
                                    .ok()
                                    .filter(|p| p.is_dir());
                            }
                        }

                        let mut edit = egui::TextEdit::singleline(&mut self.destination_dir_str)
                            .min_size(egui::vec2(ui.available_width(), 0.0));
                        if self.destination_dir.is_none() {
                            edit = edit.text_color(egui::Color32::RED);
                        }
                        if ui.add(edit).changed() {
                            self.destination_dir = canonicalize(&self.destination_dir_str)
                                .ok()
                                .filter(|p| p.is_dir());
                        }
                    });
                });

                ui.end_row();

                ui.label("base name:");
                ui.horizontal(|ui| {
                    ui.set_width(ui.spacing().text_edit_width);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.base_name)
                            .min_size(egui::vec2(ui.available_width(), 0.0)),
                    );
                });

                ui.end_row();

                ui.horizontal(|_| {});

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            sketch.is_some() && self.destination_dir.is_some(),
                            egui::Button::new("save"),
                        )
                        .clicked()
                    {
                        if let Some(sketch) = sketch {
                            if let Some(path) = self.get_output_path() {
                                self.last_error = Some(sketch.save(&path).map(|_| {
                                    path.file_name()
                                        .map(|s| s.to_string_lossy().to_string())
                                        .unwrap_or("<unknown>".to_string())
                                }));
                            }
                        }
                    }

                    if let Some(last_error) = &self.last_error {
                        let txt = match last_error {
                            Ok(file_name) => file_name.to_string(),
                            Err(err) => format!("Error: {}", err),
                        };
                        let label =
                            egui::WidgetText::from(txt)
                                .strong()
                                .color(if last_error.is_ok() {
                                    egui::Color32::DARK_GREEN
                                } else {
                                    egui::Color32::RED
                                });
                        ui.label(label);
                    }
                });
            });
    }

    /// Resets the last save result.
    ///
    /// This is used to clear the status message, typically after the sketch is updated, to indicate
    /// that it changed since last save.
    pub(super) fn reset_error(&mut self) {
        self.last_error = None;
    }

    fn get_output_path(&self) -> Option<PathBuf> {
        let Some(target_dir) = &self.destination_dir else {
            return None;
        };

        let mut idx = 0;
        loop {
            let path = target_dir
                .join(format!("{}_{}", self.base_name, idx))
                .with_extension("svg");

            if !path.exists() {
                break Some(path);
            }

            idx += 1;
        }
    }
}
