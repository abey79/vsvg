use crate::engine::DocumentData;
use crate::frame_history::FrameHistory;
use eframe::Frame;
use egui::{Color32, Ui};

use crate::document_widget::DocumentWidget;
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Viewer {
    /// widget to display the [`vsvg::Document`]
    #[serde(skip)]
    document_widget: DocumentWidget,

    /// should fit to view flag
    #[serde(skip)]
    must_fit_to_view: bool,

    /// Show settings window.
    show_settings: bool,

    /// Show inspection window.
    show_inspection: bool,

    /// Show memory window.
    show_memory: bool,

    /// Record frame performance
    #[serde(skip)]
    frame_history: FrameHistory,
}

impl Viewer {
    pub fn new<'a>(
        cc: &'a eframe::CreationContext<'a>,
        document_data: Arc<DocumentData>,
    ) -> Option<Self> {
        Some(Viewer {
            document_widget: DocumentWidget::new(cc, document_data)?,
            must_fit_to_view: true,
            show_settings: false,
            show_inspection: false,
            show_memory: false,
            frame_history: FrameHistory::default(),
        })
    }

    #[allow(clippy::unused_self)]
    fn menu_file(&self, frame: &mut Frame, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                frame.close();
            }
        });
    }

    fn menu_debug(&mut self, ui: &mut Ui) {
        ui.menu_button("Debug", |ui| {
            if ui.button("Show settings window").clicked() {
                self.show_settings = true;
                ui.close_menu();
            }
            if ui.button("Show inspection window").clicked() {
                self.show_inspection = true;
                ui.close_menu();
            }
            if ui.button("Show memory window").clicked() {
                self.show_memory = true;
                ui.close_menu();
            }
        });
    }
}

impl eframe::App for Viewer {
    /// Called by the framework to save state before shutdown.
    /*fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }*/

    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                self.menu_file(frame, ui);
                self.document_widget.view_menu_ui(ui);
                self.document_widget.layer_menu_ui(ui);
                self.menu_debug(ui);
                self.frame_history.ui(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        let panel_frame = egui::Frame::central_panel(&ctx.style())
            .inner_margin(egui::style::Margin::same(0.))
            .fill(Color32::from_rgb(242, 242, 242));
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| self.document_widget.ui(ui));

        egui::Window::new("🔧 Settings")
            .open(&mut self.show_settings)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });

        egui::Window::new("🔍 Inspection")
            .open(&mut self.show_inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });

        egui::Window::new("📝 Memory")
            .open(&mut self.show_memory)
            .resizable(false)
            .show(ctx, |ui| {
                ctx.memory_ui(ui);
            });

        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
    }
}
