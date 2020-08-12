extern crate jni;
extern crate syn;

use log::debug;

use crate::array_copy_back::{ArrayCopyBackByte, ArrayCopyBackInt, ArrayCopyBackShort};
use java_runtime_wrappers::class_is_array;
use jni::objects::{AutoLocal, JClass, JObject, JString, JValue};
use jni::sys::{jbyteArray, jintArray, jshortArray, jsize};
use jni::{AttachGuard, JNIEnv};
use std::any::Any;
use std::fmt::Write;
use syn::{GenericArgument, PathArguments, ReturnType, Type, TypeTuple};

mod array_copy_back;
mod java_runtime_wrappers;

pub struct JClassWrapper<'a, 'b> {
    pub jni_env: &'a JNIEnv<'a>,
    pub cls: JClass<'b>,
}

impl<'a, 'b> Drop for JClassWrapper<'a, 'b> {
    fn drop(&mut self) {
        let res = self.jni_env.delete_local_ref(*self.cls);
        match res {
            Ok(()) => {}
            Err(e) => debug!("error dropping global ref: {:#?}", e),
        }
    }
}

#[macro_export]
macro_rules! jni_signature_single {
    (f32) => {
        "F"
    };
    (i32) => {
        "I"
    };
    (i8) => {
        "B"
    }; //(&[ $($ty:ty) ]) => { concat![ "[", jni_signature_single($ty)] };
}

pub trait JavaSignatureFor {
    fn signature_for() -> String;
}

impl JavaSignatureFor for () {
    fn signature_for() -> String {
        String::from("V")
    }
}

impl JavaSignatureFor for bool {
    fn signature_for() -> String {
        String::from("Z")
    }
}

impl JavaSignatureFor for i8 {
    fn signature_for() -> String {
        String::from("B")
    }
}

impl JavaSignatureFor for char {
    fn signature_for() -> String {
        String::from("C")
    }
}

impl JavaSignatureFor for i16 {
    fn signature_for() -> String {
        String::from("S")
    }
}

impl JavaSignatureFor for i32 {
    fn signature_for() -> String {
        String::from("I")
    }
}

impl JavaSignatureFor for i64 {
    fn signature_for() -> String {
        String::from("J")
    }
}

impl JavaSignatureFor for f32 {
    fn signature_for() -> String {
        String::from("F")
    }
}

impl JavaSignatureFor for f64 {
    fn signature_for() -> String {
        String::from("D")
    }
}

impl JavaSignatureFor for &str {
    fn signature_for() -> String {
        format!("L{};", <Self>::java_class_name())
    }
}

impl JavaSignatureFor for String {
    fn signature_for() -> String {
        format!("L{};", <Self>::java_class_name())
    }
}

impl<T: JavaSignatureFor> JavaSignatureFor for &[T] {
    fn signature_for() -> String {
        String::from("[") + &T::signature_for()
    }
}

impl<T: JavaSignatureFor> JavaSignatureFor for &mut [T] {
    fn signature_for() -> String {
        String::from("[") + &T::signature_for()
    }
}

impl<T: JavaSignatureFor> JavaSignatureFor for Vec<T> {
    fn signature_for() -> String {
        String::from("[") + &T::signature_for()
    }
}
//

pub trait JavaClassNameFor {
    fn java_class_name() -> &'static str;
}

impl JavaClassNameFor for &str {
    fn java_class_name() -> &'static str {
        "java/lang/String"
    }
}

impl JavaClassNameFor for String {
    fn java_class_name() -> &'static str {
        "java/lang/String"
    }
}

//

pub trait ConvertJValueToRust
where
    Self: std::marker::Sized,
{
    fn to_rust(je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error>;
}

impl ConvertJValueToRust for () {
    fn to_rust<'a>(_je: &JNIEnv, val: &JValue<'a>) -> Result<Self, jni::errors::Error> {
        val.v()
    }
}

impl<'a> ConvertJValueToRust for char {
    fn to_rust(_je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        val.c().and_then(|c| match std::char::from_u32(c as u32) {
            None => Err(jni::errors::Error::from_kind(
                jni::errors::ErrorKind::JavaException,
            )),
            Some(ch) => Ok(ch),
        })
    }
}

impl ConvertJValueToRust for i8 {
    fn to_rust(_je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        val.b()
    }
}

impl ConvertJValueToRust for i16 {
    fn to_rust(_je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        val.s()
    }
}

impl ConvertJValueToRust for i32 {
    fn to_rust(_je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        val.i()
    }
}

impl ConvertJValueToRust for i64 {
    fn to_rust(_je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        val.j()
    }
}

impl ConvertJValueToRust for String {
    fn to_rust(je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        let obj = val.l()?;
        let x = je.get_string(obj.into())?;
        let result = x.to_str();
        match result {
            Err(e) => panic!(e),
            Ok(rval) => Ok(String::from(rval)),
        }
    }
}

fn vec_u8_into_i8(v: Vec<u8>) -> Vec<i8> {
    // converse of https://stackoverflow.com/a/59707887/995935
    // ideally we'd use Vec::into_raw_parts, but it's unstable,
    // so we have to do it manually:

    // first, make sure v's destructor doesn't free the data
    // it thinks it owns when it goes out of scope
    let mut v = std::mem::ManuallyDrop::new(v);

    // then, pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // finally, adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

impl ConvertJValueToRust for Vec<i8> {
    fn to_rust(je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        let tmp: Vec<u8> =
            //Vec::u8::to_rust
            Vec::<u8>::to_rust
            (je, val)?;

        Ok(vec_u8_into_i8(tmp))
    }
}

impl ConvertJValueToRust for Vec<u8> {
    fn to_rust(je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let rval = je.convert_byte_array(*object);
        je.exception_check()?;
        //println!("delete_local_ref()");
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        rval
    }
}

pub fn convert_jvalue_list_or_array_to_rust<T>(
    je: &JNIEnv,
    src: JObject,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust,
{
    //println!("convert_jvalue_list_or_array_to_rust");

    let cls = je.get_object_class(src)?;
    if class_is_array(je, &cls)? {
        convert_jarray_to_rust(je, src)
    } else {
        convert_iterable_to_rust_vec(je, src)
    }
}

pub fn convert_jarray_to_rust<T>(je: &JNIEnv, array: JObject) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust,
{
    let count = je.get_array_length(*array)?;
    let mut rval: Vec<T> = Vec::new();
    for i in 0..count {
        let obj_i = je.get_object_array_element(*array, i)?;
        let val: T = T::to_rust(je, &JValue::from(obj_i))?;
        rval.push(val);
    }
    Ok(rval)
}

pub fn convert_iterable_to_rust_vec<T>(
    je: &JNIEnv,
    iterable: JObject,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust,
{
    let iter = je.call_method(iterable, "iterator", "()Ljava/util/Iterator;", &[])?;
    let iter = iter.l()?;

    let mut rval: Vec<T> = Vec::new();
    loop {
        let has_next = je.call_method(iter, "hasNext", "()Z", &[])?;
        if !has_next.z()? {
            break;
        }
        let val = je.call_method(iter, "next", "()Ljava/lang/Object;", &[])?;
        let val: T = T::to_rust(je, &val)?;
        rval.push(val);
    }

    Ok(rval)
}

/*
#[macro_export]
macro_rules! impl_convert_jvalue_to_rust_vec {
  ( $($t:ty),* ) => {
  $( impl ConvertJValueToRust for Vec<$t> {
    fn to_rust(je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
          use $crate::convert_jvalue_list_or_array_to_rust;
          let jobject:JObject = val.l()?;
          convert_jvalue_list_or_array_to_rust(jobject)
      }
  })*
  }
}

impl_convert_jvalue_to_rust_vec!{String}
*/

pub trait JValueNonScalar {}

impl JValueNonScalar for String {}
impl<T> JValueNonScalar for Vec<T> {}

impl<T: JValueNonScalar + ConvertJValueToRust> ConvertJValueToRust for Vec<T> {
    fn to_rust(je: &JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
        let jobject: JObject = val.l()?;
        convert_jvalue_list_or_array_to_rust(je, jobject)
    }
}

//

pub trait ConvertRustToJValue<'a, 'b, T> {
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<T, jni::errors::Error>;
    fn temporary_into_jvalue(tmp: &T) -> JValue<'a>;
}

#[macro_export]
macro_rules! impl_convert_rust_to_jvalue {
    ( $($t:ty),* ) => {
    $( impl<'a,'b> ConvertRustToJValue<'a, 'b, $t> for $t
    {
        fn into_temporary(self, _je:&'b JNIEnv<'a>) ->Result<$t, jni::errors::Error> { Ok(self) }
        fn temporary_into_jvalue(tmp: &$t) -> JValue<'a>
        {
        (*tmp).into()
        }
    }) *
    }
}

impl_convert_rust_to_jvalue! { i8, i16, i32, i64, f32, f64 }

impl<'a, 'b> ConvertRustToJValue<'a, 'b, char> for char {
    fn into_temporary(self, _je: &'b JNIEnv<'a>) -> Result<char, jni::errors::Error> {
        Ok(self)
    }
    fn temporary_into_jvalue(tmp: &char) -> JValue<'a> {
        JValue::Char((*tmp) as u16)
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, bool> for bool {
    fn into_temporary(self, _je: &'b JNIEnv<'a>) -> Result<bool, jni::errors::Error> {
        Ok(self)
    }
    fn temporary_into_jvalue(tmp: &bool) -> JValue<'a> {
        JValue::Bool((*tmp) as u8)
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, jbyteArray> for &[i8] {
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<jbyteArray, jni::errors::Error> {
        let shenanigans = unsafe { &*(self as *const [i8] as *const [u8]) };
        Ok(je.byte_array_from_slice(shenanigans).unwrap())
    }
    fn temporary_into_jvalue(tmp: &jbyteArray) -> JValue<'a> {
        JObject::from(*tmp).into()
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, jbyteArray> for &[u8] {
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<jbyteArray, jni::errors::Error> {
        je.byte_array_from_slice(self)
    }
    fn temporary_into_jvalue(tmp: &jbyteArray) -> JValue<'a> {
        JObject::from(*tmp).into()
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, JString<'a>> for &str {
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<JString<'a>, jni::errors::Error> {
        je.new_string(self)
    }
    fn temporary_into_jvalue(tmp: &JString<'a>) -> JValue<'a> {
        let jo: JObject = JObject::from(*tmp);
        jo.into()
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, JString<'a>> for String {
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<JString<'a>, jni::errors::Error> {
        je.new_string(&self)
    }
    fn temporary_into_jvalue(tmp: &JString<'a>) -> JValue<'a> {
        let jo: JObject = JObject::from(*tmp);
        jo.into()
    }
}

/*
impl<'a, 'b, T> ConvertRustToJValue<'a, 'b, AutoLocal<'a, 'b>> for &[T]
{
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, Error> {
        let cls =
        je.new_object_array(self.len() , )
    }

    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        unimplemented!()
    }
}
*/

impl<'a, 'b> ConvertRustToJValue<'a, 'b, AutoLocal<'a, 'b>> for &[i32] {
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval: jintArray = je.new_int_array(self.len() as jsize)?;
        je.set_int_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a, 'b, 'c> ConvertRustToJValue<'a, 'b, ArrayCopyBackInt<'a, 'b, 'c>> for &'c mut [i32] {
    fn into_temporary(
        self,
        je: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackInt<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackInt::new(self, je)
    }
    fn temporary_into_jvalue(tmp: &ArrayCopyBackInt<'a, 'b, 'c>) -> JValue<'a> {
        tmp.as_jvalue()
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, AutoLocal<'a, 'b>> for &[i16] {
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval: jshortArray = je.new_short_array(self.len() as jsize)?;
        je.set_short_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a, 'b, 'c> ConvertRustToJValue<'a, 'b, ArrayCopyBackShort<'a, 'b, 'c>> for &'c mut [i16] {
    fn into_temporary(
        self,
        je: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackShort<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackShort::new(self, je)
    }
    fn temporary_into_jvalue(tmp: &ArrayCopyBackShort<'a, 'b, 'c>) -> JValue<'a> {
        tmp.as_jvalue()
    }
}

impl<'a, 'b, 'c> ConvertRustToJValue<'a, 'b, ArrayCopyBackByte<'a, 'b, 'c>> for &'c mut [i8] {
    fn into_temporary(
        self,
        je: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackByte<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackByte::new(self, je)
    }
    fn temporary_into_jvalue(tmp: &ArrayCopyBackByte<'a, 'b, 'c>) -> JValue<'a> {
        tmp.as_jvalue()
    }
}

//

pub trait JavaConstructible<'a> {
    fn wrap_jobject(jni_env: &'a AttachGuard<'a>, java_this: AutoLocal<'a, 'a>) -> Self;
}

//

pub fn function_argument_declaration_text(inputs: &[String]) -> String {
    let mut rval = String::new();
    for (idx, type_str) in inputs.iter().enumerate() {
        if idx > 0 {
            rval.push_str(", ");
        }
        rval.push_str("arg");
        rval.push_str(&idx.to_string());
        rval.push_str(":");
        rval.push_str(&type_str);
    }

    rval
}

pub fn jni_method_signature_string(
    argument_types: &[String],
    return_type_str: &Option<String>,
) -> String {
    let mut rval = String::from("String::from(\"(\")");
    for arg_type in argument_types {
        rval.push_str("+&");
        rval.push_str("<");
        rval.push_str(&arg_type);
        rval.push_str(">::signature_for()\n")
    }
    rval.push_str("+\")\"+");
    match return_type_str {
        None => rval.push_str("\"V\""),
        Some(ty) => {
            rval.push_str("&<");
            rval.push_str(&ty);
            rval.push_str(">::signature_for()\n")
        }
    };

    rval
}

pub fn jni_argument_array(argument_types: &[String], _jni_env_variable_name: &str) -> String {
    let mut body = String::from("&[");
    for (i, arg_type) in argument_types.iter().enumerate() {
        if i > 0 {
            body.push_str(", ");
        }
        write!(body, "<{}>::temporary_into_jvalue(&tmp{})", arg_type, i).unwrap();
    }
    body.push_str("]");

    body
}

pub fn return_type_to_string(ty: &ReturnType, freaky: bool) -> String {
    match ty {
        ReturnType::Default => String::from("()"),
        ReturnType::Type(_arrow, ty) => type_to_string(ty, freaky),
    }
}

pub fn type_to_string(ty: &Type, freaky: bool) -> String {
    match ty {
        Type::Path(type_path) => path_segments_to_string(&type_path.path, freaky),
        Type::Reference(reference) => {
            String::from("&")
                + (if reference.mutability.is_some() {
                    "mut "
                } else {
                    ""
                })
                + &type_to_string(&reference.elem, false)
        }
        Type::Slice(array) => {
            //println!("{:?}", ty.type_id());
            String::from("[") + &type_to_string(&array.elem, false) + "]"
        }
        Type::Tuple(tup) => tuple_to_string(tup),

        _ => panic!("unhandled variant of Type {:?}", ty.type_id()),
    }
}

pub fn path_segments_to_string(path1: &syn::Path, freaky: bool) -> String {
    let prefix: String = match path1.leading_colon {
        Some(_) => String::from("::"),
        None => String::new(),
    };

    path1.segments.iter().fold(prefix, |mut acc, v| {
        if !acc.is_empty() {
            acc.push_str("::")
        }
        acc.push_str(&v.ident.to_string());
        acc.push_str(&path_arguments_to_string(&v.arguments, freaky));

        acc
    })
}

pub fn path_arguments_to_string(args: &PathArguments, freaky: bool) -> String {
    match args {
        PathArguments::None => String::from(""),
        PathArguments::AngleBracketed(abga) => {
            let mut acc = String::new();
            for part in &abga.args {
                match part {
                    GenericArgument::Type(t) => acc.push_str(&type_to_string(t, false)),
                    _ => panic!("I don't support this"),
                }
            }

            if freaky {
                format!("::<{}>", acc)
            } else {
                format!("<{}>", acc)
            }
        }
        PathArguments::Parenthesized(_) => panic!("I don't support this"),
    }
}

pub fn tuple_to_string(tuple: &TypeTuple) -> String {
    let mut rval = String::from("(");
    for elem in &tuple.elems {
        if rval.len() > 1 {
            rval.push_str(",");
        }
        rval.push_str(&type_to_string(elem, false));
    }
    rval.push_str(")");

    rval
}

pub fn jni_boilerplate_instance_method_invocation(
    rust_name: &str,
    java_name: &str,
    argument_types: &[String],
    return_type_str: &Option<String>,
    return_type: &ReturnType,
) -> String {
    let mut body: String = String::from("pub fn ");

    body.push_str(rust_name);

    body.push_str("(&self, ");
    body.push_str(&function_argument_declaration_text(&argument_types));
    body.push(')');

    body.push_str(" -> Result<");

    match &return_type_str {
        None => body.push_str("()"),
        Some(type_str) => body.push_str(&type_str),
    }
    body.push_str(", jni::errors::Error>\n");

    body.push_str("{\n");
    //body.push_str("extern crate jni_boilerplate_helper;\n"); // this doesn't seem to help
    body.push_str(
        "use jni_boilerplate_helper::{JavaSignatureFor,ConvertRustToJValue,ConvertJValueToRust};\n",
    );
    body.push_str("let sig = \n");
    body.push_str(&jni_method_signature_string(
        &argument_types,
        &return_type_str,
    ));
    body.push_str(";\n");

    body.push_str(&build_temporaries(argument_types, "&self.jni_env"));

    let returns_void = return_type_str.is_none();

    if !returns_void {
        body.push_str("let results = ");
    }
    body.push_str("self.jni_env.call_method(self.java_this.as_obj(), \"");
    body.push_str(java_name);
    body.push_str("\", sig, ");
    body.push_str(&jni_argument_array(argument_types, "&self.jni_env"));
    body.push_str(")");

    body.push_str("?;\n");
    if returns_void {
        body.push_str("Ok(())\n");
    } else {
        writeln!(
            body,
            "{}::to_rust(&self.jni_env, &results)",
            return_type_to_string(return_type, true)
        )
        .unwrap();
    }

    body.push_str("}\n");

    if false {
        println!("{}", body);
    }
    body
}

fn build_temporaries(argument_types: &[String], jni_env_variable_name: &str) -> String {
    let mut tmp = String::new();
    for (i, _arg_type) in argument_types.iter().enumerate() {
        writeln!(
            tmp,
            "let tmp{} = arg{}.into_temporary({})?;",
            i, i, jni_env_variable_name
        )
        .unwrap();
    }
    tmp
}

pub fn jni_boilerplate_unwrapped_instance_method_invocation(
    rust_name: &str,
    java_name: &str,
    argument_types: &[String],
    return_type_str: &Option<String>,
    return_type: &ReturnType,
) -> String {
    let mut body: String = String::from("pub fn ");

    body.push_str(rust_name);

    body.push_str("(je: &jni::JNIEnv, java_this: jni::objects::JObject, ");
    body.push_str(&function_argument_declaration_text(&argument_types));
    body.push(')');

    body.push_str(" -> Result<");

    match &return_type_str {
        None => body.push_str("()"),
        Some(type_str) => body.push_str(&type_str),
    }
    body.push_str(", jni::errors::Error>\n");

    body.push_str("{\n");
    //body.push_str("extern crate jni_boilerplate_helper;\n"); // this doesn't seem to help
    body.push_str(
        "use jni_boilerplate_helper::{JavaSignatureFor,ConvertRustToJValue,ConvertJValueToRust};\n",
    );
    body.push_str("let sig = \n");
    body.push_str(&jni_method_signature_string(
        &argument_types,
        &return_type_str,
    ));
    body.push_str(";\n");

    body.push_str(&build_temporaries(argument_types, "je"));

    let returns_void = return_type_str.is_none();

    if !returns_void {
        body.push_str("let results = ");
    }
    body.push_str("je.call_method(java_this, \"");
    body.push_str(java_name);
    body.push_str("\", sig, ");
    body.push_str(&jni_argument_array(argument_types, "je"));
    body.push_str(")");

    body.push_str("?;\n");
    if returns_void {
        body.push_str("Ok(())\n");
    } else {
        writeln!(
            body,
            "{}::to_rust(je, &results)",
            return_type_to_string(return_type, true)
        )
        .unwrap();
    }

    body.push_str("}\n");

    if false {
        println!("{}", body);
    }
    body
}

//

pub fn jni_boilerplate_constructor_invocation(
    class_name: &str,
    constructor_name: &str,
    argument_types: &[String],
) -> String {
    let mut body = String::new();

    body.push_str("pub fn ");
    body.push_str(constructor_name);
    body.push_str("(je: &'a jni::AttachGuard<'a>");

    for (i, ty) in argument_types.iter().enumerate() {
        body.push_str(", arg");
        body.push_str(&i.to_string());
        body.push_str(": ");
        body.push_str(ty);
    }

    body.push_str(") -> Result<Self, jni::errors::Error> {\n");
    body.push_str(
        "use jni_boilerplate_helper::{JavaSignatureFor,ConvertRustToJValue,ConvertJValueToRust};\n",
    );
    body.push_str(&build_temporaries(argument_types, "&je"));

    body.push_str("let cls = je.find_class(\"");
    body.push_str(class_name);
    body.push_str("\")?;");
    body.push_str("let rval = je.new_object(cls, ");
    body.push_str(&jni_method_signature_string(&argument_types, &None));
    body.push_str(", ");
    body.push_str(&jni_argument_array(argument_types, "&je"));
    body.push_str(")?;\n");
    body.push_str("Ok(Self::wrap_jobject(je, AutoLocal::new(&je, rval)))");
    body.push_str("}\n");
    body
}
