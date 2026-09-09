extern crate proc_macro;
use core::panic;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, FnArg, ItemFn, Pat, Path, ReturnType, Type, parse::Parser, parse_macro_input};

/// Create a new button handler
///
/// This macro signifies that a function should handle when a discord button is pressed.
/// The name of the function is used to identify the button and thus should be unique from other
/// buttons handlers.
///
/// The function should have the following signature:
/// ```
/// async fn my_button(
///     ctx: &poise::serenity_prelude::Context,
///     interaction: &poise::serenity_prelude::ComponentInteraction,
///     // any number of additional arguments of any serde serailizable  type (these will be passed in when the button is created)
/// ) -> Result<(), E> { // This can also return nothing
///     Ok(())
/// }
/// ```
///
/// The additional fields will be json serialized and inserted into the button id.
/// The button id has the following format `{function_name};{json_serialized_fields}`.
/// Note: Discord has a 100 character limit on button ids so it is best to keep the fields
/// to a minimum and avoid long strings. If more fields are needed, consider using a database
/// to store that data and storing a Uuid in the button label. Alternatively, the message id
/// can be used as the identifier for the database.
/// When the button is pressed, the fields will be deserialized and passed into the function.
///
#[proc_macro_attribute]
pub fn dbutton(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let signature = &input.sig;
    let fn_ident = &input.sig.ident;
    let fn_name_string = fn_ident.to_string();

    // Collect field names and types for the new struct
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let framework_input = signature
        .inputs
        .get(2)
        .expect("Expected at least 4 arguments (ctx, interaction, framework, state)");
    let framework_type = if let FnArg::Typed(pat_type) = framework_input {
        &pat_type.ty
    } else {
        panic!("Expected the third argument to be the framework context");
    };
    let state_input = signature
        .inputs
        .get(3)
        .expect("Expected at least 4 arguments (ctx, interaction, framework, state)");
    let state_type = if let FnArg::Typed(pat_type) = state_input {
        &pat_type.ty
    } else {
        panic!("Expected the fourth argument to be the state context");
    };
    for arg in signature.inputs.iter().skip(4) {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = &*pat_type.pat
        {
            field_names.push(&pat_ident.ident);
            field_types.push(&pat_type.ty);
        }
    }

    // generated names
    let handler_ident = format_ident!("__handler_{}", fn_ident);
    let label_ident = format_ident!("__label_{}", fn_ident);
    let constructor_ident = format_ident!("__constructor_{}", fn_ident);

    let error_info = extract_error_info(&input);
    // error_info: (is_result, error_type)
    let (is_result, error_type) = error_info;

    // Build the call-expression: if the original function returned Result, return it directly;
    // otherwise call the function and return Ok(()).
    let execute_body = if is_result {
        quote! {
            #fn_ident(
                ctx,
                interaction,
                framework,
                state,
                #(#field_names),*
            ).await
        }
    } else {
        quote! {
            #fn_ident(
                ctx,
                interaction,
                framework,
                state,
                #(#field_names),*
            ).await;
            Ok(())
        }
    };

    quote! {
        #input
        #[allow(non_camel_case_types)]
        #[derive(wincode::SchemaWrite, wincode::SchemaRead)]
        pub struct #handler_ident {
            #(
                pub #field_names: #field_types,
            )*
        }

        impl #handler_ident {
            pub fn new(
                #(
                    #field_names: #field_types,
                )*
            ) -> Self {
                Self {
                    #(
                        #field_names,
                    )*
                }
            }

            pub fn serialize_to_string(&self) -> String {
                use base64::Engine;
                let raw_data = wincode::serialize(&self).expect("Failed to serialize button handler");
                base64::prelude::BASE64_STANDARD.encode(&raw_data)
            }

            pub async fn execute(
                self,
                ctx: &poise::serenity_prelude::Context,
                interaction: &poise::serenity_prelude::ComponentInteraction,
                framework: #framework_type,
                state: #state_type,
            ) -> Result<(), #error_type> {
                let #handler_ident {
                    #(
                        #field_names,
                    )*
                } = self;

                #execute_body
            }
        }
        #[allow(non_upper_case_globals)]
        pub const #label_ident: &str = #fn_name_string;

        pub fn #constructor_ident(data: &[u8]) -> #handler_ident {
            wincode::deserialize(data).expect("Failed to deserialize button handler data")
        }
    }.into()
}

#[proc_macro]
pub fn create_dbutton(attr: TokenStream) -> TokenStream {
    let mut args = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
        .parse2(attr.into())
        .unwrap()
        .into_iter();

    let fn_expr = args
        .next()
        .expect("Expected at least a function identifier as the first argument");

    let (handler_path, label_path) = if let Expr::Path(fn_expr_path) = fn_expr {
        let mut handler_path = fn_expr_path.clone();
        handler_path
            .path
            .segments
            .last_mut()
            .expect("Expected a function identifier as the first argument")
            .ident = format_ident!(
            "__handler_{}",
            handler_path.path.segments.last().unwrap().ident
        );

        let mut label_path = fn_expr_path.clone();
        label_path
            .path
            .segments
            .last_mut()
            .expect("Expected a function identifier as the first argument")
            .ident = format_ident!("__label_{}", label_path.path.segments.last().unwrap().ident);

        (handler_path, label_path)
    } else {
        panic!("Expected the first argument to be a function identifier");
    };

    quote! {
        {
            let handler = #handler_path::new(
                #(
                    #args,
                )*
            );

            let button_data = handler.serialize_to_string();

            let button_tag = format!(
                "{};{}",
                #label_path,
                button_data
            );

            poise::serenity_prelude::CreateButton::new(button_tag)
        }
    }
    .into()
}

#[proc_macro]
pub fn dbutton_handler(input: TokenStream) -> TokenStream {
    // let input = parse_macro_input!(attr as AttributeArgs);
    // let handler_ident = format_ident!("__handler_{}", &input);
    let fn_path = parse_macro_input!(input as Path);

    let mut constructor_path = fn_path.clone();
    constructor_path
        .segments
        .last_mut()
        .expect("Expected a function identifier as the first argument")
        .ident = format_ident!(
        "__constructor_{}",
        constructor_path.segments.last().unwrap().ident
    );

    let mut label_path = fn_path.clone();
    label_path
        .segments
        .last_mut()
        .expect("Expected a function identifier as the first argument")
        .ident = format_ident!("__label_{}", label_path.segments.last().unwrap().ident);

    quote! {
        (#label_path, #constructor_path)
    }
    .into()
}

#[proc_macro_attribute]
pub fn event_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    // let (label_path, constructor_path) = parse_macro_input!(attr as (Path, Path));

    let button_handler_paths =
        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
            .parse2(attr.into())
            .unwrap()
            .into_iter();

    let (label_paths, constructor_paths): (Vec<_>, Vec<_>) = button_handler_paths
        .map(|path| {
            let mut constructor_path = path.clone();
            constructor_path
                .segments
                .last_mut()
                .expect("Expected a function identifier as the first argument")
                .ident = format_ident!(
                "__constructor_{}",
                constructor_path.segments.last().unwrap().ident
            );

            let mut label_path = path.clone();
            label_path
                .segments
                .last_mut()
                .expect("Expected a function identifier as the first argument")
                .ident = format_ident!("__label_{}", label_path.segments.last().unwrap().ident);

            (label_path, constructor_path)
        })
        .unzip();

    let input_fn = parse_macro_input!(item as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input_fn;
    let stmts = &block.stmts;

    let output = quote! {
        #(#attrs)* #vis #sig {

            if let poise::serenity_prelude::FullEvent::InteractionCreate { interaction} = event {
                if let poise::serenity_prelude::Interaction::Component(component) = interaction {
                    let custom_id = &component.data.custom_id;
                    if let Some((button_id, serialized_data)) = custom_id.split_once(';') {
                        use base64::Engine;
                        let raw_data = base64::prelude::BASE64_STANDARD.decode(serialized_data).expect("Failed to decode button data");
                        match button_id {
                            #(
                            #label_paths => {
                                let handler = #constructor_paths(&raw_data);
                                return handler.execute(
                                    ctx,
                                    &component,
                                    framework,
                                    state
                                ).await;
                            }
                            )*
                            _ => {}
                        }

                    }
                }

            }
            #(#stmts)*
        }
    };

    output.into()
}

fn extract_error_info(input_fn: &ItemFn) -> (bool, Type) {
    // Returns (is_result, error_type). If not a Result return, is_result=false and error_type=() .
    if let ReturnType::Type(_, ref ret_type) = input_fn.sig.output
        && let Type::Path(type_path) = &**ret_type
        && let Some(last_segment) = type_path.path.segments.last()
        && last_segment.ident == "Result"
        && let syn::PathArguments::AngleBracketed(angle_bracketed_args) = &last_segment.arguments
        && angle_bracketed_args.args.len() == 2
        && let syn::GenericArgument::Type(err_type) = &angle_bracketed_args.args[1]
    {
        return (true, err_type.clone());
    }
    (false, syn::parse_quote! { () })
}
