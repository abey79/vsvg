//! This module implements the UI widgets used to display the sketch parameters in the UI.
//!
//! For each supported sketch parameter type `T`, there must exist a widget type that implements
//! [`Widget<T>`] and is registered with [`crate::register_widget_ui!`] macro. This module include
//! the traits and macros needed to support this mechanism, as well as widgets for basic types.
//!
//! For example, let's consider the [`prim@bool`] type:
//!
//! ```ignore
//! #[derive(Default)]
//! pub struct BoolWidget;
//!
//! impl whiskers::widgets::Widget<bool> for BoolWidget {
//!     fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut bool) -> egui::Response {
//!         ui.horizontal(|_| {});
//!         ui.checkbox(value, label)
//!     }
//! }
//!
//! whiskers::register_widget_ui!(bool, BoolWidget);
//! ```
//!
//! The [`BoolWidget`] type implements the [`Widget<bool>`] trait, which requires the [`Widget::ui`]
//! method, and is registered with the [`crate::register_widget_ui!`] macro. Note that the
//! [`Widget::ui`] method is called in the context of an 2-column [`egui::Grid`], so it must contain
//! exactly two top level UI calls, where the first one typically is the label, and the second the
//! actual interactive widget. In the case of a checkbox, the label is already embedded in the UI
//! widget, we leave the first column empty.
//!
//! The [`BoolWidget`] type is already provided by the [`crate`] crate, but custom widgets can be
//! implemented for custom types using the same pattern.
//!
//! # Configuring widgets
//!
//! Many widgets support additional configuration options, which can be set using the
//! `#[param(...)]` attribute of the [`whiskers_derive::Sketch`] macro. This is done by using the
//! builder pattern on the widget type. For example, here is an extract of [`NumericWidget`], which
//! supports numerical types such as [`f64`] and [`i32`]:
//!
//! ```ignore
//! # use egui::emath::Numeric;
//! use egui::{Response, Ui};
//!
//! #[derive(Default)]
//! pub struct NumericWidget<T: Numeric> {
//!     step: Option<T>,
//!     slider: bool,
//!     /* ... */
//! }
//!
//! impl<T: Numeric> NumericWidget<T> {
//!     pub fn step(mut self, step: T) -> Self {
//!         self.step = Some(step);
//!         self
//!     }
//!
//!     pub fn slider(mut self, slider: bool) -> Self {
//!         self.slider = slider;
//!         self
//!     }
//! }
//!
//! impl<T: Numeric> whiskers::widgets::Widget<T> for NumericWidget<T> {
//!     /* ... */
//! #    fn ui(&self, ui: &mut Ui, label: &str, value: &mut T) -> Response { todo!(); }
//! }
//!
//! whiskers::register_widget_ui!(::f64, NumericWidget<f64>);
//! /* ... */
//!
//! # fn main() {}
//! ```
//! Now let's consider a hypothetical sketch:
//! ```rust
//! # use whiskers::prelude::*;
//! #[derive(Sketch)]
//! struct MySketch {
//!     #[param(slider, step = 0.1)]
//!     irregularity: f64,
//! }
//!
//! # impl App for MySketch {fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
//! #         todo!()
//! #     } }
//! ```
//! Based on the `#[param(...)]` attributes, the [`whiskers_derive::Sketch`] derive macro will
//! automatically generate the corresponding builder pattern calls:
//! ```rust
//! # #[allow(unused_must_use)]
//! # fn main() {
//! whiskers::widgets::NumericWidget::<f64>::default().slider(true).step(0.1);
//! # }
//! ```
//! Note that when no value is provided for a key (such as `slider` here), a boolean value of
//! [`true`] is assumed.

mod bool;
mod numeric_widget;
mod point;
mod string_widget;

pub use crate::widgets::bool::*;
pub use numeric_widget::*;
pub use point::*;
pub use string_widget::*;

/// This is the base trait for widgets used to display sketch parameters in the UI.
///
/// For each supported sketch parameter type `T`, there must exist an implementation of
/// [`Widget<T>`] that is furthermore registered using the [`crate::register_widget_ui!`] macro.
pub trait Widget<T> {
    /// This function implements the actual UI for the widget.
    ///
    /// Note that the [`crate::Runner`] calls this function in the context of a 2-column
    /// [`egui::Grid`], so it must contain exactly two top level egui UI calls.
    fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut T) -> egui::Response;
}

/// This utility trait serves to associate a [`Widget`] type with a given sketch parameter type `T`.
///
/// Do not implement this trait manually, instead use the [`crate::register_widget_ui!`] macro.
pub trait WidgetMapper<T> {
    /// [`Widget`] type associated with the sketch parameter type `T`.
    type Type: Widget<T>;
}

/// Registers a given [`Widget`] type for given sketch parameter type.
///
/// This is a convenience macro that implements the [`WidgetMapper`] trait for the given types.
///
/// # Example
///
/// ```ignore
/// register_widget_ui!(bool, BoolWidget);
/// ```
#[macro_export]
macro_rules! register_widget_ui {
    ($t: ty, $ui: ty) => {
        impl $crate::widgets::WidgetMapper<$t> for $t {
            type Type = $ui;
        }
    };
}
