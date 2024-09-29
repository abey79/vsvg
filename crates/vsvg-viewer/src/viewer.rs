use crate::frame_history::FrameHistory;
use eframe::Frame;
use egui::{Color32, Ui};

#[cfg(puffin)]
use crate::profiler::Profiler;
use crate::{document_widget::DocumentWidget, ViewerApp};

const VSVG_VIEWER_STATE_STORAGE_KEY: &str = "vsvg-viewer-state";
const VSVG_VIEWER_ANTIALIAS_STORAGE_KEY: &str = "vsvg-viewer-aa";

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[allow(clippy::struct_excessive_bools, clippy::struct_field_names)]
struct ViewerState {
    /// Show settings window.
    show_settings: bool,

    /// Show inspection window.
    show_inspection: bool,

    /// Show memory window.
    show_memory: bool,
}

#[allow(clippy::struct_excessive_bools)]
pub struct Viewer {
    state: ViewerState,

    /// widget to display the [`vsvg::Document`]
    document_widget: DocumentWidget,

    /// Record frame performance
    frame_history: FrameHistory,

    viewer_app: Box<dyn ViewerApp>,

    #[cfg(puffin)]
    profiler: Profiler,
}

impl Viewer {
    #[must_use]
    pub(crate) fn new<'a>(
        cc: &'a eframe::CreationContext<'a>,
        mut viewer_app: Box<dyn ViewerApp>,
    ) -> Option<Self> {
        let mut document_widget = DocumentWidget::new(cc)?;

        let state = if let Some(storage) = cc.storage {
            viewer_app.load(storage);

            if let Some(aa) = eframe::get_value(storage, VSVG_VIEWER_ANTIALIAS_STORAGE_KEY) {
                document_widget.set_antialias(aa);
            };

            eframe::get_value(storage, VSVG_VIEWER_STATE_STORAGE_KEY).unwrap_or_default()
        } else {
            ViewerState::default()
        };

        //TODO: better error handling
        viewer_app
            .setup(cc, &mut document_widget)
            .expect("viewer app setup failed");

        Some(Viewer {
            state,
            document_widget,
            frame_history: FrameHistory::default(),
            viewer_app,
            #[cfg(puffin)]
            profiler: Profiler::default(),
        })
    }

    #[allow(clippy::unused_self)]
    #[cfg(not(target_arch = "wasm32"))]
    fn menu_file(&self, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn menu_debug(&mut self, ui: &mut Ui) {
        ui.menu_button("Debug", |ui| {
            if ui.button("Show settings window").clicked() {
                self.state.show_settings = true;
                ui.close_menu();
            }
            if ui.button("Show inspection window").clicked() {
                self.state.show_inspection = true;
                ui.close_menu();
            }
            if ui.button("Show memory window").clicked() {
                self.state.show_memory = true;
                ui.close_menu();
            }

            #[cfg(puffin)]
            if ui.button("Show profiler window").clicked() {
                self.profiler.start();
                ui.close_menu();
            }

            #[cfg(debug_assertions)]
            {
                ui.separator();
                Self::egui_debug_options_ui(ui);
            }
        });
    }

    #[cfg(debug_assertions)]
    fn egui_debug_options_ui(ui: &mut Ui) {
        // copied from rerun!

        let mut debug = ui.style().debug;
        let mut any_clicked = false;

        any_clicked |= ui
            .checkbox(&mut debug.debug_on_hover, "Ui debug on hover")
            .on_hover_text("Hover over widgets to see their rectangles")
            .changed();
        any_clicked |= ui
            .checkbox(&mut debug.show_expand_width, "Show expand width")
            .on_hover_text("Show which widgets make their parent wider")
            .changed();
        any_clicked |= ui
            .checkbox(&mut debug.show_expand_height, "Show expand height")
            .on_hover_text("Show which widgets make their parent higher")
            .changed();
        any_clicked |= ui.checkbox(&mut debug.show_resize, "Show resize").changed();
        any_clicked |= ui
            .checkbox(
                &mut debug.show_interactive_widgets,
                "Show interactive widgets",
            )
            .on_hover_text("Show an overlay on all interactive widgets")
            .changed();

        if any_clicked {
            let mut style = (*ui.ctx().style()).clone();
            style.debug = debug;
            ui.ctx().set_style(style);
        }
    }
}

impl eframe::App for Viewer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        #[cfg(feature = "puffin")]
        {
            puffin::GlobalProfiler::lock().new_frame();
        }

        vsvg::trace_function!();

        // hook to handle input (called early to allow capturing input before egui)
        self.viewer_app.handle_input(ctx, &mut self.document_widget);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                self.menu_file(ui);

                self.document_widget.view_menu_ui(ui);
                self.document_widget.layer_menu_ui(ui);
                self.menu_debug(ui);
                self.frame_history.ui(ui);
                ui.add_enabled(
                    false,
                    egui::Label::new(format!("Vertices: {}", self.document_widget.vertex_count())),
                );
                egui::warn_if_debug_build(ui);
            });
        });

        // hook for creating side panels
        //TODO: better error management
        self.viewer_app
            .show_panels(ctx, &mut self.document_widget)
            .expect("ViewerApp failed!!!");

        let panel_frame = egui::Frame::central_panel(&ctx.style())
            .inner_margin(egui::Margin::same(0.))
            .fill(Color32::from_rgb(242, 242, 242));

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                self.document_widget.ui(ui);

                //TODO: better error management
                self.viewer_app
                    .show_central_panel(ui, &mut self.document_widget)
                    .expect("ViewerApp failed!!!");
            });

        egui::Window::new("🔧 Settings")
            .open(&mut self.state.show_settings)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });

        egui::Window::new("🔍 Inspection")
            .open(&mut self.state.show_inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });

        egui::Window::new("📝 Memory")
            .open(&mut self.state.show_memory)
            .resizable(false)
            .show(ctx, |ui| {
                ctx.memory_ui(ui);
            });

        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, VSVG_VIEWER_STATE_STORAGE_KEY, &self.state);
        eframe::set_value(
            storage,
            VSVG_VIEWER_ANTIALIAS_STORAGE_KEY,
            &self.document_widget.antialias(),
        );
        self.viewer_app.save(storage);
    }

    /// Called by the framework before shutting down.
    fn on_exit(&mut self) {
        self.viewer_app.on_exit();
    }
}
