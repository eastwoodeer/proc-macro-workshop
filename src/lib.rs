use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    // eprintln!("{ast:#?}");
    let ident = ast.ident.clone();

    let fields = match ast {
        syn::DeriveInput {
            data:
                syn::Data::Struct(syn::DataStruct {
                    fields: syn::Fields::Named(syn::FieldsNamed { named: fields, .. }),
                    ..
                }),
            ..
        } => fields,
        _ => unimplemented!("derive(Builder) only supports structs with named fields"),
    };

    println!("{:#?}", fields);

    let builder_ident = format_ident!("{ident}Builder");

    let builder_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = field.ty;

        quote! {
            #id: std::option::Option<#ty>
        }
    });

    let builder_defaults = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();

        quote! {
            #id: std::option::Option::None
        }
    });

    let setters = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = try_optional(&field.ty).or(std::option::Option::Some(field.ty));

        quote! {
            pub fn #id(&mut self, v: #ty) -> &mut Self {
                self.#id = std::option::Option::Some(v);
                self
            }
        }
    });

    let build_checks = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let err = format!("{id} not found");

        quote! {
            if self.#id.is_none() {
                return std::result::Result::Err(#err.to_owned().into());
            }
        }
    });

    let build_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();

        quote! {
            #id: self.#id.clone().unwrap()
        }
    });


    let output = quote! {
        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_defaults,)*
                }
            }
        }

        pub struct #builder_ident {
            #(#builder_fields,)*
        }

        impl #builder_ident {
            #(#setters)*

            pub fn build(&mut self) -> std::result::Result<#ident, std::boxed::Box<dyn std::error::Error>> {
                #(#build_checks;)*

                std::result::Result::Ok(#ident {
                    #(#build_fields,)*
                })
            }
        }
    };

    proc_macro::TokenStream::from(output)
}

fn try_optional(ty: &syn::Type) -> std::option::Option<syn::Type> {
    let segments = match ty {
        syn::Type::Path (
            syn::TypePath {
                path: syn::Path {
                    segments,
                    ..
                },
                ..
            }
        )
        if segments.len() == 1 => segments.clone(),
        _ => return std::option::Option::None,
    };

    let args = match &segments[0] {
        syn::PathSegment {
            ident,
            arguments: syn::PathArguments::AngleBracketed(
                syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }
            )
        }
        if ident == "Option" && args.len() == 1 => args,
        _ => return std::option::Option::None,
    };

    let ty = match &args[0] {
        syn::GenericArgument::Type(t) => t,
        _ => return std::option::Option::None,
    };

    Some(ty.clone())
}
