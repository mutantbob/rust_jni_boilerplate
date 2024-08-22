#![allow(clippy::wrong_self_convention)]

use log::debug;

use crate::array_copy_back::*;
use java_runtime_wrappers::class_is_array;
use java_runtime_wrappers::Throwable;
use jni::errors::Error;
use jni::objects::{
    AutoLocal, JBooleanArray, JByteArray, JCharArray, JClass, JDoubleArray, JFloatArray, JIntArray,
    JLongArray, JObject, JObjectArray, JShortArray, JValue, JValueOwned,
};
use jni::sys::{jboolean, jobject, jsize};
use jni::JNIEnv;

pub mod array_copy_back;
pub mod java_runtime_wrappers;

pub struct JClassWrapper<'a, 'b> {
    pub jni_env: &'a JNIEnv<'a>,
    pub cls: JClass<'b>,
}

impl<'a, 'b> Drop for JClassWrapper<'a, 'b> {
    fn drop(&mut self) {
        let res = self.jni_env.delete_local_ref(self.cls);
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

impl JavaClassNameFor for &[i8] {
    fn java_class_name() -> &'static str {
        "[B"
    }
}

/*impl<T: JavaClassNameFor> JavaClassNameFor for &[T] {
    fn java_class_name() -> &'static str {
        format!("[{}", <T as JavaClassNameFor>::java_class_name())
    }
}*/

//

///
/// part of the job of the implementation of the to_rust() method is to release the resources
/// held by the JValue to prevent memory leaks
/// (which is probably anything where the JValue has a jobject returned by .l() )
pub trait ConvertJValueToRust
where
    Self: std::marker::Sized,
{
    fn to_rust<'a, 'b>(
        je: &'b mut JNIEnv<'a>,
        val: JValueOwned<'a>,
    ) -> Result<Self, jni::errors::Error>;
}

impl ConvertJValueToRust for () {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.v()
    }
}

impl ConvertJValueToRust for bool {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.z()
    }
}
impl ConvertJValueToRust for char {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.c().and_then(|c| match std::char::from_u32(c as u32) {
            None => Err(java_exception()),
            Some(ch) => Ok(ch),
        })
    }
}

pub fn java_exception() -> Error {
    jni::errors::Error::JavaException
}

impl ConvertJValueToRust for i8 {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.b()
    }
}

impl ConvertJValueToRust for i16 {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.s()
    }
}

impl ConvertJValueToRust for i32 {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.i()
    }
}

impl ConvertJValueToRust for i64 {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.j()
    }
}

impl ConvertJValueToRust for f32 {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.f()
    }
}

impl ConvertJValueToRust for f64 {
    fn to_rust<'a, 'b>(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.d()
    }
}

impl ConvertJValueToRust for String {
    fn to_rust<'a, 'b>(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        let obj = val.l()?;
        let x = je.get_string((&obj).into())?;
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

impl ConvertJValueToRust for Vec<bool> {
    fn to_rust<'a, 'b>(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let array: JBooleanArray = object.into();
        let count: jsize = je.get_array_length(&array)?;
        je.exception_check()?;
        #[allow(clippy::unnecessary_cast)]
        let mut rval = vec![0 as jboolean; count as usize];
        let slice: &mut [jboolean] = &mut rval;
        je.get_boolean_array_region(&array, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(array) {
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
        Err(java_exception())
    }
}

impl ConvertJValueToRust for Vec<char> {
    fn to_rust<'a, 'b>(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let array: JCharArray = object.into();
        let count = je.get_array_length(&array)?;
        let mut rval = vec![0 as char; count as usize];
        move_jchararray_to_char_array(je, array, &mut rval)?;

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

impl ConvertJValueToRust for Vec<i8> {
    fn to_rust<'a, 'b>(
        je: &'b mut JNIEnv<'a>,
        val: JValueOwned,
    ) -> Result<Self, jni::errors::Error> {
        let tmp: Vec<u8> = Vec::<u8>::to_rust(je, val)?;

        Ok(vec_u8_into_i8(tmp))
    }
}

impl ConvertJValueToRust for Vec<u8> {
    fn to_rust<'a, 'b>(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let array: JByteArray = object.into();
        let rval = je.convert_byte_array(&array);
        je.exception_check()?;
        //println!("delete_local_ref()");
        if let Err(e) = je.delete_local_ref(array) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        rval
    }
}
/*
impl ConvertJValueToRust for Vec<i16> {
    fn to_rust<'a, 'b>(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let array: JShortArray = object.into();
        let count: jsize = je.get_array_length(&array)?;
        je.exception_check()?;
        let mut rval = vec![0_i16; count as usize];
        let slice: &mut [i16] = &mut rval;
        je.get_short_array_region(array, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}

impl ConvertJValueToRust for Vec<i32> {
    fn to_rust<'a, 'b>(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let array: JIntArray = object.into();
        let count: jsize = je.get_array_length(&array)?;
        je.exception_check()?;
        let mut rval = vec![0_i32; count as usize];
        let slice: &mut [i32] = &mut rval;
        je.get_int_array_region(array, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}

impl ConvertJValueToRust for Vec<i64> {
    fn to_rust<'a, 'b>(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let array: JLongArray = object.into();
        let count: jsize = je.get_array_length(&array)?;
        je.exception_check()?;
        let mut rval = vec![0_i64; count as usize];
        let slice: &mut [i64] = &mut rval;
        je.get_long_array_region(array, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(object) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}
*/

macro_rules! convert_jvalue_to_rust_impl_vec {
    ($scalar:ty, $j_array:ty, $j_get_function:ident) => {
        impl ConvertJValueToRust for Vec<$scalar> {
            fn to_rust<'a, 'b>(
                je: &mut JNIEnv,
                val: JValueOwned,
            ) -> Result<Self, jni::errors::Error> {
                let object: JObject = val.l()?;
                let array: $j_array = object.into();
                let count: jsize = je.get_array_length(&array)?;
                je.exception_check()?;
                let mut rval = vec![0 as $scalar; count as usize];
                let slice: &mut [$scalar] = &mut rval;
                je.$j_get_function(&array, 0, slice)?;
                je.exception_check()?;
                if let Err(e) = je.delete_local_ref(array) {
                    debug!("jni failed to delete_local_ref() : {:?}", e)
                }
                Ok(rval)
            }
        }
    };
}

convert_jvalue_to_rust_impl_vec! {i16, JShortArray, get_short_array_region}
convert_jvalue_to_rust_impl_vec! {i32, JIntArray, get_int_array_region}
convert_jvalue_to_rust_impl_vec! {i64, JLongArray, get_long_array_region}
convert_jvalue_to_rust_impl_vec! {f32, JFloatArray, get_float_array_region}
convert_jvalue_to_rust_impl_vec! {f64, JDoubleArray, get_double_array_region}

/*impl ConvertJValueToRust for Vec<f64> {
    fn to_rust<'a, 'b>(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        let object: JObject = val.l()?;
        let array: JDoubleArray = object.into();
        let count: jsize = je.get_array_length(&array)?;
        je.exception_check()?;
        let mut rval = vec![0 as f64; count as usize];
        let slice: &mut [f64] = &mut rval;
        je.get_double_array_region(&array, 0, slice)?;
        je.exception_check()?;
        if let Err(e) = je.delete_local_ref(array) {
            debug!("jni failed to delete_local_ref() : {:?}", e)
        }
        Ok(rval)
    }
}
*/
/// does not free the resources referenced by src
pub fn convert_jvalue_list_or_array_to_rust<'a, 'b, T>(
    je: &'b mut JNIEnv<'a>,
    src: JObject<'a>,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust,
{
    //println!("convert_jvalue_list_or_array_to_rust");

    let cls = je.get_object_class(&src)?;
    if class_is_array(je, &cls)? {
        convert_jarray_to_rust(je, src)
    } else {
        convert_iterable_to_rust_vec(je, src)
    }
}

pub fn convert_jarray_to_rust<'a, 'b, T>(
    je: &'b mut JNIEnv<'a>,
    array: JObject,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust,
{
    let array: JObjectArray = array.into();
    let count = je.get_array_length(&array)?;
    let mut rval: Vec<T> = Vec::new();
    for i in 0..count {
        let obj_i = je.get_object_array_element(&array, i)?;
        let val: T = T::to_rust(je, JValueOwned::from(obj_i))?;
        rval.push(val);
    }
    Ok(rval)
}

pub fn convert_iterable_to_rust_vec<'a, 'b, T>(
    je: &'b mut JNIEnv<'a>,
    iterable: JObject<'a>,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust,
{
    let iter = je.call_method(iterable, "iterator", "()Ljava/util/Iterator;", &[])?;
    let iter = iter.l()?;

    let mut rval: Vec<T> = Vec::new();
    loop {
        let has_next = je.call_method(&iter, "hasNext", "()Z", &[])?;
        if !has_next.z()? {
            break;
        }
        let val = je.call_method(&iter, "next", "()Ljava/lang/Object;", &[])?;
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
    fn to_rust(je: &mut JNIEnv, val: &JValue) -> Result<Self, jni::errors::Error> {
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

impl<T: JValueNonScalar + ConvertJValueToRust> ConvertJValueToRust for Vec<T> {
    fn to_rust<'a, 'b>(
        je: &'b mut JNIEnv<'a>,
        val: JValueOwned<'a>,
    ) -> Result<Self, jni::errors::Error> {
        let jobject: JObject<'a> = val.l()?;
        let rval = convert_jvalue_list_or_array_to_rust(je, jobject)?;
        // je.delete_local_ref(val)?; XXX is this a leak?

        Ok(rval)
    }
}

//

/// In most cases the type of T should be AutoLocal<'a,'b>
pub trait ConvertRustToJValue<'a> {
    type T;
    fn into_temporary<'b>(&self, je: &mut JNIEnv<'a>) -> Result<Self::T<'a>, jni::errors::Error>;
    // tmp is borrowed, so that the value doesn't get dropped before the temporary is used.
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's>;
}

pub trait ConvertMutableRustToJValue {
    type T<'a: 'b, 'b>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<Self::T<'a, 'b>, jni::errors::Error>;
    // tmp is borrowed, so that the value doesn't get dropped before the temporary is used.
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's>;
}

#[macro_export]
macro_rules! impl_convert_rust_to_jvalue {
    ( $($t:ty),* ) => {
    $( impl ConvertRustToJValue for $t
    {
        type T<'a:'b,'b>=$t;
        fn into_temporary<'a, 'b>(&self, _je: &'b mut JNIEnv<'a>) ->Result<$t, jni::errors::Error> { Ok(*self) }
        fn temporary_into_jvalue<'a:'b, 'b, 's>(tmp: &'s $t) -> JValue<'a,'s>
        {
        (*tmp).into()
        }
    }
    impl ConvertRustToJValue for &$t
        {
            type T<'a:'b, 'b>=$t;
            fn into_temporary<'a, 'b>(&self, _je: &'b mut JNIEnv<'a>) ->Result<$t, jni::errors::Error> { Ok(**self) }
            fn temporary_into_jvalue<'a:'b, 'b, 's>(tmp: &'s $t) -> JValue<'a,'s>
            {
            (*tmp).into()
            }
        }
        ) *
    }
}

impl_convert_rust_to_jvalue! { i8, i16, i32, i64, f32, f64 }

impl ConvertRustToJValue for char {
    type T<'a: 'b, 'b> = char;
    fn into_temporary<'a, 'b>(&self, _je: &'b mut JNIEnv<'a>) -> Result<char, jni::errors::Error> {
        Ok(*self)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::Char((*tmp) as u16)
    }
}

impl ConvertRustToJValue for bool {
    type T<'a: 'b, 'b> = bool;
    fn into_temporary<'a, 'b>(&self, _je: &'b mut JNIEnv<'a>) -> Result<bool, jni::errors::Error> {
        Ok(*self)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::Bool((*tmp) as u8)
    }
}

/// I would like to get rid of these, but I haven't figured out exactly how to
/// generically delegate from something to a &'c[T] without rust becoming confused
macro_rules! impl_convert_rust_vec_to_jvalue {
( $($t:ty ), *) => {
$(
/*impl ConvertRustToJValue for &Vec<$t> {
    type T<'a:'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T, jni::errors::Error> {
        <&[$t] as ConvertRustToJValue>::into_temporary(&self.as_slice(), je) // delegate to the slice
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}*/
 impl ConvertRustToJValue for Vec<$t> {
     type T<'a:'b, 'b> = AutoLocal<'a, JObject<'a>>;
     fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a,'b>, jni::errors::Error> {
         <&[$t] as ConvertRustToJValue>::into_temporary(&self.as_slice(), je) // delegate to the slice
     }
     fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
         JValue::from(tmp)
     }
 }
 )*
};
}

impl_convert_rust_vec_to_jvalue! {bool, char, u8, i8, i16, i32, i64, f32, f64 }

impl ConvertRustToJValue for &[bool] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = copy_bool_array_to_jbooleanarray(je, self)?;
        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

pub fn wrap_jobject<'a>(rval: jobject) -> JObject<'a> {
    unsafe { JObject::from_raw(rval) }
}

impl ConvertRustToJValue for &[char] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = copy_char_array_to_jchararray(je, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl ConvertRustToJValue for &[i8] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let shenanigans = unsafe { &*((*self) as *const [i8] as *const [u8]) };
        let rval = je.byte_array_from_slice(shenanigans)?;
        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl ConvertRustToJValue for &[u8] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.byte_array_from_slice(self)?;
        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl ConvertRustToJValue for &str {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_string(self)?;
        Ok(JNIEnv::auto_local(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl ConvertRustToJValue for String {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_string(self)?;
        Ok(JNIEnv::auto_local(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl ConvertRustToJValue for &String {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_string(self)?;
        Ok(JNIEnv::auto_local(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

/*
impl<'a, 'b, T> ConvertRustToJValue<'a, 'b, AutoLocal<'a, JObject<'a>>> for &[T]
{
    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<AutoLocal<'a, JObject<'a>>, Error> {
        let cls =
        je.new_object_array(self.len() , )
    }

    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        unimplemented!()
    }
}
*/

impl ConvertRustToJValue for &[i32] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_int_array(self.len() as jsize)?;
        je.set_int_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'c> ConvertMutableRustToJValue for &'c mut [i32] {
    type T<'a: 'b, 'b> = ArrayCopyBackInt<'a, 'b, 'c>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<ArrayCopyBackInt<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackInt::new(self, je)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        tmp.as_jvalue()
    }
}

impl ConvertRustToJValue for &[i16] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_short_array(self.len() as jsize)?;
        je.set_short_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl ConvertRustToJValue for &[i64] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_long_array(self.len() as jsize)?;
        je.set_long_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl ConvertRustToJValue for &[f32] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_float_array(self.len() as jsize)?;
        je.set_float_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

fn bacon<'a: 'b, 'b>(
    x: &[f64],
    je: &'b mut JNIEnv<'a>,
) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
    let rval = je.new_double_array(x.len() as jsize)?;
    je.set_double_array_region(&rval, 0, x)?;

    Ok(JNIEnv::auto_local(je, rval.into()))
}

impl ConvertRustToJValue for &[f64] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a, 'b>(
        &self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_double_array(self.len() as jsize)?;
        je.set_double_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'c> ConvertMutableRustToJValue for &'c mut [bool] {
    type T<'a: 'b, 'b> = ArrayCopyBackBool<'a, 'b, 'c>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<ArrayCopyBackBool<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackBool::new(self, je)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        tmp.as_jvalue()
    }
}

impl<'c> ConvertMutableRustToJValue for &'c mut [char] {
    type T<'a: 'b, 'b> = ArrayCopyBackChar<'a, 'b, 'c>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<ArrayCopyBackChar<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackChar::new(self, je)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        tmp.as_jvalue()
    }
}

impl<'c> ConvertMutableRustToJValue for &'c mut [i16] {
    type T<'a: 'b, 'b> = ArrayCopyBackShort<'a, 'b, 'c>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<ArrayCopyBackShort<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackShort::new(self, je)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        tmp.as_jvalue()
    }
}

impl<'c> ConvertMutableRustToJValue for &'c mut [i8] {
    type T<'a: 'b, 'b> = ArrayCopyBackByte<'a, 'b, 'c>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<ArrayCopyBackByte<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackByte::new(self, je)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        tmp.as_jvalue()
    }
}

impl<'c> ConvertMutableRustToJValue for &'c mut [i64] {
    type T<'a: 'b, 'b> = ArrayCopyBackLong<'a, 'b, 'c>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<ArrayCopyBackLong<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackLong::new(self, je)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        tmp.as_jvalue()
    }
}

impl<'c> ConvertMutableRustToJValue for &'c mut [f32] {
    type T<'a: 'b, 'b> = ArrayCopyBackFloat<'a, 'b, 'c>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<ArrayCopyBackFloat<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackFloat::new(self, je)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        tmp.as_jvalue()
    }
}

impl<'c> ConvertMutableRustToJValue for &'c mut [f64] {
    type T<'a: 'b, 'b> = ArrayCopyBackDouble<'a, 'b, 'c>;
    fn into_temporary<'a, 'b>(
        self,
        je: &'b mut JNIEnv<'a>,
    ) -> Result<ArrayCopyBackDouble<'a, 'b, 'c>, jni::errors::Error> {
        ArrayCopyBackDouble::new(self, je)
    }
    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        tmp.as_jvalue()
    }
}

macro_rules! atrocious_lifetime_kludge {
    ($je:expr) => {
        unsafe { &mut *($je as *mut _) }
    };
}

impl<S> ConvertRustToJValue for Vec<S>
where
    S: ConvertRustToJValue + JavaClassNameFor + JValueNonScalar,
{
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;

    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
        let cls = je.find_class(S::java_class_name())?;
        let rval = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        let jem = atrocious_lifetime_kludge!(je);
        for (i, val) in self.iter().enumerate() {
            let tmp: <S as ConvertRustToJValue>::T<'_, '_> =
                <S as ConvertRustToJValue>::into_temporary(val, jem)?;
            let object = <S as ConvertRustToJValue>::temporary_into_jvalue(&tmp).l()?;
            je.set_object_array_element(&rval, i as i32, object)?;
        }
        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}
/*
impl<'a: 'b, 'b, 'c, T> ConvertRustToJValue for &Vec<T>
where
    &'c T: ConvertRustToJValue,
    T: JavaClassNameFor + JValueNonScalar,
    Self: 'c,
{
    type T<'a:'b, 'b> = AutoLocal<'a, JObject<'a>;

    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
        let cls = je.find_class(T::java_class_name())?;
        let rval = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        for (i, val) in self.iter().enumerate() {
            let tmp: <T as ConvertRustToJValue>::T =
                <T as ConvertRustToJValue>::into_temporary(val, je)?;
            let object = <T as ConvertRustToJValue>::temporary_into_jvalue(&tmp).l()?;
            je.set_object_array_element(rval, i as i32, object)?;
        }
        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue(tmp: &Self::T) -> JValue<'a,'a> {
        JValue::from(tmp)
    }
}
*/

// error[E0207]: the lifetime parameter `'c` is not constrained by the impl trait, self type, or predicates
//    --> /home/thoth/src/jni_boilerplate/jni_boilerplate_helper/src/lib.rs:910:18
//     |
// 910 | impl<'c, S> ConvertRustToJValue for Vec<S>
//     |                  ^^ unconstrained lifetime parameter
/*
impl<'c, S> ConvertRustToJValue for Vec<S>
where
    &'c [S]: ConvertRustToJValue<'a,'b>,
    Self: 'c,
{
    type T<'a:'b, 'b> = <&'c [S] as ConvertRustToJValue<'a,'b>>::T;

    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
        <&'c [S] as ConvertRustToJValue>::into_temporary(&self.as_slice(), je)
    }

    fn temporary_into_jvalue(tmp: &Self::T) -> JValue<'a,'a> {
        <&'c [S] as ConvertRustToJValue>::temporary_into_jvalue(tmp)
    }
}
*/

impl<'c, S> ConvertRustToJValue for &'c Vec<S>
where
    &'c [S]: ConvertRustToJValue,
    Self: 'c,
{
    type T<'a: 'b, 'b> = <&'c [S] as ConvertRustToJValue>::T<'a, 'b>;

    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
        <&'c [S] as ConvertRustToJValue>::into_temporary(&self.as_slice(), je)
    }

    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        <&'c [S] as ConvertRustToJValue>::temporary_into_jvalue(tmp)
    }
}

/*impl<S> ConvertRustToJValue for &[S]
where
    S: ConvertRustToJValue,
    S: JavaClassNameFor + JValueNonScalar + Copy,
{
    type T<'a:'b, 'b> = AutoLocal<'a, JObject<'a>;

    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
        let cls = je.find_class(S::java_class_name())?;
        let rval = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        for (i, val) in self.iter().enumerate() {
            let tmp: <S as ConvertRustToJValue>::T =
                <S as ConvertRustToJValue>::into_temporary(val, je)?;
            let t2 = <S as ConvertRustToJValue>::temporary_into_jvalue(&tmp);
            je.set_object_array_element(rval, i as i32, t2.l()?)?;
        }
        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue(tmp: &Self::T) -> JValue<'a,'a> {
        JValue::from(tmp)
    }
}
*/
impl<S> ConvertRustToJValue for &[S]
where
    S: ConvertRustToJValue,
    S: JavaClassNameFor + JValueNonScalar, // I need JValueNonScalar to void conflicting with &[i8] and friends
                                           // Self: 'c,
{
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;

    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
        let cls = je.find_class(S::java_class_name())?;
        let rval = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        let jem = {
            let ptr = je as *mut _;
            unsafe { &mut *(ptr) }
        };
        for (i, val) in self.iter().enumerate() {
            let x: &S = val;
            let tmp: <S as ConvertRustToJValue>::T<'_, '_> =
                <S as ConvertRustToJValue>::into_temporary(x, jem)?;
            let t2 = <S as ConvertRustToJValue>::temporary_into_jvalue(&tmp);
            let t3 = t2.l()?;
            je.set_object_array_element(&rval, i as i32, t3)?;
        }
        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

/// I need this because of how `<&[&[i8]] as ConvertRustToJValue>` depends on <&&[i8] as ConvertRustToJValue>
impl<'c, S> ConvertRustToJValue for &&'c [S]
where
    &'c [S]: ConvertRustToJValue,
{
    type T<'a: 'b, 'b> = <&'c [S] as ConvertRustToJValue>::T<'a, 'b>;

    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
        <&'c [S] as ConvertRustToJValue>::into_temporary(*self, je)
    }

    fn temporary_into_jvalue<'a: 'b, 'b, 's>(tmp: &'s Self::T<'a, 'b>) -> JValue<'a, 's> {
        <&'c [S] as ConvertRustToJValue>::temporary_into_jvalue(tmp)
    }
}

/// I'm not entirely sure why the impl above doesn't match &[&str],
/// and I'm in too much of a hurry to think it through.
impl ConvertRustToJValue for &[&str] {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;

    // fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
    fn into_temporary<'a, 'b>(&self, je: &'b mut JNIEnv<'a>) -> Result<Self::T<'a, 'b>, Error> {
        let cls = je.find_class(<&str as JavaClassNameFor>::java_class_name())?;
        let rval = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        for (i, val) in self.iter().enumerate() {
            let tmp: <&str as ConvertRustToJValue>::T<'a, 'b> =
                <&str as ConvertRustToJValue>::into_temporary(val, je)?;
            let t2 = <&str as ConvertRustToJValue>::temporary_into_jvalue(&tmp);
            je.set_object_array_element(&rval, i as i32, t2.l()?)?;
        }
        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    // fn temporary_into_jvalue<'a: 'b, 'b, 'c>(tmp: &'c Self::T<'a, 'b>) -> JValue<'a, 'c> {
    fn temporary_into_jvalue<'a: 'b, 'b, 'c>(tmp: &'c Self::T<'a, 'b>) -> JValue<'a, 'c> {
        JValue::from(tmp)
    }
}

//
//

pub trait JavaConstructible<'a, 'b> {
    fn wrap_jobject(jni_env: &'b JNIEnv<'a>, java_this: AutoLocal<'a, JObject<'a>>) -> Self;
}

//

pub fn raise_if_exception(jni_env: &mut JNIEnv) -> Result<(), Error> {
    match jni_env.exception_check() {
        Ok(boom) => {
            if boom {
                if let Ok(throwable) = jni_env.exception_occurred() {
                    jni_env.exception_clear()?;
                    #[cfg(debug_assertions)]
                    {
                        let jni_this: AutoLocal<JObject> =
                            JNIEnv::auto_local(jni_env, throwable.into());
                        let t2 = Throwable::wrap_jobject(jni_env, jni_this);
                        let _ = t2.printStackTrace(jni_env);
                    }
                } else {
                    jni_env.exception_clear()?;
                }
                Err(java_exception())
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
    }
}

//

/// This trait only exists so I can chain calls off a Result.
/// If the Result is an Err, and jni_env.exception_check() reports true, we will call jni_env.exception_clear()
pub trait ClearIfErr<T> {
    fn clear_if_err(self, jni_env: &mut JNIEnv) -> Result<T, Error>;
}

impl<T> ClearIfErr<T> for Result<T, Error> {
    fn clear_if_err(self, jni_env: &mut JNIEnv) -> Result<T, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                raise_if_exception(jni_env)?;
                Err(e)
            }
        }
    }
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
/// ` jni_wrapper_cliche_impl! { rust_type_name, "package/path/to/java/class" }`
///
#[macro_export]
macro_rules! jni_wrapper_cliche_impl {
    ($ty:ident, $java_class_slash:literal) => {
        pub struct $ty<'a: 'b, 'b> {
            #[allow(dead_code)]
            java_this: jni::objects::AutoLocal<'a, JObject>,
            #[allow(dead_code)]
            jni_env: &'b jni::JNIEnv<'a>,
        }

        impl<'a, 'b> $ty<'a, 'b> {
            pub fn null(jni_env: &'b jni::JNIEnv<'a>) -> $ty<'a, 'b> {
                $ty {
                    java_this: jni_env.auto_local(jni::objects::JObject::null()),
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
                java_this: jni::objects::AutoLocal<'a, JObject>,
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

        impl<'a: 'b, 'b> $crate::ConvertRustToJValue for &$ty<'a, 'b> {
            type T<'a: 'b, 'b> = jni::sys::jobject;
            fn into_temporary<'a, 'b>(
                &self,
                _je: &'b jni::JNIEnv<'a>,
            ) -> Result<jni::sys::jobject, jni::errors::Error> {
                Ok(*self.java_this.as_obj())
            }

            fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a, 'a> {
                jni::objects::JValue::from($crate::wrap_jobject(*tmp))
            }
        }

        impl<'a: 'b, 'b> $crate::ConvertRustToJValue for $ty<'a, 'b> {
            type T<'a: 'b, 'b> = jni::sys::jobject;
            fn into_temporary<'a, 'b>(
                &self,
                _je: &'b jni::JNIEnv<'a>,
            ) -> Result<jni::sys::jobject, jni::errors::Error> {
                Ok(*self.java_this.as_obj())
            }

            fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a, 'a> {
                jni::objects::JValue::from($crate::wrap_jobject(*tmp))
            }
        }

        impl<'a: 'b, 'b> $crate::ConvertJValueToRust for $ty<'a, 'b> {
            fn to_rust<'a, 'b>(
                jni_env: &'b jni::JNIEnv<'a>,
                val: jni::objects::JValue<'a, 'a>,
            ) -> Result<Self, jni::errors::Error> {
                Ok($ty {
                    java_this: JNIEnv::auto_local(jni_env, val.l()?),
                    jni_env,
                })
            }
        }
    };
}

#[macro_export]
macro_rules! jni_wrapper_cliche_impl_T {
    ($ty:ident, $java_class_slash:literal) => {
        pub struct $ty<'a: 'b, 'b, T: ConvertJValueToRust> {
            #[allow(dead_code)]
            java_this: jni::objects::AutoLocal<'a, JObject>,
            #[allow(dead_code)]
            jni_env: &'b jni::JNIEnv<'a>,
            phantom: PhantomData<T>,
        }

        impl<'a, 'b, T: ConvertJValueToRust> $ty<'a, 'b, T> {
            pub fn null(jni_env: &'b jni::JNIEnv<'a>) -> $ty<'a, 'b, T> {
                $ty {
                    java_this: JNIEnv::auto_local(jni_env, jni::objects::JObject::null()),
                    jni_env,
                    phantom: PhantomData,
                }
            }
        }

        impl<'a, 'b, T: ConvertJValueToRust> $crate::JValueNonScalar for $ty<'a, 'b, T> {}

        impl<'a, 'b, T: ConvertJValueToRust> jni_boilerplate_helper::JavaClassNameFor
            for $ty<'a, 'b, T>
        {
            fn java_class_name() -> &'static str {
                $java_class_slash
            }
        }

        impl<'a, 'b, T: ConvertJValueToRust> $crate::JavaConstructible<'a, 'b> for $ty<'a, 'b, T> {
            fn wrap_jobject(
                jni_env: &'b jni::JNIEnv<'a>,
                java_this: jni::objects::AutoLocal<'a, JObject>,
            ) -> Self {
                $ty {
                    java_this,
                    jni_env,
                    phantom: PhantomData,
                }
            }
        }

        impl<'a, 'b, T: ConvertJValueToRust> $crate::JavaSignatureFor for $ty<'a, 'b, T> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a, 'b, T: ConvertJValueToRust> $crate::JavaSignatureFor for &$ty<'a, 'b, T> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a: 'b, 'b, T: ConvertJValueToRust> $crate::ConvertRustToJValue for &$ty<'a, 'b, T> {
            type T<'a: 'b, 'b> = jni::sys::jobject;
            fn into_temporary<'a, 'b>(
                &self,
                _je: &'b jni::JNIEnv<'a>,
            ) -> Result<jni::sys::jobject, jni::errors::Error> {
                Ok(*self.java_this.as_obj())
            }

            fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a, 'a> {
                jni::objects::JValue::from($crate::wrap_jobject(*tmp))
            }
        }

        impl<'a: 'b, 'b, T: ConvertJValueToRust> $crate::ConvertRustToJValue for $ty<'a, 'b, T> {
            type T<'a: 'b, 'b> = jni::sys::jobject;
            fn into_temporary<'a, 'b>(
                &self,
                _je: &'b jni::JNIEnv<'a>,
            ) -> Result<jni::sys::jobject, jni::errors::Error> {
                Ok(*self.java_this.as_obj())
            }

            fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a, 'a> {
                jni::objects::JValue::from($crate::wrap_jobject(*tmp))
            }
        }

        impl<'a: 'b, 'b, T: ConvertJValueToRust> $crate::ConvertJValueToRust for $ty<'a, 'b, T> {
            fn to_rust<'a, 'b>(
                jni_env: &'b jni::JNIEnv<'a>,
                val: jni::objects::JValue<'a, 'a>,
            ) -> Result<Self, jni::errors::Error> {
                Ok($ty {
                    java_this: jni::objects::JNIEnv::auto_local(jni_env, val.l()?),
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
