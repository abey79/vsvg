/// Display a properly-styled collapsing header with some content. If the header is collapsed, the
/// summary is displayed inline instead of the content.
pub fn collapsing_header<R>(
    ui: &mut egui::Ui,
    label: impl AsRef<str>,
    summary: impl AsRef<str>,
    default_open: bool,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    let label = label.as_ref();

    let id = ui.make_persistent_id(label);
    let collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        id,
        default_open,
    );
    let open = collapsing.is_open();
    let (_, _, body_response) = collapsing
        .show_header(ui, |ui| {
            ui.strong(label);
            if !open {
                ui.label(summary.as_ref());
            }
        })
        .body(body);

    body_response.map(|r| r.inner)
}
