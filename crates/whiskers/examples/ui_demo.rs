//! This example demonstrates all UI building capabilities of the [`derive@Sketch`] and
//! [`derive@Widget`] derived traits.

// The `#[sketch_widget]` macro generates struct literals for enum variants with `#[skip]` fields.
// These fields are intentionally never read, triggering `unused_assignments`. The lint reports at
// the user's source span, so `#[allow]` in the macro doesn't help - this is a proc macro limitation.
#![allow(unused_assignments)]

use whiskers::prelude::*;

#[sketch_app]
#[derive(Default)]
struct UiDemoSketch {
    // all basic numerical types are supported
    int_64: i64,

    // numerical types can be configured with the `#[param(...)]` attribute
    #[param(min = 0, max = 100)]
    int_0_to_100: i8,

    // other fields may be used
    #[param(min = 0, max = self.int_0_to_100)]
    int_variable_bound: i8,

    // a slider can be used instead of a DragValue
    #[param(slider, min = 0.0, max = 100.0)]
    float_0_to_100: f32,

    // a logarithmic slider can be used also
    #[param(slider, logarithmic, min = 0.01, max = 10.)]
    float_log: f64,

    // `vsvg::Length` are supported...
    length: Length,

    // ...and have similar parameters as numeric types. Also, by default only a subset of the available units is
    // provided. All units can be shown using `all_units`.
    #[param(slider, logarithmic, min = 0.01, max = 10., all_units)]
    length_log: Length,

    // a unit to be used by the sketch to, e.g., create `Length`
    #[param(all_units)]
    unit: Unit,

    // custom types

    // simple enums (no variant with data) are displayed as a combo box that fits the UI gruid
    simple_enum: SimpleEnum,

    // complex enum hierarchically display their variant's content
    custom_enum: CustomEnum,

    // structs display their content hierarchically and can be nested
    custom_struct: CustomStruct,

    // unnamed structs have their field names displayed as "field_0", "field_1", etc.
    custom_struct_unnamed: CustomStructUnnamed,

    // these types are supported but have no configuration options
    boolean: bool,
    string: String,
    color: Color,
    point: Point,

    #[skip]
    incompatible: IncompatibleStruct,

    // lists of simple types are supported
    //
    // Note: the `inner` attribute is used to specify attributes for the inner type (here `f64`)
    #[param(inner(slider, min = 0.0, max = 10.))]
    list_of_simple_type: Vec<f64>,

    custom_struct_list: Vec<CustomStruct>,
}

// If a type doesn't implement [`Widget`], it can still be used, but `#[skip]` must be used. The
// type still must implement `Default` and `{S|Des}erialize` so be compatible with the enclosing
// type.
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct IncompatibleStruct {
    some_float: f64,
}

// Custom types may be used as sketch parameter if a corresponding [`whiskers::widgets::Widget`]
// type exists. This can be done using the [`whiskers_derive::Widget`] derive macro. Alternatively,
// the [`whiskers::widgets::WidgetMapper`] trait can be implemented manually, see the `custom_ui`
// example.
// Note: all types must implement [`Default`].
#[sketch_widget]
#[derive(Default)]
struct CustomStruct {
    #[param(min = 0.0, max = 10.0)]
    some_float: f64,

    #[param(slider, min = 0.0, max = self.some_float)]
    another_float: f64,

    // nested struct are supported
    custom_struct_unnamed: CustomStructUnnamed,

    #[skip]
    incompatible: IncompatibleStruct,
}

// Tuple structs are supported too
#[sketch_widget]
#[derive(Default)]
struct CustomStructUnnamed(
    bool,
    String,
    #[param(slider, min = 0.0, max = 1.0)] f64,
    #[skip] IncompatibleStruct,
);

#[sketch_widget]
#[derive(Default)]
enum SimpleEnum {
    #[default]
    Poodle,
    Corgy,
    Dalmatian,
}

#[sketch_widget]
#[derive(Default)]
enum CustomEnum {
    Variant1 {
        #[param(slider, min = 0.0, max = 1.0)]
        some_float: f64,

        #[skip]
        incompatible: IncompatibleStruct,
    },
    Variant2(
        bool,
        #[param(slider, min = 0.0, max = 1.0)] f64,
        #[skip] IncompatibleStruct,
    ),
    #[default]
    Variant3,
}

impl App for UiDemoSketch {
    fn update(&mut self, _sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        Ok(())
    }
}

fn main() -> Result {
    UiDemoSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .with_layout_options(LayoutOptions::Center)
        .run()
}
