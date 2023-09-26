use crate::Sketch;

#[derive(serde::Deserialize, serde::Serialize)]
pub(super) struct SaveUI {
    /// The output file base name.
    pub(super) base_name: String,
}

impl Default for SaveUI {
    fn default() -> Self {
        Self {
            base_name: String::from("output"),
        }
    }
}

impl SaveUI {
    pub(super) fn ui(&mut self, _ui: &mut egui::Ui, _sketch: Option<&Sketch>) {
        //TODO
    }
}
