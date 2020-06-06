use crate::args;
use crate::utils::get_crate_name;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result};

pub fn generate(directive_args: &args::Directive, input: &mut DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(directive_args.internal);
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
    };
    Ok(expanded.into())
}
