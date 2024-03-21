use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{self};

#[proc_macro_derive(Builder, attributes(builder))]
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

    struct FieldData {
        id: syn::Ident,
        ty: syn::Type,
        nested_ty: std::option::Option<(String, syn::Type)>,
        each_id: std::option::Option<syn::Ident>,
    }

    // for field in fields.iter() {
    //     let id = field.ident.clone().unwrap();
    //     let v = try_parse_builder_each(field);
    //     let ty = field.ty.clone();
    //     println!("{id:#}: {v:?}, {ty:?}");
    // }

    let each_builder_fields = fields.iter().map(|field| {
        let id = field.ident.clone().unwrap();

        if let Ok(Some(each)) = try_parse_builder_each(field) {
            println!("============{each}===========");
            quote! {
                pub fn #each(&mut self, #each: String) -> &mut Self {
                    self.#id.push(#each);
                    self
                }
            }
        } else {
            println!("============fail===========");
            quote! {}
        }
    });

    // println!("{:#?}", fields);

    let builder_ident = format_ident!("{ident}Builder");

    let builder_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = try_optional(&field.ty).or(std::option::Option::Some(field.ty));

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

        if try_optional(&field.ty).is_none() {
            quote! {
                if self.#id.is_none() {
                    return std::result::Result::Err(#err.into());
                }
            }
        } else {
            quote! {}
        }
    });

    let build_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();

        if try_optional(&field.ty).is_some() {
            quote! {
                #id: self.#id.clone()
            }
        } else {
            quote! {
                #id: self.#id.clone().unwrap()
            }
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

        pub struct test;

        impl #builder_ident {
            #(#setters)*

            pub struct test2;
            #(#each_builder_fields,)*
            pub struct test3;

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
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) if segments.len() == 1 => segments.clone(),
        _ => return std::option::Option::None,
    };

    let args = match &segments[0] {
        syn::PathSegment {
            ident,
            arguments:
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }),
        } if ident == "Option" && args.len() == 1 => args,
        _ => return std::option::Option::None,
    };

    let ty = match &args[0] {
        syn::GenericArgument::Type(t) => t,
        _ => return std::option::Option::None,
    };

    Some(ty.clone())
}

fn try_parse_builder_each(
    field: &syn::Field,
) -> std::result::Result<std::option::Option<String>, syn::Error> {
    println!("try_parse_builder_each");
    for attr in field.attrs.iter() {
        if attr.path().is_ident("builder") {
            let mut value: String = String::new();
            match attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("each") {
                    let s: syn::LitStr = meta.value()?.parse()?;
                    value = s.value();
                    return std::result::Result::Ok(());
                } else {
                    return std::result::Result::Err(meta.error("Unrecognized attribute"));
                }
            }) {
                Err(e) => return std::result::Result::Err(e),
                Ok(()) => return std::result::Result::Ok(std::option::Option::Some(value)),
            }
        }
    }

    return std::result::Result::Ok(std::option::Option::None);
}
