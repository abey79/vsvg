//! This example demonstrates all UI building capabilities of the [`derive@Sketch`] and
//! [`derive@Widget`] derived traits.

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
    custom_struct: CustomStruct,
    custom_struct_unnamed: CustomStructUnnamed,
    custom_enum: CustomEnum,

    // these types are supported but have no configuration options
    boolean: bool,
    string: String,
    color: Color,
    point: Point,
}

// Custom types may be used as sketch parameter if a corresponding [`whiskers::widgets::Widget`]
// type exists. This can be done using the [`whiskers_derive::Widget`] derive macro. Alternatively,
// the [`whiskers::widgets::WidgetMapper`] trait can be implemented manually, see the `custom_ui`
// example.
// Note: all types must implement [`Default`].
#[sketch_widget]
#[derive(Default)]
struct CustomStruct {
    #[param(min = 0.0)]
    some_float: f64,

    #[param(min = 0.0, max = self.some_float)]
    another_float: f64,

    // nested struct are supported
    custom_struct_unnamed: CustomStructUnnamed,
}

// Tuple structs are supported too
#[sketch_widget]
#[derive(Default)]
struct CustomStructUnnamed(bool, String);

impl App for UiDemoSketch {
    fn update(&mut self, _sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Widget, serde::Serialize, serde::Deserialize)]
#[serde(crate = "::whiskers::prelude::serde")]
enum CustomEnum {
    Variant1,
    Variant2,
    /*Variant1 { some_float: f64, some_bool: bool },
    Variant2(String, bool),*/
    Variant3,
}

impl Default for CustomEnum {
    fn default() -> Self {
        /*Self::Variant1 {
            some_float: 0.0,
            some_bool: false,
        }*/

        Self::Variant1
    }
}
// impl CustomEnum {
//     #[allow(non_snake_case)]
//     fn __default_Variant1() -> Self {
//         CustomEnum::Variant1
//     }
//     #[allow(non_snake_case)]
//     fn __default_Variant2() -> Self {
//         CustomEnum::Variant2
//     }
//     #[allow(non_snake_case)]
//     fn __default_Variant3() -> Self {
//         CustomEnum::Variant3
//     }
// }
// #[derive(Default)]
// pub struct CustomEnumWidget;
// impl ::whiskers::widgets::Widget<CustomEnum> for CustomEnumWidget {
//     fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut CustomEnum) -> bool {
//         let mut selected_text = match value {
//             CustomEnum::Variant1 => "Variant1",
//             CustomEnum::Variant2 => "Variant2",
//             CustomEnum::Variant3 => "Variant3",
//         }
//         .to_owned();
//         let initial_selected_text = selected_text.clone();
//         egui::ComboBox::from_label("CustomEnum")
//             .selected_text(&selected_text)
//             .show_ui(ui, |ui| {
//                 ui.selectable_value(&mut selected_text, "Variant1".to_owned(), "Variant1");
//                 ui.selectable_value(&mut selected_text, "Variant2".to_owned(), "Variant2");
//                 ui.selectable_value(&mut selected_text, "Variant3".to_owned(), "Variant3");
//             });
//         let mut changed = initial_selected_text != selected_text;
//         if changed {
//             *value = match selected_text.as_str() {
//                 "Variant1" => CustomEnum::__default_Variant1(),
//                 "Variant2" => CustomEnum::__default_Variant2(),
//                 "Variant3" => CustomEnum::__default_Variant3(),
//                 _ => unreachable!(),
//             };
//         }
//         changed |= match value {
//             CustomEnum::Variant1 => false,
//             CustomEnum::Variant2 => false,
//             CustomEnum::Variant3 => false,
//         };
//         changed
//     }
//     fn use_grid() -> bool {
//         false
//     }
// }
// impl ::whiskers::widgets::WidgetMapper<CustomEnum> for CustomEnum {
//     type Type = CustomEnumWidget;
// }

// #[derive(Default)]
// pub struct CustomEnumWidget;
// impl ::whiskers::widgets::Widget<CustomEnum> for CustomEnumWidget {
//     fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut CustomEnum) -> bool {
//         match value {
//             CustomEnum::Variant1 { some_float } => {
//                 let mut array: &mut [(&mut dyn FnMut(&mut egui::Ui) -> bool, &dyn Fn() -> bool)] =
//                     &mut [(
//                         &mut |ui| {
//                             <f64 as ::whiskers::widgets::WidgetMapper<f64>>::Type::default().ui(
//                                 ui,
//                                 "some float:",
//                                 some_float,
//                             )
//                         },
//                         &<f64 as ::whiskers::widgets::WidgetMapper<f64>>::Type::use_grid,
//                     )];
//                 let mut cur_index = 0;
//                 let mut changed = false;
//                 while cur_index < array.len() {
//                     if array[cur_index].1() {
//                         egui::Grid::new(cur_index).num_columns(2).show(ui, |ui| {
//                             while cur_index < array.len() && array[cur_index].1() {
//                                 changed = (array[cur_index].0)(ui) || changed;
//                                 ui.end_row();
//                                 cur_index += 1;
//                             }
//                         });
//                     }
//                     while cur_index < array.len() && !array[cur_index].1() {
//                         changed = (array[cur_index].0)(ui) || changed;
//                         cur_index += 1;
//                     }
//                 }
//                 changed
//             }
//             CustomEnum::Variant2(p0) => {
//                 let array: &[(&dyn FnMut(&mut egui::Ui) -> bool, &dyn Fn() -> bool)] = &[(
//                     &|ui| {
//                         <String as ::whiskers::widgets::WidgetMapper<String>>::Type::default()
//                             .ui(ui, "field 0:", p0)
//                     },
//                     &<String as ::whiskers::widgets::WidgetMapper<String>>::Type::use_grid,
//                 )];
//                 let mut cur_index = 0;
//                 let mut changed = false;
//                 while cur_index < array.len() {
//                     if array[cur_index].1() {
//                         egui::Grid::new(cur_index).num_columns(2).show(ui, |ui| {
//                             while cur_index < array.len() && array[cur_index].1() {
//                                 changed = (array[cur_index].0)(ui) || changed;
//                                 ui.end_row();
//                                 cur_index += 1;
//                             }
//                         });
//                     }
//                     while cur_index < array.len() && !array[cur_index].1() {
//                         changed = (array[cur_index].0)(ui) || changed;
//                         cur_index += 1;
//                     }
//                 }
//                 changed
//             }
//             CustomEnumVariant3 => false,
//         }
//     }
//     fn use_grid() -> bool {
//         false
//     }
// }
// impl ::whiskers::widgets::WidgetMapper<CustomEnum> for CustomEnum {
//     type Type = CustomEnumWidget;
// }

fn main() -> Result {
    UiDemoSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .with_layout_options(LayoutOptions::Center)
        .run()
}
