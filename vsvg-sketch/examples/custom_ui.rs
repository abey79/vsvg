use egui::Ui;
use vsvg_sketch::prelude::*;

/// Very custom data structure that is not supported by default
#[derive(Debug, Clone, Copy)]
struct GrayRed {
    gray: f64,
    red: f64,
}

impl From<GrayRed> for vsvg::Color {
    fn from(value: GrayRed) -> Self {
        let red = ((value.red) * 255.0) as u8;
        let gray = (value.gray * 255.0) as u8;
        vsvg::Color::new(red, gray, gray, 255)
    }
}

/// Custom UI widget for [`GreyRed`]. It must implement the [`Widget<GrayRed>`] trait.
#[derive(Default)]
struct GrayRedWidget {
    label_color: egui::Color32,
    underline: bool,
}

/// We want the ability to customise the look of our widget!
impl GrayRedWidget {
    pub fn label_color(mut self, color: egui::Color32) -> Self {
        self.label_color = color;
        self
    }

    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }
}

/// This is where the custom UI code happens.
impl Widget<GrayRed> for GrayRedWidget {
    fn ui(&self, ui: &mut Ui, label: &str, value: &mut GrayRed) -> egui::Response {
        ui.horizontal(|ui| {
            let mut label = egui::RichText::new(label).color(self.label_color);
            if self.underline {
                label = label.underline();
            }
            ui.add(egui::Label::new(label));

            ui.label("gr:");
            let resp1 = ui.add(egui::Slider::new(&mut value.gray, 0.0..=1.0));
            ui.label("rd:");
            let resp2 = ui.add(egui::Slider::new(&mut value.red, 0.0..=1.0));

            // we must return a response that combines the responses of the sub-widgets to make
            // sure any change to the slider are reported
            resp1 | resp2
        })
        .inner
    }
}

// Let the [`Sketch`] derive macro know that [`GrayRedWidget`] is the UI widget for [`GrayRed`].
register_widget_ui!(GrayRed, GrayRedWidget);

// =================================================================================
// from here on, we're back to super standard  sketch code...

#[derive(Sketch)]
struct CustomUISketch {
    // these param key/value will call into the [`GrayRedWidget`]'s builder methods.
    #[param(underline, label_color = egui::Color32::BLUE)]
    color: GrayRed,
}

impl App for CustomUISketch {
    fn update(&mut self, sketch: &mut Sketch) -> anyhow::Result<()> {
        sketch.page_size(PageSize::new(200.0, 200.0));

        sketch.color(self.color);
        for i in 0..5 {
            sketch.circle(100.0, 100.0, 30.0 + 40.0 + i as f64 * 3.0);
        }

        Ok(())
    }
}

fn main() -> Result {
    run(CustomUISketch {
        color: GrayRed {
            red: 0.5,
            gray: 0.5,
        },
    })
}
