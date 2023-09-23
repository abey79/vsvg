use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Expr, Fields, FieldsNamed};

#[proc_macro_derive(Sketch, attributes(param, skip))]
pub fn sketch_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = input.ident;

    let mut ui_func_tokens = proc_macro2::TokenStream::new();

    match input.data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                for field in named {
                    let field_name = field.ident.unwrap();
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
                                let expr: Expr = meta.input.parse()?;
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

                    ui_func_tokens.extend(quote! {
                        <#field_type as ::vsvg_sketch::widgets::WidgetMapper<#field_type>>::Type::default()
                            #chained_calls
                            .ui(ui, #label, &mut self.#field_name);
                    });
                }
            }
            _ => panic!("The Sketch derive macro only supports named-field structs"),
        },
        _ => panic!("The Sketch derive macro only supports structs"),
    }

    TokenStream::from(quote! {
        impl ::vsvg_sketch::SketchApp for #name { }

        impl ::vsvg_sketch::SketchUI for #name {
            fn ui(&mut self, ui: &mut egui::Ui) {
                #ui_func_tokens
            }
        }
    })
}
