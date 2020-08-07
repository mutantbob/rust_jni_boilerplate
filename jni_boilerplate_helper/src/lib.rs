extern crate jni;

use log::debug;

use jni::objects::{JObject, JValue, AutoLocal, JString, JClass};
use jni::sys::{jbyteArray, jintArray, jshortArray, jsize};
use jni::{JNIEnv, AttachGuard};
use std::fmt::Write;

pub struct JClassWrapper<'a, 'b>
{
    pub jni_env: &'a JNIEnv<'a>,
    pub cls: JClass<'b>,
}

impl<'a, 'b> Drop for JClassWrapper<'a, 'b>
{
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

impl<T: JavaSignatureFor> JavaSignatureFor for Vec<T> {
    fn signature_for() -> String {
        String::from("[") + &T::signature_for()
    }
}
//

pub trait JavaClassNameFor
{
    fn java_class_name() -> &'static str;
}

impl JavaClassNameFor for &str
{
    fn java_class_name() -> &'static str {
        "java/lang/String"
    }
}

impl JavaClassNameFor for String
{
    fn java_class_name() -> &'static str {
        "java/lang/String"
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

impl<'a> ConvertJValueToRust<Vec<i8>> for JValue<'a> {
    fn into_rust(self, je: &JNIEnv) -> Result<Vec<i8>, jni::errors::Error> {
        let tmp:Vec<u8> = self.into_rust(je)?;

        Ok(vec_u8_into_i8(tmp))
    }
}

impl<'a> ConvertJValueToRust<Vec<u8>> for JValue<'a> {
    fn into_rust(self, je: &JNIEnv) -> Result<Vec<u8>, jni::errors::Error> {
        let object:JObject = self.l()?;
        let rval = je.convert_byte_array(*object);
        je.exception_check()?;
        //println!("delete_local_ref()");
        if let Err(e) =je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        rval
    }
}

//

pub trait ConvertRustToJValue<'a, 'b, T> {
    fn into_temporary(self, je:&'b JNIEnv<'a>) ->Result<T, jni::errors::Error>;
    fn temporary_into_jvalue(tmp: &T) -> JValue<'a>;
}

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
    fn into_temporary(self, _je:&'b JNIEnv<'a>) ->Result<char, jni::errors::Error> {Ok(self)}
    fn temporary_into_jvalue(tmp:&char) -> JValue<'a> {
        JValue::Char((*tmp) as u16)
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, bool> for bool {
    fn into_temporary(self, _je:&'b JNIEnv<'a>) ->Result<bool, jni::errors::Error> {Ok(self)}
    fn temporary_into_jvalue(tmp:&bool) -> JValue<'a> {
        JValue::Bool((*tmp) as u8)
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, jbyteArray> for &[i8] {
    fn into_temporary(self, je:&'b JNIEnv<'a>) ->Result<jbyteArray, jni::errors::Error> {
        let shenanigans = unsafe { &*(self as *const [i8] as *const [u8]) };
        Ok(je.byte_array_from_slice(shenanigans).unwrap())
    }
    fn temporary_into_jvalue(tmp: &jbyteArray) -> JValue<'a> {
        JObject::from(*tmp).into()
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, jbyteArray> for &[u8] {
    fn into_temporary(self, je:&'b JNIEnv<'a>) ->Result<jbyteArray, jni::errors::Error> {
        je.byte_array_from_slice(self)
    }
    fn temporary_into_jvalue(tmp: &jbyteArray) -> JValue<'a> {
        JObject::from(*tmp).into()
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, JString<'a>> for &str {
    fn into_temporary(self, je:&'b JNIEnv<'a>) ->Result<JString<'a>, jni::errors::Error> {
        je.new_string(self)
    }
    fn temporary_into_jvalue(tmp: &JString<'a>) -> JValue<'a> {
        let jo: JObject = JObject::from(*tmp);
        jo.into()
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, AutoLocal<'a,'b>> for &[i32] {
    fn into_temporary(self, je:&'b JNIEnv<'a>) ->Result<AutoLocal<'a,'b>,jni::errors::Error> {
        let rval:jintArray = je.new_int_array(self.len() as jsize)?;
        je.set_int_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b, AutoLocal<'a,'b>> for &[i16] {
    fn into_temporary(self, je:&'b JNIEnv<'a>) ->Result<AutoLocal<'a,'b>,jni::errors::Error> {
        let rval:jshortArray = je.new_short_array(self.len() as jsize)?;
        je.set_short_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
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

pub fn jni_boilerplate_instance_method_invocation(
    rust_name: &str,
    java_name: &str,
    argument_types: &[String],
    return_type_str: &Option<String>,
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
        body.push_str("results.into_rust(&self.jni_env)\n");
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
        writeln!(tmp, "let tmp{} = arg{}.into_temporary({})?;", i, i, jni_env_variable_name).unwrap();
    }
    tmp
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
        body.push_str("results.into_rust(je)\n");
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
