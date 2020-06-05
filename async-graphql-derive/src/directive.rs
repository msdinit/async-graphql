use crate::args;
use crate::utils::{check_reserved_name, get_crate_name, get_rustdoc};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{Error, FnArg, ImplItem, ItemImpl, Pat, Result, Type, TypeReference};

pub fn generate(directive_args: &args::Directive, item_impl: &mut ItemImpl) -> Result<TokenStream> {
    let crate_name = get_crate_name(directive_args.internal);
    let (self_ty, self_name) = match item_impl.self_ty.as_ref() {
        Type::Path(path) => (
            path,
            path.path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap(),
        ),
        _ => return Err(Error::new_spanned(&item_impl.self_ty, "Invalid type")),
    };
    let generics = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;

    let gql_typename = directive_args
        .name
        .clone()
        .unwrap_or_else(|| self_name.clone());
    check_reserved_name(&gql_typename, directive_args.internal)?;

    let desc = directive_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&item_impl.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});
    let mut impls = HashMap::new();

    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            if method.sig.asyncness.is_none() {
                return Err(Error::new_spanned(&method, "Must be asynchronous"));
            }

            let mut create_ctx = true;
            let mut args = Vec::new();

            for (idx, arg) in method.sig.inputs.iter_mut().enumerate() {
                if let FnArg::Receiver(receiver) = arg {
                    if idx != 0 {
                        return Err(Error::new_spanned(
                            receiver,
                            "The self receiver must be the first parameter.",
                        ));
                    }
                } else if let FnArg::Typed(pat) = arg {
                    if idx == 0 {
                        return Err(Error::new_spanned(
                            pat,
                            "The self receiver must be the first parameter.",
                        ));
                    }

                    match (&*pat.pat, &*pat.ty) {
                        (Pat::Ident(arg_ident), Type::Path(arg_ty)) => {
                            args.push((
                                arg_ident.clone(),
                                arg_ty.clone(),
                                args::Argument::parse(&crate_name, &pat.attrs)?,
                            ));
                            pat.attrs.clear();
                        }
                        (arg, Type::Reference(TypeReference { elem, .. })) => {
                            if let Type::Path(path) = elem.as_ref() {
                                if idx != 1
                                    || path.path.segments.last().unwrap().ident
                                        != "ContextDirective"
                                {
                                    return Err(Error::new_spanned(
                                        arg,
                                        "The ContextDirective must be the second argument.",
                                    ));
                                } else {
                                    create_ctx = false;
                                }
                            }
                        }
                        _ => return Err(Error::new_spanned(arg, "Invalid argument type.")),
                    }
                }
            }

            if create_ctx {
                let arg = syn::parse2::<FnArg>(quote! { _: &#crate_name::Context<'_> }).unwrap();
                method.sig.inputs.insert(1, arg);
            }

            let mut schema_args = Vec::new();
            let mut use_params = Vec::new();
            let mut get_params = Vec::new();

            for (
                ident,
                ty,
                args::Argument {
                    name,
                    desc,
                    default,
                    validator,
                },
            ) in args
            {
                let name = name
                    .clone()
                    .unwrap_or_else(|| ident.ident.to_string().to_camel_case());
                let desc = desc
                    .as_ref()
                    .map(|s| quote! {Some(#s)})
                    .unwrap_or_else(|| quote! {None});
                let schema_default = default
                    .as_ref()
                    .map(|value| {
                        quote! {Some( <#ty as #crate_name::InputValueType>::to_value(&#value).to_string() )}
                    })
                    .unwrap_or_else(|| quote! {None});

                schema_args.push(quote! {
                    args.insert(#name, #crate_name::registry::MetaInputValue {
                        name: #name,
                        description: #desc,
                        ty: <#ty as #crate_name::Type>::create_type_info(registry),
                        default_value: #schema_default,
                        validator: #validator,
                    });
                });

                use_params.push(quote! { #ident });

                let default = match default {
                    Some(default) => quote! { Some(|| -> #ty { #default }) },
                    None => quote! { None },
                };
                get_params.push(quote! {
                    let #ident: #ty = ctx.param_value(#name, #default)?;
                });
            }

            // funcs.insert(method.sig.ident.to_string());
        }
    }

    let expanded = quote! {
        #item_impl
    };
    println!("{}", expanded);
    Ok(expanded.into())
}
