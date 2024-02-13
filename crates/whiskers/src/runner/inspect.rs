use std::collections::BTreeMap;

use whiskers_widgets::collapsing_header;

/// Controls the debug section of the runner
pub struct InspectVariables {
    pub(crate) label: Option<String>,
    pub(crate) params: BTreeMap<String, String>,
}

impl Default for InspectVariables {
    fn default() -> Self {
        Self {
            label: Some(String::from("Inspect")),
            params: BTreeMap::new(),
        }
    }
}

impl InspectVariables {
    /// Sets the section's label, as it can serve not only for debugging purposes
    #[must_use]
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = Some(value.into());
        self
    }

    /// adds new parameter to the section
    pub fn add_parameter(&mut self, parameter: &(impl AsRef<str>, impl AsRef<str>)) {
        self.params
            .insert(parameter.0.as_ref().into(), parameter.1.as_ref().into());
    }
}

impl InspectVariables {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        if !self.params.is_empty() {
            collapsing_header(ui, self.label.as_ref().unwrap(), "", true, |ui| {
                self.params.iter().for_each(|param| {
                    let (key, value) = param;
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", key.as_str()));
                        ui.label(value.as_str());
                    });
                });
            });
        }
    }
}
