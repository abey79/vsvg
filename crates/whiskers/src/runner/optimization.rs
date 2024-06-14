use itertools::Itertools;
use std::fmt::{Display, Formatter};
use vsvg::DocumentTrait;
use whiskers_widgets::collapsing_header;

/// Options controlling the optimization pass before saving to files.
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OptimizationOptions {
    sort_paths: bool,
    allow_paths_flip: bool,
}

impl OptimizationOptions {
    /// Enable all possible optimization.
    pub fn full() -> Self {
        Self {
            sort_paths: true,
            allow_paths_flip: true,
        }
    }
}

impl Default for OptimizationOptions {
    fn default() -> Self {
        Self {
            sort_paths: true,
            allow_paths_flip: false,
        }
    }
}

impl Display for OptimizationOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let strs = [
            self.sort_paths.then_some("sort paths"),
            self.allow_paths_flip.then_some("flip ok"),
        ];

        write!(f, "{}", strs.into_iter().flatten().join(", "))
    }
}

impl OptimizationOptions {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        collapsing_header(ui, "Optimization", self.to_string(), true, |ui| {
            ui.checkbox(&mut self.sort_paths, "sort paths");
            ui.checkbox(&mut self.allow_paths_flip, "allow path flip");

            let mut ignore: bool = false;
            ui.add_enabled_ui(false, |ui| {
                ui.checkbox(&mut ignore, "merge paths")
                    .on_disabled_hover_text("No yet implemented");
                ui.checkbox(&mut ignore, "reloop paths")
                    .on_disabled_hover_text("No yet implemented");
            });
        });
    }

    pub(crate) fn apply(&self, doc: &mut vsvg::Document) {
        if self.sort_paths {
            doc.for_each(|layer| {
                layer.sort(self.allow_paths_flip);
            });
        }
    }
}
