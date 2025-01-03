use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, parse_quote, visit_mut::VisitMut, Attribute, Data, DataEnum, DataStruct,
    DeriveInput, Expr, ExprPath, Field, Fields, FieldsNamed, FieldsUnnamed, Index, Variant,
};

fn label_from_ident(ident: &Ident) -> String {
    format!("{}:", ident.to_string().to_case(Case::Lower))
}

/// Attribute macro to automatically derive some of the required traits for a sketch app.
///
/// This is equivalent to:
/// ```ignore
/// #[derive(Sketch, serde::Serialize, serde::Deserialize)]
/// #[serde(crate = "::whiskers::exports::serde")]
/// ```
#[proc_macro_attribute]
pub fn sketch_app(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let expanded = quote! {
        #[derive(Sketch, ::whiskers::exports::serde::Serialize, ::whiskers::exports::serde::Deserialize)]
        #[serde(crate = "::whiskers::exports::serde")]
        #ast
    };

    TokenStream::from(expanded)
}

/// Attribute macro to automatically derive some of the required traits for a sketch widget.
///
/// This is equivalent to:
/// ```ignore
/// #[derive(Widget, serde::Serialize, serde::Deserialize)]
/// #[serde(crate = "::whiskers::exports::serde")]
/// ```
#[proc_macro_attribute]
pub fn sketch_widget(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let expanded = quote! {
        #[derive(Widget, ::whiskers::exports::serde::Serialize, ::whiskers::exports::serde::Deserialize)]
        #[serde(crate = "::whiskers::exports::serde")]
        #ast
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Sketch, attributes(param, skip))]
pub fn sketch_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = input.ident;

    let fields_ui = match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            process_fields(fields, &format_ident!("Self"), &format_ident!("self"))
        }
        _ => panic!("The Sketch derive macro only supports structs"),
    };

    TokenStream::from(quote! {
        impl ::whiskers::prelude::whiskers_widgets::WidgetApp for #name {
            fn name(&self) -> String {
                stringify!(#name).to_string()
            }

            fn ui(&mut self, ui: &mut ::whiskers::exports::egui::Ui) -> bool {
                #fields_ui
            }
        }

        impl ::whiskers::SketchApp for #name {}
    })
}

#[proc_macro_derive(Widget, attributes(param, skip))]
pub fn sketch_ui_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = input.ident;
    let widget_name = format_ident!("{}Widget", name);

    match input.data {
        Data::Struct(DataStruct { fields, .. }) => process_struct(fields, &name, &widget_name),
        Data::Enum(DataEnum { variants, .. }) => process_enum(variants, &name, &widget_name),
        Data::Union(_) => {
            unimplemented!()
        }
    }
}

fn process_struct(fields: Fields, name: &Ident, widget_name: &Ident) -> TokenStream {
    let fields_ui = process_fields(fields, name, &format_ident!("value"));

    TokenStream::from(quote! {
        #[derive(Default)]
        pub struct #widget_name;

        impl ::whiskers::prelude::whiskers_widgets::Widget<#name> for #widget_name {
            fn ui(&self, ui: &mut ::whiskers::exports::egui::Ui, label: &str, value: &mut #name) -> bool {
                ::whiskers::prelude::whiskers_widgets::collapsing_header(ui, label.trim_end_matches(':'), "", true, |ui|{
                        #fields_ui
                    })
                    .unwrap_or(false)
            }

            fn use_grid() -> bool {
                false
            }
        }

        impl ::whiskers::prelude::whiskers_widgets::WidgetMapper<#name> for #name {
            type Type = #widget_name;
        }
    })
}

fn field_defaults<'a>(fields: impl Iterator<Item = &'a Field>) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    for field in fields {
        let typ_ = &field.ty;
        if let Some(name) = &field.ident {
            output.extend(quote! {
                #name: #typ_::default(),
            });
        } else {
            output.extend(quote! {
                #typ_::default(),
            });
        }
    }

    output
}

fn default_function_name_for_variant(variant_ident: &Ident) -> Ident {
    format_ident!("__default_{}", variant_ident)
}

fn process_enum(
    variants: Punctuated<Variant, Comma>,
    name: &Ident,
    widget_name: &Ident,
) -> TokenStream {
    //
    // For each variant, create a function that returns the default value for that variant.
    //

    let mut default_functions = proc_macro2::TokenStream::new();
    let mut simple_enum = true;
    for Variant { ident, fields, .. } in variants.iter() {
        let func_ident = default_function_name_for_variant(ident);

        let fields_defaults = match fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                simple_enum = false;

                let fields = field_defaults(unnamed.iter());
                quote! {( #fields )}
            }
            Fields::Named(FieldsNamed { named, .. }) => {
                simple_enum = false;

                let fields = field_defaults(named.iter());
                quote! {{ #fields }}
            }
            Fields::Unit => {
                quote! {}
            }
        };

        default_functions.extend(quote! {
            #[allow(non_snake_case)]
            fn #func_ident() -> Self {
                #name::#ident #fields_defaults
            }
        });
    }

    let impl_default_functions = quote! {
        impl #name {
            #default_functions
        }
    };

    //
    // Create the UI code for the combo box menu. This is done in parts that are combined later, differently
    // depending on whether the enum is simple or complex.
    //

    let idents = variants
        .iter()
        .map(|Variant { ident, .. }| ident.clone())
        .collect::<Vec<_>>();

    let field_captures_catch_all: Vec<_> = variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Named(FieldsNamed { .. }) => quote! { { .. } },
            Fields::Unnamed(FieldsUnnamed { .. }) => quote! { ( .. ) },
            Fields::Unit => quote! {},
        })
        .collect();

    let ident_default_functions = idents
        .iter()
        .map(default_function_name_for_variant)
        .collect::<Vec<_>>();
    let ident_strings = idents
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<_>>();

    let name_string = name.to_string();

    let pre_combo_code = quote! {
        let mut selected_text = match value {
            #(
                #name::#idents #field_captures_catch_all => #ident_strings,
            )*
        }.to_owned();
        let initial_selected_text = selected_text.clone();
    };

    let combo_code = quote! {
        ::whiskers::exports::egui::ComboBox::from_id_source(#name_string).selected_text(&selected_text).show_ui(ui, |ui| {
            #(
                ui.selectable_value(&mut selected_text, #ident_strings.to_owned(), #ident_strings);
            )*
        });
    };

    let post_combo_code = quote! {
        let mut changed = initial_selected_text != selected_text;

        if changed {
            *value = match selected_text.as_str() {
                #(
                    #ident_strings => #name::#ident_default_functions(),
                )*
                _ => unreachable!(),
            };
        }
    };

    //
    // Simple enum case: build a simple UI and return.
    //

    if simple_enum {
        let simple_enum_full_code = quote! {
            #impl_default_functions

            #[derive(Default)]
            pub struct #widget_name;

            impl ::whiskers::prelude::whiskers_widgets::Widget<#name> for #widget_name {
                fn ui(&self, ui: &mut ::whiskers::exports::egui::Ui, label: &str, value: &mut #name) -> bool {
                    #pre_combo_code

                    ui.label(label);
                    #combo_code

                    #post_combo_code

                    changed
                }

                fn use_grid() -> bool {
                    true
                }
            }

            impl ::whiskers::prelude::whiskers_widgets::WidgetMapper<#name> for #name {
                type Type = #widget_name;
            }
        };

        return TokenStream::from(simple_enum_full_code);
    }

    //
    // Complex enum case: use a collapsing header whose body display the variant's UI.
    //

    // collect things like:
    // - tuple variant => (field_0, field_1)
    // - struct variant => { some_field, another_field }
    // - unit variant => <empty>
    let field_captures: Vec<_> = variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named
                    .iter()
                    .map(|field| field.ident.clone().unwrap())
                    .collect::<Vec<_>>();

                quote! {
                    { #( #fields, )* }
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let fields = (0..unnamed.len())
                    .map(|idx| format_ident!("field_{}", Index::from(idx)))
                    .collect::<Vec<_>>();

                quote! {
                    ( #( #fields, )* )
                }
            }
            Fields::Unit => quote! {},
        })
        .collect();

    // collect (UI function, grid predicate) list of tuples for each variants
    let field_tuples: Vec<_> = variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Named(FieldsNamed { named: field_list, .. })
                | Fields::Unnamed(FieldsUnnamed { unnamed: field_list, .. }) => {
                let field_names = field_list
                    .iter()
                    .filter(|variant| !has_skip_attr(&variant.attrs))
                    .enumerate()
                    .map(|(idx, field)| field
                        .ident
                        .clone()
                        .unwrap_or(format_ident!("field_{}", Index::from(idx))))
                    .collect::<Vec<_>>();
                let field_types = field_list
                    .iter()
                    .filter(|variant| !has_skip_attr(&variant.attrs))
                    .map(|field| field.ty.clone())
                    .collect::<Vec<_>>();
                let field_labels = field_names
                    .iter()
                    .map(label_from_ident)
                    .collect::<Vec<_>>();
                let chained_calls = field_list
                    .iter()
                    .filter(|variant| !has_skip_attr(&variant.attrs))
                    .map(|field| chained_call_for_attrs(&field.attrs))
                    .collect::<Vec<_>>();

                quote! {
                    #(
                        (
                            &mut |ui| {
                                <#field_types as ::whiskers::prelude::whiskers_widgets::WidgetMapper<#field_types>>::Type::default()
                                    #chained_calls
                                    .ui(
                                        ui,
                                        #field_labels,
                                        #field_names,
                                    )
                            },
                            &<#field_types as ::whiskers::prelude::whiskers_widgets::WidgetMapper<#field_types>>::Type::use_grid,
                        )
                    ),*
                }
            }
            Fields::Unit => quote!{
                (
                    &mut |ui| {
                        ui.label(::whiskers::exports::egui::RichText::new("no fields for this variant").weak().italics());
                        false
                    },
                    &|| false,
                )
            }
        })
        .collect();

    //
    // Final assembly of the complex enum code.
    //

    TokenStream::from(quote! {
        #impl_default_functions

        #[derive(Default)]
        pub struct #widget_name;

        impl ::whiskers::prelude::whiskers_widgets::Widget<#name> for #widget_name {
            fn ui(&self, ui: &mut ::whiskers::exports::egui::Ui, label: &str, value: &mut #name) -> bool {

                // draw the UI for a bunch of fields, swapping the grid on and off based on grid support
                fn draw_ui(
                    ui: &mut ::whiskers::exports::egui::Ui,
                    changed: &mut bool,
                    array: &mut [(&mut dyn FnMut(&mut egui::Ui) -> bool, &dyn Fn() -> bool)],
                ) {
                    let mut cur_index = 0;
                    while cur_index < array.len() {
                        if array[cur_index].1() {
                            ::whiskers::exports::egui::Grid::new(cur_index).num_columns(2).show(ui, |ui| {
                                while cur_index < array.len() && array[cur_index].1() {
                                    *changed = (array[cur_index].0)(ui) || *changed;
                                    ui.end_row();
                                    cur_index += 1;
                                }
                            });
                        }
                        while cur_index < array.len() && !array[cur_index].1() {
                            *changed = (array[cur_index].0)(ui) || *changed;
                            cur_index += 1;
                        }
                    }
                }

                let (header_changed, body_changed) = ::whiskers::prelude::whiskers_widgets::enum_collapsing_header(
                    ui,
                    label,
                    value,
                    |ui, value| {
                        #pre_combo_code
                        #combo_code
                        #post_combo_code

                        changed
                    },
                    true,
                    |ui, value| {

                        let mut changed = false;

                        match value {
                            #(
                                #[allow(unused_variables)]
                                #name::#idents #field_captures => {
                                    draw_ui(
                                        ui,
                                        &mut changed,
                                        &mut [ #field_tuples ],
                                    );
                                }
                            )*
                        };

                        changed

                    },
                );

                header_changed || body_changed.unwrap_or(false)
            }

            fn use_grid() -> bool {
                false
            }
        }

        impl ::whiskers::prelude::whiskers_widgets::WidgetMapper<#name> for #name {
            type Type = #widget_name;
        }
    })
}

fn has_skip_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("skip"))
}
fn chained_call_for_attrs(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let param_attr = attrs.iter().find(|attr| attr.path().is_ident("param"));

    let mut chained_calls = proc_macro2::TokenStream::new();

    let mut add_chained_call = |meta: syn::meta::ParseNestedMeta, inner: bool| -> syn::Result<()> {
        let ident = meta.path.get_ident().expect("expected ident");
        let value = meta.value();

        if value.is_ok() {
            let mut expr: Expr = meta.input.parse()?;

            // replaces occurrences of self with obj
            ReplaceSelf.visit_expr_mut(&mut expr);

            if inner {
                chained_calls.extend(quote! {
                    .inner(|obj| obj.#ident(#expr))
                })
            } else {
                chained_calls.extend(quote! {
                    .#ident(#expr)
                });
            }
        } else if inner {
            chained_calls.extend(quote! {
                .inner(|obj| obj.#ident(true))

            });
        } else {
            chained_calls.extend(quote! {
                .#ident(true)
            });
        }

        Ok(())
    };

    if let Some(param_attr) = param_attr {
        let res = param_attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("inner") {
                meta.parse_nested_meta(|meta| add_chained_call(meta, true))
            } else {
                add_chained_call(meta, false)
            }
        });

        match res {
            Ok(_) => {}
            Err(err) => {
                panic!("failed to parse param attribute {err}");
            }
        }
    }

    chained_calls
}

fn process_fields(
    fields: Fields,
    parent_type: &Ident,
    parent_var: &Ident,
) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();

    let fields = match fields {
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
        Fields::Named(FieldsNamed { named, .. }) => named,
        Fields::Unit => {
            return quote! { false };
        }
    };

    for (idx, field) in fields.into_iter().enumerate() {
        let (field_name, field_access) = match field.ident {
            Some(ident) => (ident.clone(), quote!(#ident)),
            None => {
                let i = Index::from(idx);
                (format_ident!("field_{}", idx), quote!(#i))
            }
        };

        let field_type = field.ty;

        if has_skip_attr(&field.attrs) {
            continue;
        }

        let chained_call = chained_call_for_attrs(&field.attrs);
        let formatted_label = label_from_ident(&field_name);

        output.extend(quote! {
            (
                &|ui, obj| {
                    <#field_type as ::whiskers::prelude::whiskers_widgets::WidgetMapper<#field_type>>::Type::default()
                        #chained_call
                        .ui(ui, #formatted_label, &mut obj.#field_access)

                },
                &<#field_type as ::whiskers::prelude::whiskers_widgets::WidgetMapper<#field_type>>::Type::use_grid,
            ),
        });
    }

    // This is the magic UI code that handles `whiskers_widgets::Widget::use_grid()`. It works as
    // follows:
    // - An array of closure tuple are created for all fields. The first closure is the actual UI
    //   code, the second is a predicate that returns whether the grid should be used.
    // - The array is then walked, and contiguous stretches of tuple for which the predicate returns
    //   `true` grouped together and rendered in a grid.
    quote! {
        {
            let array: &[(
                &dyn Fn(&mut ::whiskers::exports::egui::Ui, &mut #parent_type) -> bool, // ui code
                &dyn Fn() -> bool                                  // use grid predicate
            )] = &[
                #output
            ];

            let mut cur_index = 0;
            let mut changed = false;

            while cur_index < array.len() {
                if array[cur_index].1() {
                    ::whiskers::exports::egui::Grid::new(cur_index)
                        .num_columns(2)
                        .show(ui, |ui| {
                            while cur_index < array.len() && array[cur_index].1() {
                                changed = (array[cur_index].0)(ui, #parent_var) || changed;
                                ui.end_row();
                                cur_index += 1;
                            }
                        });
                }

                while cur_index < array.len() && !array[cur_index].1() {
                    changed = (array[cur_index].0)(ui, #parent_var) || changed;
                    cur_index += 1;
                }
            }

            changed
        }
    }
}

/// Expression visitor to replace `self` with `obj`.
struct ReplaceSelf;

impl VisitMut for ReplaceSelf {
    fn visit_expr_path_mut(&mut self, node: &mut ExprPath) {
        if node.path.is_ident("self") {
            *node = parse_quote!(obj);
        }
    }
}
