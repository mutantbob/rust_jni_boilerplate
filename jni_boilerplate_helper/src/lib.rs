#![allow(clippy::wrong_self_convention)]

use log::debug;

use crate::array_copy_back::*;
use java_runtime_wrappers::class_is_array;
use jni::errors::Error;
use jni::objects::{AutoLocal, JClass, JObject, JValue};
use jni::sys::{
    jboolean, jbooleanArray, jdoubleArray, jfloatArray, jintArray, jlongArray, jobjectArray,
    jshortArray, jsize,
};
use jni::JNIEnv;

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

/// The class name JNI needs is separated by /s, not .s .
/// This is probably the cause of a *lot* of NoClassDefFoundError exceptions.
pub trait JavaClassNameFor {
    /// The class name JNI needs is separated by /s, not .s .
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

///
/// part of the job of the implementation of the to_rust() method is to release the resources
/// held by the JValue to prevent memory leaks
/// (which is probably anything where the JValue has a jobject returned by .l() )
pub trait ConvertJValueToRust<'a, 'b>
where
    Self: std::marker::Sized,
{
    fn to_rust(je: &'b JNIEnv<'a>, val: JValue<'a>) -> Result<Self, jni::errors::Error>;
}

impl ConvertJValueToRust<'_, '_> for () {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.v()
    }
}

impl ConvertJValueToRust<'_, '_> for bool {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.z()
    }
}
impl ConvertJValueToRust<'_, '_> for char {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.c().and_then(|c| match std::char::from_u32(c as u32) {
            None => Err(jni::errors::Error::from_kind(
                jni::errors::ErrorKind::JavaException,
            )),
            Some(ch) => Ok(ch),
        })
    }
}

impl ConvertJValueToRust<'_, '_> for i8 {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.b()
    }
}

impl ConvertJValueToRust<'_, '_> for i16 {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.s()
    }
}

impl ConvertJValueToRust<'_, '_> for i32 {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.i()
    }
}

impl ConvertJValueToRust<'_, '_> for i64 {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.j()
    }
}

impl ConvertJValueToRust<'_, '_> for f32 {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.f()
    }
}

impl ConvertJValueToRust<'_, '_> for f64 {
    fn to_rust(_je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        val.d()
    }
}

impl ConvertJValueToRust<'_, '_> for String {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let obj = val.l()?;
        let x = je.get_string(obj.into())?;
        let result = x.to_str();
        match result {
            Err(e) => panic!("{}", e),
            Ok(rval) => {
                let rval = String::from(rval);
                drop(x);
                je.delete_local_ref(obj)?;
                Ok(rval)
            }
        }
    }
}

impl ConvertJValueToRust<'_, '_> for Vec<bool> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let count: jsize = je.get_array_length(*object)?;
        je.exception_check()?;
        #[allow(clippy::unnecessary_cast)]
        let mut rval = vec![0 as jboolean; count as usize];
        let slice: &mut [jboolean] = &mut rval;
        je.get_boolean_array_region(*object, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }

        let rval = rval.iter().map(|&x| x != 0).collect();
        Ok(rval)
    }
}

pub fn u32_to_char(val: u32) -> Result<char, jni::errors::Error> {
    if let Some(ch) = std::char::from_u32(val) {
        Ok(ch)
    } else {
        Err(jni::errors::Error::from_kind(
            jni::errors::ErrorKind::JavaException,
        ))
    }
}

impl ConvertJValueToRust<'_, '_> for Vec<char> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let count = je.get_array_length(*object)?;
        let mut rval = vec![0 as char; count as usize];
        move_jchararray_to_char_array(je, *object, &mut rval)?;

        Ok(rval)
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

impl ConvertJValueToRust<'_, '_> for Vec<i8> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let tmp: Vec<u8> =
            //Vec::u8::to_rust
            Vec::<u8>::to_rust
            (je, val)?;

        Ok(vec_u8_into_i8(tmp))
    }
}

impl ConvertJValueToRust<'_, '_> for Vec<u8> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
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

impl ConvertJValueToRust<'_, '_> for Vec<i16> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let count: jsize = je.get_array_length(*object)?;
        je.exception_check()?;
        let mut rval = vec![0_i16; count as usize];
        let slice: &mut [i16] = &mut rval;
        je.get_short_array_region(*object, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}

impl ConvertJValueToRust<'_, '_> for Vec<i32> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let count: jsize = je.get_array_length(*object)?;
        je.exception_check()?;
        let mut rval = vec![0_i32; count as usize];
        let slice: &mut [i32] = &mut rval;
        je.get_int_array_region(*object, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}

impl ConvertJValueToRust<'_, '_> for Vec<i64> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let count: jsize = je.get_array_length(*object)?;
        je.exception_check()?;
        let mut rval = vec![0_i64; count as usize];
        let slice: &mut [i64] = &mut rval;
        je.get_long_array_region(*object, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}

impl ConvertJValueToRust<'_, '_> for Vec<f32> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let count: jsize = je.get_array_length(*object)?;
        je.exception_check()?;
        let mut rval = vec![0 as f32; count as usize];
        let slice: &mut [f32] = &mut rval;
        je.get_float_array_region(*object, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}

impl ConvertJValueToRust<'_, '_> for Vec<f64> {
    fn to_rust(je: &JNIEnv, val: JValue) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let count: jsize = je.get_array_length(*object)?;
        je.exception_check()?;
        let mut rval = vec![0 as f64; count as usize];
        let slice: &mut [f64] = &mut rval;
        je.get_double_array_region(*object, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}

/// does not free the resources referenced by src
pub fn convert_jvalue_list_or_array_to_rust<'a, 'b, T>(
    je: &'b JNIEnv<'a>,
    src: JObject<'a>,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust<'a, 'b>,
{
    //println!("convert_jvalue_list_or_array_to_rust");

    let cls = je.get_object_class(src)?;
    if class_is_array(je, &cls)? {
        convert_jarray_to_rust(je, src)
    } else {
        convert_iterable_to_rust_vec(je, src)
    }
}

pub fn convert_jarray_to_rust<'a, 'b, T>(
    je: &'b JNIEnv<'a>,
    array: JObject,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust<'a, 'b>,
{
    let count = je.get_array_length(*array)?;
    let mut rval: Vec<T> = Vec::new();
    for i in 0..count {
        let obj_i = je.get_object_array_element(*array, i)?;
        let val: T = T::to_rust(je, JValue::from(obj_i))?;
        rval.push(val);
    }
    Ok(rval)
}

pub fn convert_iterable_to_rust_vec<'a, 'b, T>(
    je: &'b JNIEnv<'a>,
    iterable: JObject<'a>,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust<'a, 'b>,
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
        let val: T = T::to_rust(je, val)?;
        rval.push(val);
    }

    je.delete_local_ref(iter)?;

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
impl<T> JValueNonScalar for &[T] {}

impl<'a, 'b, T: JValueNonScalar + ConvertJValueToRust<'a, 'b>> ConvertJValueToRust<'a, 'b>
    for Vec<T>
{
    fn to_rust(je: &'b JNIEnv<'a>, val: JValue<'a>) -> Result<Self, jni::errors::Error> {
        let jobject: JObject<'a> = val.l()?;
        let rval = convert_jvalue_list_or_array_to_rust(je, jobject)?;
        je.delete_local_ref(jobject)?;

        Ok(rval)
    }
}

//

/// In most cases the type of T should be AutoLocal<'a,'b>
pub trait ConvertRustToJValue<'a, 'b> {
    type T;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<Self::T, jni::errors::Error>;
    // tmp is borrowed, so that the value doesn't get dropped before the temporary is used.
    fn temporary_into_jvalue(tmp: &Self::T) -> JValue<'a>;
}

#[macro_export]
macro_rules! impl_convert_rust_to_jvalue {
    ( $($t:ty),* ) => {
    $( impl<'a,'b> ConvertRustToJValue<'a, 'b> for $t
    {
        type T=$t;
        fn into_temporary(self, _je:&'b JNIEnv<'a>) ->Result<$t, jni::errors::Error> { Ok(self) }
        fn temporary_into_jvalue(tmp: &$t) -> JValue<'a>
        {
        (*tmp).into()
        }
    }
    impl<'a,'b> ConvertRustToJValue<'a, 'b> for &$t
        {
            type T=$t;
            fn into_temporary(self, _je:&'b JNIEnv<'a>) ->Result<$t, jni::errors::Error> { Ok(*self) }
            fn temporary_into_jvalue(tmp: &$t) -> JValue<'a>
            {
            (*tmp).into()
            }
        }
        ) *
    }
}

impl_convert_rust_to_jvalue! { i8, i16, i32, i64, f32, f64 }

impl<'a, 'b> ConvertRustToJValue<'a, 'b> for char {
    type T = char;
    fn into_temporary(self, _je: &'b JNIEnv<'a>) -> Result<char, jni::errors::Error> {
        Ok(self)
    }
    fn temporary_into_jvalue(tmp: &char) -> JValue<'a> {
        JValue::Char((*tmp) as u16)
    }
}

impl<'a, 'b> ConvertRustToJValue<'a, 'b> for bool {
    type T = bool;
    fn into_temporary(self, _je: &'b JNIEnv<'a>) -> Result<bool, jni::errors::Error> {
        Ok(self)
    }
    fn temporary_into_jvalue(tmp: &bool) -> JValue<'a> {
        JValue::Bool((*tmp) as u8)
    }
}

macro_rules! impl_convert_rust_vec_to_jvalue {
( $($t:ty ), *) => {
$(
impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &Vec<$t> {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        <&[$t] as ConvertRustToJValue>::into_temporary(&self, je) // delegate to the slice
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}
 impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for Vec<$t> {
     type T = AutoLocal<'a, 'b>;
     fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
         <&[$t] as ConvertRustToJValue>::into_temporary(&self, je) // delegate to the slice
     }
     fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
         JValue::from(tmp.as_obj())
     }
 }
 )*
};
}

impl_convert_rust_vec_to_jvalue! {bool, char, u8, i8, i16, i32, i64, f32, f64 }

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[bool] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval: jbooleanArray = copy_bool_array_to_jbooleanarray(je, self)?;
        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[char] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval = copy_char_array_to_jchararray(je, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[i8] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let shenanigans = unsafe { &*(self as *const [i8] as *const [u8]) };
        let rval = je.byte_array_from_slice(shenanigans)?;
        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[u8] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval = je.byte_array_from_slice(self)?;
        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &str {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval = je.new_string(self)?;
        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for String {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval = je.new_string(&self)?;
        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &String {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval = je.new_string(self)?;
        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
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

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[i32] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval: jintArray = je.new_int_array(self.len() as jsize)?;
        je.set_int_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b, 'c> ConvertRustToJValue<'a, 'b> for &'c mut [i32] {
    type T = ArrayCopyBackInt<'a, 'b, 'c>;
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

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[i16] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval: jshortArray = je.new_short_array(self.len() as jsize)?;
        je.set_short_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[i64] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval: jlongArray = je.new_long_array(self.len() as jsize)?;
        je.set_long_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[f32] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval: jfloatArray = je.new_float_array(self.len() as jsize)?;
        je.set_float_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b> ConvertRustToJValue<'a, 'b> for &[f64] {
    type T = AutoLocal<'a, 'b>;
    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<AutoLocal<'a, 'b>, jni::errors::Error> {
        let rval: jdoubleArray = je.new_double_array(self.len() as jsize)?;
        je.set_double_array_region(rval, 0, self)?;

        Ok(AutoLocal::new(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue(tmp: &AutoLocal<'a, 'b>) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b, 'c> ConvertRustToJValue<'a, 'b> for &'c mut [bool] {
    type T = ArrayCopyBackBool<'a, 'b, 'c>;
    fn into_temporary(
        self,
        je: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackBool<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackBool::new(self, je)
    }
    fn temporary_into_jvalue(tmp: &ArrayCopyBackBool<'a, 'b, 'c>) -> JValue<'a> {
        tmp.as_jvalue()
    }
}

impl<'a: 'b, 'b, 'c> ConvertRustToJValue<'a, 'b> for &'c mut [char] {
    type T = ArrayCopyBackChar<'a, 'b, 'c>;
    fn into_temporary(
        self,
        je: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackChar<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackChar::new(self, je)
    }
    fn temporary_into_jvalue(tmp: &ArrayCopyBackChar<'a, 'b, 'c>) -> JValue<'a> {
        tmp.as_jvalue()
    }
}

impl<'a: 'b, 'b, 'c> ConvertRustToJValue<'a, 'b> for &'c mut [i16] {
    type T = ArrayCopyBackShort<'a, 'b, 'c>;
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

impl<'a: 'b, 'b, 'c> ConvertRustToJValue<'a, 'b> for &'c mut [i8] {
    type T = ArrayCopyBackByte<'a, 'b, 'c>;
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

impl<'a: 'b, 'b, 'c> ConvertRustToJValue<'a, 'b> for &'c mut [i64] {
    type T = ArrayCopyBackLong<'a, 'b, 'c>;
    fn into_temporary(
        self,
        je: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackLong<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackLong::new(self, je)
    }
    fn temporary_into_jvalue(tmp: &ArrayCopyBackLong<'a, 'b, 'c>) -> JValue<'a> {
        tmp.as_jvalue()
    }
}

impl<'a: 'b, 'b, 'c> ConvertRustToJValue<'a, 'b> for &'c mut [f32] {
    type T = ArrayCopyBackFloat<'a, 'b, 'c>;
    fn into_temporary(
        self,
        je: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackFloat<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackFloat::new(self, je)
    }
    fn temporary_into_jvalue(tmp: &ArrayCopyBackFloat<'a, 'b, 'c>) -> JValue<'a> {
        tmp.as_jvalue()
    }
}

impl<'a: 'b, 'b, 'c> ConvertRustToJValue<'a, 'b> for &'c mut [f64] {
    type T = ArrayCopyBackDouble<'a, 'b, 'c>;
    fn into_temporary(
        self,
        je: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackDouble<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackDouble::new(self, je)
    }
    fn temporary_into_jvalue(tmp: &ArrayCopyBackDouble<'a, 'b, 'c>) -> JValue<'a> {
        tmp.as_jvalue()
    }
}

impl<'a: 'b, 'b, T> ConvertRustToJValue<'a, 'b> for Vec<T>
where
    T: ConvertRustToJValue<'a, 'b, T = AutoLocal<'a, 'b>> + JavaSignatureFor + JValueNonScalar,
{
    type T = AutoLocal<'a, 'b>;

    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<Self::T, Error> {
        let cls = je.find_class(T::signature_for())?;
        let rval: jobjectArray = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        for (i, val) in self.into_iter().enumerate() {
            let x: AutoLocal<'a, 'b> = val.into_temporary(je)?;
            let x: JObject = x.as_obj();
            je.set_object_array_element(rval, i as i32, x)?;
        }
        Ok(AutoLocal::new(je, JObject::from(rval)))
    }

    fn temporary_into_jvalue(tmp: &Self::T) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

impl<'a: 'b, 'b, T> ConvertRustToJValue<'a, 'b> for &[T]
where
    T: ConvertRustToJValue<'a, 'b, T = AutoLocal<'a, 'b>>
        + JavaSignatureFor
        + JValueNonScalar
        + Copy,
{
    type T = AutoLocal<'a, 'b>;

    fn into_temporary(self, je: &'b JNIEnv<'a>) -> Result<Self::T, Error> {
        let cls = je.find_class(T::signature_for())?;
        let rval: jobjectArray = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        for (i, val) in self.iter().enumerate() {
            let x: AutoLocal<'a, 'b> = val.into_temporary(je)?;
            let x: JObject = x.as_obj();
            je.set_object_array_element(rval, i as i32, x)?;
        }
        Ok(AutoLocal::new(je, JObject::from(rval)))
    }

    fn temporary_into_jvalue(tmp: &Self::T) -> JValue<'a> {
        JValue::from(tmp.as_obj())
    }
}

//

//

pub trait JavaConstructible<'a, 'b> {
    fn wrap_jobject(jni_env: &'b JNIEnv<'a>, java_this: AutoLocal<'a, 'b>) -> Self;
}

//

///
/// This creates a trivial rust struct for wrapping a java object reference.
/// The struct will have two fields:
///
/// * `java_this` will be an AutoLocal
/// * `jni_env` is the reference to the JNI environment the object lives in
///
///  The structure is just right for use with the
/// `jni_constructor!`, `jni_instance_method!`, and `static_method!` macros.
///
/// usage:
///
/// ` jni_wrapper_cliche! { rust_type_name, "package/path/to/java/class" }`
///
#[macro_export]
macro_rules! jni_wrapper_cliche_impl {
    ($ty:ident, $java_class_slash:literal) => {
        pub struct $ty<'a: 'b, 'b> {
            #[allow(dead_code)]
            java_this: jni::objects::AutoLocal<'a, 'b>,
            #[allow(dead_code)]
            jni_env: &'b jni::JNIEnv<'a>,
        }

        impl<'a, 'b> $ty<'a, 'b> {
            pub fn null(jni_env: &'b jni::JNIEnv<'a>) -> $ty<'a, 'b> {
                $ty {
                    java_this: jni::objects::AutoLocal::new(jni_env, jni::objects::JObject::null()),
                    jni_env,
                }
            }
        }

        impl<'a, 'b> $crate::JValueNonScalar for $ty<'a, 'b> {}

        impl<'a, 'b> jni_boilerplate_helper::JavaClassNameFor for $ty<'a, 'b> {
            fn java_class_name() -> &'static str {
                $java_class_slash
            }
        }

        impl<'a, 'b> $crate::JavaConstructible<'a, 'b> for $ty<'a, 'b> {
            fn wrap_jobject(
                jni_env: &'b jni::JNIEnv<'a>,
                java_this: jni::objects::AutoLocal<'a, 'b>,
            ) -> Self {
                $ty { java_this, jni_env }
            }
        }

        impl<'a, 'b> $crate::JavaSignatureFor for $ty<'a, 'b> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a, 'b> $crate::JavaSignatureFor for &$ty<'a, 'b> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a: 'b, 'b> $crate::ConvertRustToJValue<'a, 'b> for &$ty<'a, 'b> {
            type T = jni::sys::jobject;
            fn into_temporary(
                self,
                _je: &'b jni::JNIEnv<'a>,
            ) -> Result<jni::sys::jobject, jni::errors::Error> {
                Ok(*self.java_this.as_obj())
            }

            fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a> {
                jni::objects::JValue::from(*tmp)
            }
        }

        impl<'a: 'b, 'b> $crate::ConvertRustToJValue<'a, 'b> for $ty<'a, 'b> {
            type T = jni::sys::jobject;
            fn into_temporary(
                self,
                _je: &'b jni::JNIEnv<'a>,
            ) -> Result<jni::sys::jobject, jni::errors::Error> {
                Ok(*self.java_this.as_obj())
            }

            fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a> {
                jni::objects::JValue::from(*tmp)
            }
        }

        impl<'a: 'b, 'b> $crate::ConvertJValueToRust<'a, 'b> for $ty<'a, 'b> {
            fn to_rust(
                jni_env: &'b jni::JNIEnv<'a>,
                val: jni::objects::JValue<'a>,
            ) -> Result<Self, jni::errors::Error> {
                Ok($ty {
                    java_this: jni::objects::AutoLocal::new(jni_env, val.l()?),
                    jni_env,
                })
            }
        }
    };
}

#[macro_export]
macro_rules! jni_wrapper_cliche_impl_T {
    ($ty:ident, $java_class_slash:literal) => {
        pub struct $ty<'a: 'b, 'b, T: ConvertJValueToRust<'a, 'b>> {
            #[allow(dead_code)]
            java_this: jni::objects::AutoLocal<'a, 'b>,
            #[allow(dead_code)]
            jni_env: &'b jni::JNIEnv<'a>,
            phantom: PhantomData<T>,
        }

        impl<'a, 'b, T: ConvertJValueToRust<'a, 'b>> $ty<'a, 'b, T> {
            pub fn null(jni_env: &'b jni::JNIEnv<'a>) -> $ty<'a, 'b, T> {
                $ty {
                    java_this: jni::objects::AutoLocal::new(jni_env, jni::objects::JObject::null()),
                    jni_env,
                    phantom: PhantomData,
                }
            }
        }

        impl<'a, 'b, T: ConvertJValueToRust<'a, 'b>> $crate::JValueNonScalar for $ty<'a, 'b, T> {}

        impl<'a, 'b, T: ConvertJValueToRust<'a, 'b>> jni_boilerplate_helper::JavaClassNameFor
            for $ty<'a, 'b, T>
        {
            fn java_class_name() -> &'static str {
                $java_class_slash
            }
        }

        impl<'a, 'b, T: ConvertJValueToRust<'a, 'b>> $crate::JavaConstructible<'a, 'b>
            for $ty<'a, 'b, T>
        {
            fn wrap_jobject(
                jni_env: &'b jni::JNIEnv<'a>,
                java_this: jni::objects::AutoLocal<'a, 'b>,
            ) -> Self {
                $ty {
                    java_this,
                    jni_env,
                    phantom: PhantomData,
                }
            }
        }

        impl<'a, 'b, T: ConvertJValueToRust<'a, 'b>> $crate::JavaSignatureFor for $ty<'a, 'b, T> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a, 'b, T: ConvertJValueToRust<'a, 'b>> $crate::JavaSignatureFor for &$ty<'a, 'b, T> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a: 'b, 'b, T: ConvertJValueToRust<'a, 'b>> $crate::ConvertRustToJValue<'a, 'b>
            for &$ty<'a, 'b, T>
        {
            type T = jni::sys::jobject;
            fn into_temporary(
                self,
                _je: &'b jni::JNIEnv<'a>,
            ) -> Result<jni::sys::jobject, jni::errors::Error> {
                Ok(*self.java_this.as_obj())
            }

            fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a> {
                jni::objects::JValue::from(*tmp)
            }
        }

        impl<'a: 'b, 'b, T: ConvertJValueToRust<'a, 'b>> $crate::ConvertRustToJValue<'a, 'b>
            for $ty<'a, 'b, T>
        {
            type T = jni::sys::jobject;
            fn into_temporary(
                self,
                _je: &'b jni::JNIEnv<'a>,
            ) -> Result<jni::sys::jobject, jni::errors::Error> {
                Ok(*self.java_this.as_obj())
            }

            fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a> {
                jni::objects::JValue::from(*tmp)
            }
        }

        impl<'a: 'b, 'b, T: ConvertJValueToRust<'a, 'b>> $crate::ConvertJValueToRust<'a, 'b>
            for $ty<'a, 'b, T>
        {
            fn to_rust(
                jni_env: &'b jni::JNIEnv<'a>,
                val: jni::objects::JValue<'a>,
            ) -> Result<Self, jni::errors::Error> {
                Ok($ty {
                    java_this: jni::objects::AutoLocal::new(jni_env, val.l()?),
                    jni_env,
                    phantom: PhantomData,
                })
            }
        }
    };
}

//

pub fn panic_if_bad_sigs(sigs: &[String]) {
    for sig in sigs {
        if sig.contains('.') {
            panic!(
                "bad class signature {} contains a . (should probably be /, maybe $)",
                sig
            );
        }
    }
}

/*
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
*/

/*
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
*/
/*
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
*/
/*
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
*/

//

/*
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
*/
