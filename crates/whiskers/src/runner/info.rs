use whiskers_widgets::collapsing_header;

/// Controls the info section of the runner.
#[derive(Default)]
pub struct InfoOptions {
    pub(crate) description: Option<String>, // TODO: convert to markdown when migrating to 0.23?
    pub(crate) author: Option<String>,
    pub(crate) author_url: Option<String>,
    pub(crate) source_url: Option<String>,
}

impl InfoOptions {
    /// Sets the sketch description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the sketch author.
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets a URL for the author.
    #[must_use]
    pub fn author_url(mut self, author_url: impl Into<String>) -> Self {
        self.author_url = Some(author_url.into());
        self
    }

    /// Sets a URL for the source code.
    #[must_use]
    pub fn source_url(mut self, source_url: impl Into<String>) -> Self {
        self.source_url = Some(source_url.into());
        self
    }
}

impl InfoOptions {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        collapsing_header(ui, "Info", "", true, |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if let Some(description) = &self.description {
                ui.add(egui::Label::new(description.clone()).wrap(true));
            }

            if let Some(author) = &self.author {
                ui.horizontal(|ui| {
                    ui.label("Author:");
                    if let Some(author_url) = &self.author_url {
                        ui.add(egui::Hyperlink::from_label_and_url(author, author_url));
                    } else {
                        ui.label(author);
                    }
                });
            }

            ui.horizontal(|ui| {
                if let Some(source_url) = &self.source_url {
                    ui.add(egui::Hyperlink::from_label_and_url(
                        egui::WidgetText::from("Source"),
                        source_url,
                    ));

                    ui.label("–");
                }

                let font_id = egui::FontId {
                    size: ui.style().text_styles[&egui::TextStyle::Body].size,
                    family: egui::FontFamily::Proportional,
                };
                let mut made_with = egui::text::LayoutJob::default();
                made_with.append(
                    "Made with ",
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        ..Default::default()
                    },
                );
                made_with.append(
                    "whiskers",
                    0.0,
                    egui::TextFormat {
                        font_id,
                        italics: true,
                        ..Default::default()
                    },
                );

                ui.add(egui::Hyperlink::from_label_and_url(
                    made_with,
                    "https://github.com/abey79/vsvg",
                ));
            });
        });
    }
}
