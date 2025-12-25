/// Options for the viewer display.
///
/// These options control what debug information is shown in the viewer, such as points,
/// pen-up trajectories, and control points. By default, all options are disabled.
///
/// # Example
///
/// ```ignore
/// MySketch::runner()
///     .with_viewer_options(
///         ViewerOptions::default()
///             .with_show_points(true)
///             .with_show_control_points(true)
///     )
///     .run()
/// ```
#[derive(Default, Clone)]
pub struct ViewerOptions {
    /// Show points (vertices) on paths.
    pub show_points: bool,

    /// Show pen-up trajectories.
    pub show_pen_up: bool,

    /// Show control points for bezier curves.
    pub show_control_points: bool,
}

impl ViewerOptions {
    /// Enable or disable showing points on paths.
    #[must_use]
    pub fn with_show_points(mut self, show: bool) -> Self {
        self.show_points = show;
        self
    }

    /// Enable or disable showing pen-up trajectories.
    #[must_use]
    pub fn with_show_pen_up(mut self, show: bool) -> Self {
        self.show_pen_up = show;
        self
    }

    /// Enable or disable showing control points for bezier curves.
    #[must_use]
    pub fn with_show_control_points(mut self, show: bool) -> Self {
        self.show_control_points = show;
        self
    }
}
