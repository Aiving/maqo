use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    Data, DeriveInput, Ident, LitBool, LitInt, Meta, Token,
};

#[proc_macro_derive(Block, attributes(block, tint, prop))]
pub fn derive_block(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut attr = input
        .attrs
        .iter()
        .find_map(|attr| match &attr.meta {
            Meta::List(meta) => {
                if meta.path.is_ident("block") {
                    meta.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                        .ok()
                        .map(|meta| {
                            meta.into_iter()
                                .flat_map(|meta| {
                                    if let Meta::NameValue(value) = meta {
                                        Some((
                                            value.path.get_ident().unwrap().to_string(),
                                            value.value,
                                        ))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<HashMap<_, _>>()
                        })
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap_or_default();

    let tints = input
        .attrs
        .into_iter()
        .filter_map(|attr| match attr.meta {
            Meta::List(meta) => {
                if meta.path.is_ident("tint") {
                    if let Ok(ident) = Ident::parse.parse2(meta.tokens.clone()) {
                        Some(ident.into_token_stream())
                    } else {
                        LitInt::parse
                            .parse2(meta.tokens)
                            .ok()
                            .map(|value| quote! { Color::from_hex(#value) })
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let name = input.ident;

    let Data::Struct(data) = input.data else {
        return quote! {
            compile_error!("this macro can be derived on struct's");
        }
        .into();
    };

    let mut properties = Vec::new();

    for field in data.fields {
        let Some(ident) = field.ident else {
            return quote! {
                compile_error!("this macro can be derived on struct's");
            }
            .into();
        };

        let mut attr = field
            .attrs
            .into_iter()
            .find_map(|attr| match &attr.meta {
                Meta::List(meta) => {
                    if meta.path.is_ident("prop") {
                        meta.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                            .ok()
                            .map(|meta| {
                                meta.into_iter()
                                    .flat_map(|meta| {
                                        if let Meta::NameValue(value) = meta {
                                            Some((
                                                value.path.get_ident().unwrap().to_string(),
                                                value.value,
                                            ))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<HashMap<_, _>>()
                            })
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap_or_default();

        let name = ident.to_string();
        let ty = field.ty;

        if let Some(default) = attr.remove("default") {
            properties.push(quote! {
                (#name.into(), (#default).into())
            });
        } else {
            properties.push(quote! {
                (#name.into(), #ty ::default().into())
            });
        }
    }

    let is_full_block = if let Some(value) = attr
        .remove("full_block")
        .and_then(|value| LitBool::parse.parse2(value.into_token_stream()).ok())
    {
        value.value
    } else {
        false
    };

    let is_translucent = if let Some(value) = attr
        .remove("translucent")
        .and_then(|value| LitBool::parse.parse2(value.into_token_stream()).ok())
    {
        value.value
    } else {
        false
    };

    let is_full_cube = if let Some(value) = attr
        .remove("full_cube")
        .and_then(|value| LitBool::parse.parse2(value.into_token_stream()).ok())
    {
        let value = value.value;

        quote! {
            fn is_full_cube(&self) -> bool {
                #value
            }
        }
    } else {
        TokenStream2::new()
    };

    let is_opaque_cube = if let Some(value) = attr
        .remove("opaque_cube")
        .and_then(|value| LitBool::parse.parse2(value.into_token_stream()).ok())
    {
        let value = value.value;

        quote! {
            fn is_opaque_cube(&self) -> bool {
                #value
            }
        }
    } else {
        TokenStream2::new()
    };

    let properties = if properties.is_empty() {
        quote! {
            HashMap::default()
        }
    } else {
        quote! {
            HashMap::from_iter([
                #(#properties)*,
            ])
        }
    };

    let tints = if tints.is_empty() {
        TokenStream2::new()
    } else {
        quote! {
            fn tints(&self) -> Vec<Color> {
                vec![#(#tints)*,]
            }
        }
    };

    quote! {
        impl Block for #name {
            fn properties(&self) -> HashMap<String, Property> {
                #properties
            }

            fn is_full_block(&self) -> bool {
                #is_full_block
            }

            fn is_translucent(&self) -> bool {
                #is_translucent
            }

            #is_full_cube

            #is_opaque_cube

            #tints
        }
    }
    .into()
}
