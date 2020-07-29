extern crate jni;

use jni::objects::{JObject, JValue};
use jni::sys::jobject;
use jni::JNIEnv;

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
        String::from("Ljava/lang/String;")
    }
}

impl JavaSignatureFor for String {
    fn signature_for() -> String {
        String::from("Ljava/lang/String;")
    }
}

impl<T: JavaSignatureFor> JavaSignatureFor for &[T] {
    fn signature_for() -> String {
        String::from("[") + &T::signature_for()
    }
}

//

pub trait ConvertJValueToRust<T> {
    fn into_rust(self, je: &JNIEnv) -> Result<T, jni::errors::Error>;
}

impl<'a> ConvertJValueToRust<char> for JValue<'a> {
    fn into_rust(self, _je: &JNIEnv) -> Result<char, jni::errors::Error> {
        self.c().and_then(|c| match std::char::from_u32(c as u32) {
            None => Err(jni::errors::Error::from_kind(
                jni::errors::ErrorKind::JavaException,
            )),
            Some(ch) => Ok(ch),
        })
    }
}

impl<'a> ConvertJValueToRust<i8> for JValue<'a> {
    fn into_rust(self, _je: &JNIEnv) -> Result<i8, jni::errors::Error> {
        self.b()
    }
}

impl<'a> ConvertJValueToRust<i16> for JValue<'a> {
    fn into_rust(self, _je: &JNIEnv) -> Result<i16, jni::errors::Error> {
        self.s()
    }
}

impl<'a> ConvertJValueToRust<i32> for JValue<'a> {
    fn into_rust(self, _je: &JNIEnv) -> Result<i32, jni::errors::Error> {
        self.i()
    }
}

impl<'a> ConvertJValueToRust<i64> for JValue<'a> {
    fn into_rust(self, _je: &JNIEnv) -> Result<i64, jni::errors::Error> {
        self.j()
    }
}

impl<'a> ConvertJValueToRust<String> for JValue<'a> {
    fn into_rust(self, je: &JNIEnv) -> Result<String, jni::errors::Error> {
        let obj = self.l()?;
        let x = je.get_string(obj.into())?;
        let result = x.to_str();
        match result {
            Err(e) => panic!(e),
            Ok(rval) => Ok(String::from(rval)),
        }
    }
}

//

pub trait ConvertRustToJValue<'a> {
    fn into_jvalue(self, je: &JNIEnv<'a>) -> JValue<'a>;
}

macro_rules! impl_convert_rust_to_jvalue {
    ( $($t:ty),* ) => {
    $( impl<'a> ConvertRustToJValue<'a> for $t
    {
        fn into_jvalue(self, _je:&JNIEnv) -> JValue<'a>
        {
        self.into()
        }
    }) *
    }
}

impl_convert_rust_to_jvalue! { i8, i16, i32, i64 }

impl<'a> ConvertRustToJValue<'a> for char {
    fn into_jvalue(self, _je: &JNIEnv) -> JValue<'a> {
        JValue::Char(self as u16)
    }
}

impl<'a> ConvertRustToJValue<'a> for bool {
    fn into_jvalue(self, _je: &JNIEnv) -> JValue<'a> {
        JValue::Bool(self as u8)
    }
}

impl<'a> ConvertRustToJValue<'a> for &[i8] {
    fn into_jvalue(self, je: &JNIEnv<'a>) -> JValue<'a> {
        let shenanigans = unsafe { &*(self as *const [i8] as *const [u8]) };
        shenanigans.into_jvalue(je)
    }
}

impl<'a> ConvertRustToJValue<'a> for &[u8] {
    fn into_jvalue(self, je: &JNIEnv) -> JValue<'a> {
        let jba: jobject = je.byte_array_from_slice(self).unwrap();
        let jo = JObject::from(jba);
        jo.into()
    }
}

impl<'a> ConvertRustToJValue<'a> for &str {
    fn into_jvalue(self, je: &JNIEnv<'a>) -> JValue<'a> {
        let str = je.new_string(self).unwrap();
        let jo: JObject = JObject::from(str);
        jo.into()
    }
}

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

pub fn jni_boilerplate_instance_method_invocation(
    rust_name: &str,
    java_name: &str,
    argument_types: &[String],
    return_type_str: &Option<String>,
) -> String {
    let mut body: String = String::from("pub fn ");

    body.push_str(rust_name);

    body.push_str("(&self, je: &jni::JNIEnv, ");
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

    let returns_void = return_type_str.is_none();

    if !returns_void {
        body.push_str("let results = ");
    }
    body.push_str("je.call_method(self.java_this, \"");
    body.push_str(java_name);
    body.push_str("\", sig, &[");
    for i in 0..argument_types.len() {
        //for (i, arg_type) in macro_args.signature.inputs.iter().enumerate() {
        if i > 0 {
            body.push_str(", ");
        }
        body.push_str("arg");
        body.push_str(&i.to_string());
        body.push_str(".into_jvalue::<");
        body.push_str(">(je)");
    }
    body.push_str("])");

    body.push_str("?;\n");
    if returns_void {
        body.push_str("Ok(())\n");
    } else {
        body.push_str("results.into_rust(je)\n");
    }

    body.push_str("}\n");

    if false {
        println!("{}", body);
    }
    body
}

pub fn jni_boilerplate_unwrapped_instance_method_invocation(
    rust_name: &str,
    java_name: &str,
    argument_types: &[String],
    return_type_str: &Option<String>,
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

    let returns_void = return_type_str.is_none();

    if !returns_void {
        body.push_str("let results = ");
    }
    body.push_str("je.call_method(java_this, \"");
    body.push_str(java_name);
    body.push_str("\", sig, &[");
    for i in 0..argument_types.len() {
        //for (i, arg_type) in macro_args.signature.inputs.iter().enumerate() {
        if i > 0 {
            body.push_str(", ");
        }
        body.push_str("arg");
        body.push_str(&i.to_string());
        body.push_str(".into_jvalue::<");
        body.push_str(">(je)");
    }
    body.push_str("])");

    body.push_str("?;\n");
    if returns_void {
        body.push_str("Ok(())\n");
    } else {
        body.push_str("results.into_rust(je)\n");
    }

    body.push_str("}\n");

    if false {
        println!("{}", body);
    }
    body
}
