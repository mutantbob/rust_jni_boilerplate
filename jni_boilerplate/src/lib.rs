extern crate proc_macro;
#[macro_use]
extern crate syn;
extern crate jni_boilerplate_helper;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use proc_macro::{TokenStream, Span};
use proc_macro2::Ident;
use std::any::Any;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{ReturnType, Type, TypeBareFn, FnArg, PatType, Pat, PatIdent};
use syn::token::Comma;


use jni_boilerplate_helper::{jni_boilerplate_constructor_invocation, jni_boilerplate_instance_method_invocation, jni_boilerplate_unwrapped_instance_method_invocation};

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

///
/// example:
/// <pre>jni_instance_method!{ fn_name[=java_name], fn([ arg_type1 [,arg_type2...]])[ ->return_type ] }
/// </pre>
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
        let ident: Ident = tokens.parse()?;
        let class_name = ident.to_string();

        let (constructor_name, mut class_name) = if tokens.peek(Token![=]) {
            let _eq: Token![=] = tokens.parse()?;
            let constructor_name = class_name;
            let ident: Ident = tokens.parse()?;
            let class_name = ident.to_string();
            (constructor_name, class_name)
        } else {
            (String::from("new"), class_name)
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

    let constructor_name: &str = &macro_args.constructor_name;
    let argument_types: Vec<String> = macro_args
        .argument_types
        .iter()
        .map(|ty| type_to_string(ty))
        .collect();

    let class_name: &str = &macro_args.class_name;

    let body: String =
        jni_boilerplate_constructor_invocation(class_name, constructor_name, &argument_types);

    quote::quote! { pants };

    body.parse().unwrap()
}

//
//
//

struct StaticMethodArgs
{
    rust_name: Ident,
    java_name: String,
    argument_types: Vec<Type>,
    return_type: ReturnType,
}

impl<'a> Parse for StaticMethodArgs
{
    fn parse(tokens: &ParseBuffer) -> Result<Self, syn::Error> {
        let function_name: Ident = tokens.parse()?;

        let (rust_name, java_name) = if tokens.peek(Token![=]) {
            let _eq: Token![=] = tokens.parse()?;

            let ident: Ident = tokens.parse()?;
            ( function_name, ident.to_string())
        } else {
            (function_name.clone(), function_name.to_string())
        };

        let mut argument_types: Vec<Type> = Vec::new();
        let arg_types: ParseBuffer;
        parenthesized!(arg_types in tokens);

        while !arg_types.is_empty() {
            let arg_type:Type = arg_types.parse()?;
            argument_types.push(arg_type);
            if !arg_types.is_empty() {
                let _comma: Token![,] = arg_types.parse()?;
            }
        }

        println!("do I get a return type?");

        let return_type:ReturnType = if tokens.peek(Token![->]) {
            println!("parse return type");
            //let _arrow: Token![->] = tokens.parse()?;
            let rval = tokens.parse()?;
            println!("win");
            rval
        } else {
            ReturnType::Default
        };

        Ok(StaticMethodArgs {
            rust_name,
            java_name,
            argument_types,
            return_type,
        })
    }
}

struct AllAboutArg
{
    a_type: Type,
    p_ident: Ident,
    p_name: String,
    tmp_ident: Ident,
}

impl AllAboutArg
{
    pub fn new(arg_type:Type, index: usize)->AllAboutArg
    {
        let p_name = format!("arg{}", index);
        let tmp_name =format!("tmp{}", index);
        AllAboutArg {
            a_type: arg_type,
            p_ident: Ident::new(&p_name, Span::call_site().into()),
            p_name,
            tmp_ident: Ident::new(&tmp_name, Span::call_site().into()),
        }
    }
}

#[proc_macro]
pub fn jni_static_method(t_stream: TokenStream) -> TokenStream {
    let macro_args = syn::parse_macro_input!(t_stream as StaticMethodArgs);

    let rust_name = &macro_args.rust_name;
    let java_name: &str = &macro_args.java_name;

    let return_type:Type = match &macro_args.return_type
    {
        ReturnType::Default => {
            let ts2:TokenStream = "()".parse().unwrap();
            let blank:Type = syn::parse_macro_input!(ts2 as Type);
            blank
        },
        ReturnType::Type(_, rt) => {
            let rt:&Type = rt;
            (*rt).clone()
        }
    };

    println!("what now?");

    let arg_types = &macro_args.argument_types;

    let args_metadata:Vec<AllAboutArg> = arg_types.iter().enumerate()
        .map(|(i,t)| AllAboutArg::new((*t).clone(), i))
        .collect();

    let mut arg_sig: syn::punctuated::Punctuated<FnArg, Comma>;
    arg_sig = syn::punctuated::Punctuated::new();

    for arg in &args_metadata {
        let arg2 = named_function_argument(&arg.p_name, &arg.a_type);
        arg_sig.push(arg2)
    }

    println!("what now 2?");

    let decl:Vec<proc_macro2::TokenStream> = args_metadata.iter()
        .map( |metadata| {
            let tmp_i = &metadata.tmp_ident;
            let p_i = &metadata.p_ident;
            quote!{let #tmp_i = #p_i.into_temporary(jni_env)?;}
        } )
        .collect();
    /*

    let decl0 = {
        let md0 = &args_metadata[0];
        let tmp_0 = &md0.tmp_ident;
        let p_0 = &md0.p_ident;
        quote!{ let #tmp_0 = #p_0.into_temporary(jni_env)?; }
    };
    let rtso:Option<String> = match &macro_args.return_type {
        ReturnType::Default => None,
        ReturnType::Type(_, bt) => Some(type_to_string(&*bt))
    };
    */

    //let sig = jni_method_signature_string(&arg_type_strings, &rtso);
    //let sig:syn::LitStr = syn::LitStr::new(&sig, Span::call_site().into());

    let jvalue_param_array:Vec<proc_macro2::TokenStream> = args_metadata.iter()
        .map(|metadata| {
            let ty = &metadata.a_type;
            let tmp_i = &metadata.tmp_ident;
            quote! { <#ty>::temporary_into_jvalue(&#tmp_i) }
        }).collect();

    let body = quote!{
    pub fn #rust_name(jni_env: &jni::JNIEnv, #arg_sig) ->Result<#return_type, jni::errors::Error>
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

        results.into_rust(jni_env)
    }
    };
    //let pants = "#(arg#arg_indices.into_temporary(jni_env);),*";
    //let tuple = (#(arg#arg_indices.into_temporary(jni_env);),*);
    //#(let tmp#arg_indices = arg#arg_indices.into_temporary(jni_env);)*
    body.into()
}

/*
this is way more trouble than it is worth
fn wrap_result_type(macro_args: &StaticMethodArgs) {
    &macro_args.return_type;

    let mut result_type_args = Punctuated::new();
    result_type_args.push(GenericArgument::Type(match macro_args.return_type
    {}));

    let mut segments = Punctuated::new();

    segments.push(PathSegment {
        ident: Ident::new("Result", Span::call_site().into()),
        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: syn::token::Lt(Span::call_site().into()),
            args: result_type_args,
            gt_token: Token![>](Span::call_site().into())
        })
    });
    let tp: TypePath = TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: segments,
        }
    };
    let rt: Type = Type::Path(tp);
    let return_type = ReturnType::Type(if let ReturnType::Type(arrow, _) = macro_args.return_type {
        arrow
    } else { Token![->](Span::call_site().into()) }, Box::new(rt));
}
*/

fn named_function_argument(name: &str, arg_type: &Type) -> FnArg {
    let arg_ident: PatIdent = PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability: None,
        ident: Ident::new(name, Span::call_site().into()),
        subpat: None
    };
    let pat_type: PatType = PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(arg_ident)),
        colon_token: syn::token::Colon {
            spans: [Span::call_site().into()]
        },
        ty: Box::new((*arg_type).clone())
    };
    let arg2: syn::FnArg = FnArg::Typed(pat_type);
    arg2
}
