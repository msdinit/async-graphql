use crate::args;
use crate::utils::{check_reserved_name, get_crate_name, get_rustdoc};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result};

pub fn generate(directive_args: &args::Directive, input: &mut DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(directive_args.internal);
    let ident = &input.ident;
    let gql_typename = directive_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());
    check_reserved_name(&gql_typename, directive_args.internal)?;

    let desc = directive_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&input.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let s = match &mut input.data {
        Data::Struct(e) => e,
        _ => return Err(Error::new_spanned(input, "It should be a struct")),
    };

    let fields = match &mut s.fields {
        Fields::Named(fields) => Some(fields),
        Fields::Unit => None,
        _ => return Err(Error::new_spanned(input, "All fields must be named.")),
    };

    let mut schema_args = Vec::new();

    if let Some(fields) = fields {
        for item in &mut fields.named {
            let ty = &item.ty;
            let arg = args::Argument::parse(&item.attrs)?;
            let arg_name = arg
                .name
                .clone()
                .unwrap_or_else(|| item.ident.as_ref().unwrap().to_string().to_camel_case());
            let arg_desc = arg
                .desc
                .as_ref()
                .map(|s| quote! {Some(#s)})
                .unwrap_or_else(|| quote! {None});
            let schema_default = arg.default
                .as_ref()
                .map(|value| {
                    quote! {Some( <#ty as #crate_name::InputValueType>::to_value(&#value).to_string() )}
                })
                .unwrap_or_else(|| quote! {None});

            schema_args.push(quote! {
                args.insert(#arg_name, #crate_name::registry::MetaInputValue {
                    name: #arg_name,
                    description: #arg_desc,
                    ty: <#ty as #crate_name::Type>::create_type_info(registry),
                    default_value: #schema_default,
                });
            });
        }
    }

    let expanded = quote! {
        #input

        impl #crate_name::Directive for #ident {
            fn create_type_info(registry: &mut #crate_name::registry::Registry, location: #crate_name::__DirectiveLocation) {
                let directive = #crate_name::registry::MetaDirective {
                    name: #gql_typename,
                    description: #desc,
                    locations: Vec::new(),
                    args: {
                        let mut args = #crate_name::indexmap::IndexMap::new();
                        #(#schema_args)*
                        args
                    }
                };
                registry.create_directive(directive, location);
            }
        }
    };
    Ok(expanded.into())
}
