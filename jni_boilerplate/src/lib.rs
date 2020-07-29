extern crate proc_macro;
#[macro_use]
extern crate syn;
extern crate jni_boilerplate_helper;
extern crate proc_macro2;
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use std::any::Any;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{ReturnType, Type, TypeBareFn};

use jni_boilerplate_helper::{
    jni_boilerplate_constructor_invocation, jni_boilerplate_instance_method_invocation,
    jni_boilerplate_unwrapped_instance_method_invocation,
};

//

struct Arguments {
    rust_name: Ident,
    java_name: Ident,
    signature: TypeBareFn,
}

impl Parse for Arguments {
    fn parse(tokens: ParseStream) -> Result<Arguments, syn::Error> {
        let rust_name: Ident = tokens.parse()?;
        let java_name: Ident = if tokens.peek(Token![=]) {
            let _eq: Token![=] = tokens.parse()?;
            tokens.parse()?
        } else {
            rust_name.clone()
        };
        let _comma: Token![,] = tokens.parse()?;
        let signature: TypeBareFn = tokens.parse()?;

        Ok(Arguments {
            rust_name,
            java_name,
            signature,
        })
    }
}

//

#[proc_macro]
pub fn jni_instance_method(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as Arguments);

    let rust_name = macro_args.rust_name.to_string();
    let java_name = macro_args.java_name.to_string();

    let argument_types: Vec<String> = macro_args
        .signature
        .inputs
        .iter()
        .map(|arg_type| type_to_string(&arg_type.ty))
        .collect();

    let return_type_str: Option<String> = match &macro_args.signature.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(type_to_string(&ty)),
    };

    let body = jni_boilerplate_instance_method_invocation(
        &rust_name,
        &java_name,
        &argument_types,
        &return_type_str,
    );

    body.parse().unwrap()
}

#[proc_macro]
pub fn jni_unwrapped_instance_method(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as Arguments);

    let rust_name = macro_args.rust_name.to_string();
    let java_name = macro_args.java_name.to_string();

    let argument_types: Vec<String> = macro_args
        .signature
        .inputs
        .iter()
        .map(|arg_type| type_to_string(&arg_type.ty))
        .collect();

    let return_type_str: Option<String> = match &macro_args.signature.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(type_to_string(&ty)),
    };

    let body = jni_boilerplate_unwrapped_instance_method_invocation(
        &rust_name,
        &java_name,
        &argument_types,
        &return_type_str,
    );

    body.parse().unwrap()
}

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => path_segments_to_string(&type_path.path),
        Type::Reference(reference) => String::from("&") + &type_to_string(&reference.elem),
        Type::Slice(array) => {
            //println!("{:?}", ty.type_id());
            String::from("[") + &type_to_string(&array.elem) + "]"
        }
        _ => panic!("unhandled variant of Type {:?}", ty.type_id()),
    }
}

fn path_segments_to_string(path1: &syn::Path) -> String {
    let prefix: String = match path1.leading_colon {
        Some(_) => String::from("::"),
        None => String::new(),
    };

    path1.segments.iter().fold(prefix, |mut acc, v| {
        if !acc.is_empty() {
            acc.push_str("::")
        }
        acc.push_str(&v.ident.to_string());
        acc
    })
}

//

struct ConstructorMacroArgs {
    pub class_name: String,
    pub constructor_name: String,
    pub argument_types: Vec<Type>,
}

impl Parse for ConstructorMacroArgs {
    fn parse(tokens: ParseStream) -> Result<ConstructorMacroArgs, syn::Error> {
        let mut class_name = String::new();

        loop {
            let ident: Ident = tokens.parse()?;
            class_name.push_str(&ident.to_string());
            if tokens.peek(Token![.]) {
                let _dot: Token![.] = tokens.parse()?;
                class_name.push_str("/"); // yeah, JNI is weird
            } else {
                break;
            }
        }

        let constructor_name = if tokens.peek(Token![=]) {
            let _eq: Token![=] = tokens.parse()?;
            let cn: Ident = tokens.parse()?;
            cn.to_string()
        } else {
            String::from("new")
        };

        let mut argument_types: Vec<Type> = Vec::new();

        let arg_types: ParseBuffer;
        parenthesized!(arg_types in tokens);

        while !arg_types.is_empty() {
            let arg_type: Type = arg_types.parse()?;
            argument_types.push(arg_type);

            if !arg_types.is_empty() {
                let _comma: Token![,] = arg_types.parse()?;
            }
        }

        Ok(ConstructorMacroArgs {
            class_name,
            constructor_name,
            argument_types,
        })
    }
}

#[proc_macro]
pub fn jni_constructor(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as ConstructorMacroArgs);

    let constructor_name: &str = &macro_args.constructor_name;
    let argument_types: Vec<String> = macro_args
        .argument_types
        .iter()
        .map(|ty| type_to_string(ty))
        .collect();

    let class_name: &str = &macro_args.class_name;

    let body =
        jni_boilerplate_constructor_invocation(class_name, constructor_name, &argument_types);

    body.parse().unwrap()
}
