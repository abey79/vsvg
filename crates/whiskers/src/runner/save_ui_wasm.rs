use std::sync::Arc;
use vsvg::{Document, DocumentTrait};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, BlobPropertyBag, Url};
use whiskers_widgets::collapsing_header;

#[derive(serde::Deserialize, serde::Serialize)]
pub(super) struct SaveUI {
    /// The output file base name.
    pub(super) base_name: String,

    /// The last save result, if any.
    ///
    /// Used to display status.
    #[serde(skip)]
    last_error: Option<anyhow::Result<()>>,
}

impl Default for SaveUI {
    fn default() -> Self {
        Self {
            base_name: String::from("output"),
            last_error: None,
        }
    }
}

impl SaveUI {
    pub(super) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        document: Option<Arc<vsvg::Document>>,
        optimize_fn: impl FnOnce(&mut Document),
    ) {
        collapsing_header(ui, "Save", "", true, |ui| {
            ui.spacing_mut().text_edit_width = 250.0;

            egui::Grid::new("sketch_save_ui")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("name:");
                    ui.text_edit_singleline(&mut self.base_name);

                    ui.end_row();

                    ui.horizontal(|_| {});
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(document.is_some(), egui::Button::new("download"))
                            .clicked()
                        {
                            if let Some(document) = document {
                                let mut document = (*document).clone();
                                optimize_fn(&mut document);

                                let res = save_and_download(&self.base_name, &document);
                                self.last_error = Some(res);
                            }
                        }

                        if let Some(Err(error)) = &self.last_error {
                            ui.label(
                                egui::WidgetText::from("ERROR")
                                    .strong()
                                    .color(egui::Color32::RED),
                            )
                            .on_hover_text(error.to_string());
                        }
                    });
                });
        });
    }
}

fn save_and_download(name: &str, doc: &vsvg::Document) -> anyhow::Result<()> {
    let svg = doc.to_svg_string()?;
    download_file(name, &svg).ok_or(anyhow::anyhow!("Failed to trigger download"))
}

//https://stackoverflow.com/a/19328891/229511
fn download_file(name: &str, content: &str) -> Option<()> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let body = document.body()?;

    let aa = document.create_element("a").ok()?;
    let a = aa.dyn_into::<web_sys::HtmlElement>().ok()?;
    a.style().set_property("display", "none").ok()?;
    body.append_child(&a).ok()?;

    let mut blob_options = BlobPropertyBag::new();
    blob_options.type_("image/svg+xml;charset=utf-8");

    let blob_sequence = js_sys::Array::new_with_length(1);
    blob_sequence.set(0, JsValue::from(content));

    let blob = Blob::new_with_blob_sequence_and_options(&blob_sequence, &blob_options).ok()?;

    let url = Url::create_object_url_with_blob(&blob).ok()?;

    a.set_attribute("href", &url).ok()?;
    a.set_attribute("download", name).ok()?;
    a.click();
    Url::revoke_object_url(&url).ok()?;
    a.remove();

    Some(())
}
