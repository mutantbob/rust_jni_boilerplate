/*

still need to learn from

https://github.com/dtolnay/syn/blob/master/examples/lazy-static/lazy-static/src/lib.rs
 */

extern crate proc_macro;
#[macro_use]
extern crate syn;
extern crate jni_boilerplate_helper;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use proc_macro::{Span, TokenStream};
use proc_macro2::Ident;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::token::Comma;
use syn::{Expr, FnArg, Lifetime, ReturnType, Type, TypeTuple};

//

struct MySignature {
    pub parameter_types: Vec<Type>,
}

impl Parse for MySignature {
    fn parse(tokens: ParseStream) -> Result<Self, syn::Error> {
        let mut parameter_types: Vec<Type> = Vec::new();

        let arg_types: ParseBuffer;
        parenthesized!(arg_types in tokens);

        while !arg_types.is_empty() {
            let arg_type: Type = arg_types.parse()?;
            parameter_types.push(arg_type);

            if !arg_types.is_empty() {
                let _comma: Token![,] = arg_types.parse()?;
            }
        }

        Ok(MySignature { parameter_types })
    }
}

//

struct InstanceMacroArguments {
    pub lifetime_a: Lifetime,
    pub lifetime_b: Lifetime,
    rust_name: Ident,
    java_name: String,
    signature: MySignature,
    return_type: ReturnType,
}

impl Parse for InstanceMacroArguments {
    fn parse(tokens: ParseStream) -> Result<InstanceMacroArguments, syn::Error> {
        let (lifetime_a, lifetime_b) = parse_optional_lifetimes(tokens)?;
        let (rust_name, java_name) = parse_function_names(tokens)?;

        let signature: MySignature = tokens.parse()?;

        let return_type: ReturnType = tokens.parse()?;

        Ok(InstanceMacroArguments {
            lifetime_a,
            lifetime_b,
            rust_name,
            java_name,
            signature,
            return_type,
        })
    }
}

//

/// This macro is designed to be used inside the `impl` of a struct that has at least two fields.
/// `self.java_this` should be an `AutoLocal`.  `self.jni_env` should be an `&AttachGuard`.
///
/// usage:
/// <pre>jni_instance_method!{ fn_name[=java_name]([ arg_type1 [,arg_type2...]])[ ->return_type ] }
/// </pre>
#[proc_macro]
pub fn jni_instance_method(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as InstanceMacroArguments);

    let rust_name = &macro_args.rust_name;
    let java_name = &macro_args.java_name;

    let arg_types = &macro_args.signature.parameter_types;

    let args_metadata: Vec<AllAboutArg> = arg_types
        .iter()
        .enumerate()
        .map(|(i, t)| AllAboutArg::new((*t).clone(), i))
        .collect();

    let return_type = bare_type_from_return_type(&macro_args.return_type);
    //let return_type_prefix = type_prefix_from(return_type.clone());

    let arg_sig = formal_parameters_tokens(&args_metadata);

    let jni_env = self_jni_env();

    let decl: Vec<proc_macro2::TokenStream> =
        initializations_for_parameter_temporaries(&args_metadata, jni_env);

    let jvalue_param_array: Vec<proc_macro2::TokenStream> = value_parameter_array(&args_metadata);

    let body = quote! {
    pub fn #rust_name(&self, #arg_sig) -> Result<#return_type, jni::errors::Error>
    {
        use jni_boilerplate_helper::{JavaSignatureFor, ConvertRustToJValue,
                                     ConvertJValueToRust};

        #(#decl)*

        let sig = String::from("(") #(+&<#arg_types>::signature_for())* + ")"+&<#return_type>::signature_for();

        let results =
            self.jni_env.call_method(self.java_this.as_obj(), #java_name, sig,
                                     &[#(#jvalue_param_array),*])?;

        type Item=#return_type; // because of things like Vec<String>
        Item::to_rust(self.jni_env, &results)
    }
            };

    body.into()
}

fn self_jni_env() -> Expr {
    let ts: proc_macro::TokenStream = quote! { self.jni_env }.into();
    let expr: Expr = syn::parse_macro_input::parse::<Expr>(ts).expect("how could parsing fail?");
    expr
}

fn bare_jni_env() -> Expr {
    let ts: proc_macro::TokenStream = quote! { jni_env }.into();
    let expr: Expr = syn::parse_macro_input::parse::<Expr>(ts).expect("how could parsing fail?");
    expr
}

#[proc_macro]
pub fn jni_unwrapped_instance_method(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as InstanceMacroArguments);

    let lifetime_a = &macro_args.lifetime_a;
    let lifetime_b = &macro_args.lifetime_b;
    let rust_name = &macro_args.rust_name;
    let java_name = &macro_args.java_name;

    let arg_types = &macro_args.signature.parameter_types;

    let args_metadata: Vec<AllAboutArg> = arg_types
        .iter()
        .enumerate()
        .map(|(i, t)| AllAboutArg::new((*t).clone(), i))
        .collect();

    let return_type = bare_type_from_return_type(&macro_args.return_type);

    let arg_sig = formal_parameters_tokens(&args_metadata);

    let decl: Vec<proc_macro2::TokenStream> =
        initializations_for_parameter_temporaries(&args_metadata, bare_jni_env());

    let jvalue_param_array: Vec<proc_macro2::TokenStream> = value_parameter_array(&args_metadata);

    let body = quote! {
    pub fn #rust_name(jni_env:&#lifetime_b jni::JNIEnv<#lifetime_a>, java_this: &jni::objects::JObject<#lifetime_a>, #arg_sig) -> Result<#return_type, jni::errors::Error>
    {
        use jni_boilerplate_helper::{JavaSignatureFor, ConvertRustToJValue,
                                     ConvertJValueToRust};

        #(#decl)*

        let sig = String::from("(") #(+&<#arg_types>::signature_for())* + ")"+&<#return_type>::signature_for();

        let results =
            jni_env.call_method(*java_this, #java_name, sig,
                                     &[#(#jvalue_param_array),*])?;
        #return_type::to_rust(jni_env, &results)
    }
            };

    body.into()
}

//

struct ConstructorMacroArgs {
    pub lifetime_a: Lifetime,
    pub lifetime_b: Lifetime,
    pub class_name: String,
    pub constructor_name: Ident,
    pub signature: MySignature,
}

impl Parse for ConstructorMacroArgs {
    fn parse(tokens: ParseStream) -> Result<ConstructorMacroArgs, syn::Error> {
        let (lifetime_a, lifetime_b) = parse_optional_lifetimes(tokens)?;

        let ident: Ident = tokens.parse()?;

        let (constructor_name, mut class_name) = if tokens.peek(Token![=]) {
            let _eq: Token![=] = tokens.parse()?;
            let constructor_name = ident;
            let ident: Ident = tokens.parse()?;
            let class_name = ident.to_string();
            (constructor_name, class_name)
        } else {
            (simple_identifier("new"), ident.to_string())
        };
        // class name is separated by dots in java code, but by slashes in JNI lookups. *facepalm*
        loop {
            if tokens.peek(Token![.]) {
                let _dot: Token![.] = tokens.parse()?;
                class_name.push_str("/"); // yeah, JNI is weird
            } else {
                break;
            }
            let ident: Ident = tokens.parse()?;
            class_name.push_str(&ident.to_string());
        }

        let signature = tokens.parse()?;

        Ok(ConstructorMacroArgs {
            lifetime_a,
            lifetime_b,
            class_name,
            constructor_name,
            signature,
        })
    }
}

///
/// This allows you to define a class method that builds an instance of a wrapper class by calling a java constructor and wrapping the resulting JObject using the <code>Self::wrap_object()</code> method.
/// For example:
/// <pre>
/// struct Widget&lt;'a&gt; {
///     java_this: JObject&lt;'a&gt;,
/// }
/// impl&lt;'a&gt; Widget&lt;'a&gt; {
///     fn wrap_jobject(java_this: JObject&lt;'a&gt;) -&gt; Widget&lt;'a&gt;
///     {
///         Widget {
///             java_this,
///         }
///     }
///
///     // define a rust function named new
///     jni_constructor! { com.purplefrog.rust_callables.Widget () }
///     // since java supports overloaded methods and constructors while rust does not, you can name the function something other than new
///     jni_constructor! { new_one=com.purplefrog.rust_callables.Widget (&amp;str) }
/// ...
/// </pre>
///
#[proc_macro]
pub fn jni_constructor(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as ConstructorMacroArgs);

    let lifetime_a = &macro_args.lifetime_a;
    let lifetime_b = &macro_args.lifetime_b;
    let arg_types = &macro_args.signature.parameter_types;

    let args_metadata: Vec<AllAboutArg> = arg_types
        .iter()
        .enumerate()
        .map(|(i, t)| AllAboutArg::new((*t).clone(), i))
        .collect();

    let class_name: &str = &macro_args.class_name;

    let rust_name = &macro_args.constructor_name;

    let arg_sig = formal_parameters_tokens(&args_metadata);

    let decl: Vec<proc_macro2::TokenStream> =
        initializations_for_parameter_temporaries(&args_metadata, bare_jni_env());

    let jvalue_param_array: Vec<proc_macro2::TokenStream> = value_parameter_array(&args_metadata);

    let body = quote! {
    pub fn #rust_name(jni_env: &#lifetime_b  jni::AttachGuard<#lifetime_a>, #arg_sig)
    -> Result<Self, jni::errors::Error>
    {
            use jni_boilerplate_helper::{JavaSignatureFor, ConvertRustToJValue,
                                         ConvertJValueToRust, JClassWrapper, JavaConstructible};
            let cls = jni_env.find_class(#class_name)?;
            let cls = JClassWrapper {
                jni_env: &jni_env,
                cls,
            };

            #(#decl)*

            let sig = String::from("(")#(+&<#arg_types>::signature_for())* + ")V";

            let rval = jni_env.new_object(cls.cls, sig, &[#(#jvalue_param_array),*])?;
            jni_env.exception_check()?;

            Ok(Self::wrap_jobject(jni_env, jni::objects::AutoLocal::new(&jni_env, rval)))
    }
        };

    body.into()
}

//
//
//

struct StaticMethodArgs {
    lifetime_a: Lifetime,
    lifetime_b: Lifetime,
    rust_name: Ident,
    java_name: String,
    signature: MySignature,
    return_type: ReturnType,
}

impl Parse for StaticMethodArgs {
    fn parse(tokens: &ParseBuffer) -> Result<Self, syn::Error> {
        let (lifetime_a, lifetime_b) = parse_optional_lifetimes(tokens)?;

        let (rust_name, java_name) = parse_function_names(tokens)?;

        let signature = tokens.parse()?;

        //println!("do I get a return type?");

        let return_type: ReturnType = if tokens.peek(Token![->]) {
            tokens.parse()?
        } else {
            ReturnType::Default
        };

        Ok(StaticMethodArgs {
            lifetime_a,
            lifetime_b,
            rust_name,
            java_name,
            signature,
            return_type,
        })
    }
}

struct AllAboutArg {
    a_type: Type,
    p_ident: Ident,
    p_name: String,
    tmp_ident: Ident,
}

impl AllAboutArg {
    pub fn new(arg_type: Type, index: usize) -> AllAboutArg {
        let p_name = format!("arg{}", index);
        let tmp_name = format!("tmp{}", index);
        AllAboutArg {
            a_type: arg_type,
            p_ident: simple_identifier(&p_name),
            p_name,
            tmp_ident: simple_identifier(&tmp_name),
        }
    }
}

fn simple_identifier(name: &str) -> Ident {
    Ident::new(&name, Span::call_site().into())
}

#[proc_macro]
pub fn jni_static_method(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as StaticMethodArgs);

    let rust_name = &macro_args.rust_name;
    let java_name: &str = &macro_args.java_name;

    let return_type: Type = bare_type_from_return_type(&macro_args.return_type);

    let arg_types = &macro_args.signature.parameter_types;
    let lifetime_a = &macro_args.lifetime_a;
    let lifetime_b = &macro_args.lifetime_b;

    let args_metadata: Vec<AllAboutArg> = arg_types
        .iter()
        .enumerate()
        .map(|(i, t)| AllAboutArg::new((*t).clone(), i))
        .collect();

    let arg_sig = formal_parameters_tokens(&args_metadata);

    let decl: Vec<proc_macro2::TokenStream> =
        initializations_for_parameter_temporaries(&args_metadata, bare_jni_env());

    let jvalue_param_array: Vec<proc_macro2::TokenStream> = value_parameter_array(&args_metadata);

    let body = quote! {
    #[allow(non_snake_case)]
    pub fn #rust_name(jni_env: &#lifetime_b jni::JNIEnv<#lifetime_a>, #arg_sig) ->Result<#return_type, jni::errors::Error>
    {
        use jni_boilerplate_helper::{JavaSignatureFor, ConvertRustToJValue,
                                     ConvertJValueToRust};

        let cls = jni_env.find_class(&<Self>::java_class_name())?;
        let cls = JClassWrapper {
            jni_env: &jni_env,
            cls,
        };
        jni_env.exception_check()?;

        #(#decl)*
        let sig = String::from("(")+#(&<#arg_types>::signature_for())+* + ")"+&<#return_type>::signature_for();

        let results = jni_env.call_static_method(cls.cls, #java_name, sig, &[#(#jvalue_param_array),*])?;
        jni_env.exception_check()?;

        <#return_type>::to_rust(jni_env, &results)
    }
    };

    body.into()
}

fn formal_parameters_tokens(
    args_metadata: &[AllAboutArg],
) -> syn::punctuated::Punctuated<FnArg, Comma> {
    let mut arg_sig: syn::punctuated::Punctuated<FnArg, Comma> = syn::punctuated::Punctuated::new();

    for arg in args_metadata {
        let arg2 = named_function_argument(&arg.p_name, &arg.a_type);
        arg_sig.push(arg2)
    }

    arg_sig
}

///
/// You will probably use the result of this function inside a quote! macro like this:
/// <pre>&[#(#jvalue_param_array),*]</pre>
fn value_parameter_array(args_metadata: &[AllAboutArg]) -> Vec<proc_macro2::TokenStream> {
    args_metadata
        .iter()
        .map(|metadata| {
            let ty = &metadata.a_type;
            let tmp_i = &metadata.tmp_ident;
            quote! { <#ty>::temporary_into_jvalue(&#tmp_i) }
        })
        .collect()
}

fn initializations_for_parameter_temporaries(
    args_metadata: &[AllAboutArg],
    jni_env_ident: Expr,
) -> Vec<proc_macro2::TokenStream> {
    args_metadata
        .iter()
        .map(|metadata| {
            let tmp_i = &metadata.tmp_ident;
            let p_i = &metadata.p_ident;
            quote! {let #tmp_i = #p_i.into_temporary(#jni_env_ident)?;}
        })
        .collect()
}

/*
fn named_function_argument2(name: &str, arg_type: &Type) -> FnArg {
    let arg_ident: PatIdent = PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability: None,
        ident: simple_identifier(name),
        subpat: None,
    };
    let pat_type: PatType = PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(arg_ident)),
        colon_token: syn::token::Colon {
            spans: [Span::call_site().into()],
        },
        ty: Box::new((*arg_type).clone()),
    };
    let arg2: syn::FnArg = FnArg::Typed(pat_type);
    arg2
}
*/
/// returns tokens for <code>name:arg_type</code>
fn named_function_argument(name: &str, arg_type: &Type) -> FnArg {
    let id = simple_identifier(name);
    let tokens: proc_macro::TokenStream = quote! { #id:#arg_type }.into();
    //let x = parse_macro_input!(tokens as FnArg);
    ::syn::parse_macro_input::parse::<FnArg>(tokens)
        .expect("I did not expect this to be able to fail")
}

/// returns (rust_name:Ident, java_name:String)
fn parse_function_names(tokens: &ParseBuffer) -> Result<(Ident, String), syn::Error> {
    let function_name: Ident = tokens.parse()?;

    let (rust_name, java_name) = if tokens.peek(Token![=]) {
        let _eq: Token![=] = tokens.parse()?;

        let ident: Ident = tokens.parse()?;
        (function_name, ident.to_string())
    } else {
        (function_name.clone(), function_name.to_string())
    };
    Ok((rust_name, java_name))
}

fn bare_type_from_return_type(return_type: &ReturnType) -> Type {
    match return_type {
        ReturnType::Default => {
            let tt: TypeTuple = TypeTuple {
                paren_token: Default::default(),
                elems: Default::default(),
            };
            Type::Tuple(tt)
        }
        ReturnType::Type(_, rt) => {
            let rt: &Type = rt;
            (*rt).clone()
        }
    }
}

fn parse_optional_lifetimes(tokens: &ParseBuffer) -> Result<(Lifetime, Lifetime), syn::Error> {
    if tokens.peek(Lifetime) {
        let lifetime_a = tokens.parse()?;
        let _comma: Token![,] = tokens.parse()?;
        let lifetime_b = tokens.parse()?;
        let _comma: Token![,] = tokens.parse()?;
        Ok((lifetime_a, lifetime_b))
    } else {
        let wildcard: Lifetime = Lifetime::new("'_", proc_macro2::Span::call_site());
        Ok((wildcard.clone(), wildcard))
    }
}
