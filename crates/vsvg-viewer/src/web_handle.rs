use eframe::wasm_bindgen::prelude::*;

/// The handle JavaScript uses to load, run and interact with the app.
#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

impl WebHandle {
    pub async fn start(
        &self,
        canvas_id: &str,
        app: impl crate::ViewerApp + 'static,
    ) -> Result<(), JsValue> {
        self.runner
            .start(
                canvas_id,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    Box::new(
                        crate::viewer::Viewer::new(cc, Box::new(app))
                            .expect("what could possibly go wrong?"),
                    )
                }),
            )
            .await
    }
}

#[wasm_bindgen]
impl WebHandle {
    /// Installs a panic hook, then returns.
    #[allow(clippy::new_without_default)]
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Redirect [`log`] message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    // /// Example on how to call into your app from JavaScript.
    // #[wasm_bindgen]
    // pub fn example(&self) {
    //     if let Some(_app) = self.runner.app_mut::<WrapApp>() {
    //         // _app.example();
    //     }
    // }

    /// The JavaScript can check whether your app has crashed:
    #[must_use]
    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.message())
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn panic_callstack(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.callstack())
    }
}
