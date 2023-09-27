use crate::runner::collapsing_header;
use crate::Sketch;
use std::fmt::Display;

/// Controls the layout feature of the runner.
#[derive(serde::Serialize, serde::Deserialize, Default, PartialEq)]
pub enum LayoutOptions {
    /// The layout feature is disabled, but the UI is visible.
    #[default]
    Off,

    /// The sketch is centered on the page.
    Center,
}

impl LayoutOptions {
    pub fn off() -> Self {
        Self::Off
    }

    pub fn centered() -> Self {
        Self::Center
    }
}

impl Display for LayoutOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutOptions::Off => write!(f, "off"),
            LayoutOptions::Center => write!(f, "centered"),
        }
    }
}

impl LayoutOptions {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        collapsing_header(ui, "Layout", self.to_string(), true, |ui| {
            let mut changed = false;
            changed |= ui.radio_value(self, LayoutOptions::Off, "Off").changed();
            changed |= ui
                .radio_value(self, LayoutOptions::Center, "Centered")
                .changed();

            changed
        })
        .unwrap_or(false)
    }

    pub(crate) fn apply(&self, sketch: &mut Sketch) {
        match self {
            LayoutOptions::Off => {}
            LayoutOptions::Center => {
                sketch.document_mut().center_content();
            }
        }
    }
}
