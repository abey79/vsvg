use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, visit_mut::VisitMut, Data, DataStruct, DeriveInput, Expr,
    ExprPath, Fields, FieldsNamed, FieldsUnnamed, Index,
};

fn format_label(label: &str) -> String {
    format!("{}:", label.to_case(Case::Lower))
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

    let fields_ui = match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            process_fields(fields, &name, &format_ident!("value"))
        }
        _ => panic!("The Sketch derive macro only supports structs"),
    };

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

fn process_fields(
    fields: Fields,
    parent_type: &Ident,
    parent_var: &Ident,
) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();

    let fields = match fields {
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
        Fields::Named(FieldsNamed { named, .. }) => named,
        _ => panic!("The Sketch derive macro only supports named-field structs"),
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
