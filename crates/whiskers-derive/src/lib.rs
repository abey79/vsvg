use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, parse_quote, visit_mut::VisitMut, Data, DataEnum, DataStruct, DeriveInput,
    Expr, ExprPath, Field, Fields, FieldsNamed, FieldsUnnamed, Index, Variant,
};

fn format_label(label: &str) -> String {
    format!("{}:", label.to_case(Case::Lower))
}

/// Attribute macro to automatically derive some of the required traits for a sketch app.
///
/// This is equivalent to:
/// ```ignore
/// #[derive(Sketch, serde::Serialize, serde::Deserialize)]
/// #[serde(crate = "::whiskers::prelude::serde")]
/// ```
#[proc_macro_attribute]
pub fn sketch_app(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let expanded = quote! {
        #[derive(Sketch, serde::Serialize, serde::Deserialize)]
        #[serde(crate = "::whiskers::prelude::serde")]
        #ast
    };

    TokenStream::from(expanded)
}

/// Attribute macro to automatically derive some of the required traits for a sketch widget.
///
/// This is equivalent to:
/// ```ignore
/// #[derive(Widget, serde::Serialize, serde::Deserialize)]
/// #[serde(crate = "::whiskers::prelude::serde")]
/// ```
#[proc_macro_attribute]
pub fn sketch_widget(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let expanded = quote! {
        #[derive(Widget, serde::Serialize, serde::Deserialize)]
        #[serde(crate = "::whiskers::prelude::serde")]
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
        impl ::whiskers::SketchApp for #name {
            fn name(&self) -> String {
                stringify!(#name).to_string()
            }

            fn ui(&mut self, ui: &mut ::whiskers::prelude::egui::Ui) -> bool {
                #fields_ui
            }
        }
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
    let fields_ui = process_fields(fields, &name, &format_ident!("value"));

    TokenStream::from(quote! {
        #[derive(Default)]
        pub struct #widget_name;

        impl ::whiskers::widgets::Widget<#name> for #widget_name {
            fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut #name) -> bool {
                ::whiskers::collapsing_header(ui, label.trim_end_matches(':'), "", true, |ui|{
                        #fields_ui
                    })
                    .unwrap_or(false)
            }

            fn use_grid() -> bool {
                false
            }
        }

        impl ::whiskers::widgets::WidgetMapper<#name> for #name {
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
    for Variant { ident, fields, .. } in variants.iter() {
        let func_ident = default_function_name_for_variant(ident);

        let fields_defaults = match fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let fields = field_defaults(unnamed.iter());
                quote! {( #fields )}
            }
            Fields::Named(FieldsNamed { named, .. }) => {
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
    // Create the UI code to display the enum.
    //

    let idents = variants
        .iter()
        .map(|Variant { ident, .. }| ident.clone())
        .collect::<Vec<_>>();
    let ident_default_functions = idents
        .iter()
        .map(|ident| default_function_name_for_variant(ident))
        .collect::<Vec<_>>();
    let ident_strings = idents
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<_>>();

    let name_string = name.to_string();

    let combobox_ui_code = quote! {
        let mut selected_text = match value {
            #(
                #name::#idents => #ident_strings,
            )*
        }.to_owned();
        let initial_selected_text = selected_text.clone();
        ui.horizontal(|ui|{
            ui.label(label);
            egui::ComboBox::from_id_source(#name_string).selected_text(&selected_text).show_ui(ui, |ui| {
                #(
                    ui.selectable_value(&mut selected_text, #ident_strings.to_owned(), #ident_strings);
                )*
            });
        });

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

    //let mut fields_ui_for_ident: HashMap<Ident, proc_macro2::TokenStream> = HashMap::new();
    let mut idents = Vec::new();
    let mut field_uis = Vec::new();
    for Variant { ident, fields, .. } in variants.into_iter() {
        let fields_ui = process_fields(fields, &name, &format_ident!("value"));
        //fields_ui_for_ident.insert(ident, fields_ui);
        idents.push(ident);
        field_uis.push(fields_ui);
    }

    TokenStream::from(quote! {
        #impl_default_functions

        #[derive(Default)]
        pub struct #widget_name;

        impl ::whiskers::widgets::Widget<#name> for #widget_name {
            fn ui(&self, ui: &mut egui::Ui, label: &str, value: &mut #name) -> bool {

                #combobox_ui_code

                changed |= match value {
                    #(
                        #name::#idents => {
                             #field_uis
                        }
                    )*
                };

                changed
            }

            fn use_grid() -> bool {
                false
            }
        }

        impl ::whiskers::widgets::WidgetMapper<#name> for #name {
            type Type = #widget_name;
        }
    })
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
        let label = field_name.to_string();

        let skip_attr = field.attrs.iter().find(|attr| attr.path().is_ident("skip"));
        if skip_attr.is_some() {
            continue;
        }

        let param_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("param"));

        let mut chained_calls = proc_macro2::TokenStream::new();

        if let Some(param_attr) = param_attr {
            let res = param_attr.parse_nested_meta(|meta| {
                let ident = meta.path.get_ident().expect("expected ident");
                let value = meta.value();

                if value.is_ok() {
                    let mut expr: Expr = meta.input.parse()?;

                    // replaces occurrences of self with obj
                    ReplaceSelf.visit_expr_mut(&mut expr);

                    chained_calls.extend(quote! {
                        .#ident(#expr)
                    });
                } else {
                    chained_calls.extend(quote! {
                        .#ident(true)
                    });
                }

                Ok(())
            });

            if res.is_err() {
                panic!("failed to parse param attribute");
            }
        }

        let formatted_label = format_label(&label);

        output.extend(quote! {
            (
                &|ui, obj| {
                    <#field_type as ::whiskers::widgets::WidgetMapper<#field_type>>::Type::default()
                        #chained_calls
                        .ui(ui, #formatted_label, &mut obj.#field_access)

                },
                &<#field_type as ::whiskers::widgets::WidgetMapper<#field_type>>::Type::use_grid,
            ),
        });
    }

    // This is the magic UI code that handles `whiskers::widgets::Widget::use_grid()`. It works as
    // follows:
    // - An array of closure tuple are created for all fields. The first closure is the actual UI
    //   code, the second is a predicate that returns whether the grid should be used.
    // - The array is then walked, and contiguous stretches of tuple for which the predicate returns
    //   `true` grouped together and rendered in a grid.
    quote! {
        {
            let array: &[(
                &dyn Fn(&mut egui::Ui, &mut #parent_type) -> bool, // ui code
                &dyn Fn() -> bool                                  // use grid predicate
            )] = &[
                #output
            ];

            let mut cur_index = 0;
            let mut changed = false;

            while cur_index < array.len() {
                if array[cur_index].1() {
                    egui::Grid::new(cur_index)
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
