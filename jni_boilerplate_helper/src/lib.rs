#![allow(clippy::wrong_self_convention)]

use crate::array_copy_back::*;
use java_runtime_wrappers::class_is_array;
use java_runtime_wrappers::Throwable;
pub use jni;
use jni::errors::Error;
use jni::objects::{
    AutoLocal, JBooleanArray, JByteArray, JCharArray, JClass, JDoubleArray, JFloatArray, JIntArray,
    JLongArray, JObject, JObjectArray, JShortArray, JValue, JValueOwned,
};
use jni::sys::{jboolean, jobject, jsize};
use jni::JNIEnv;
use log::debug;

pub mod array_copy_back;
pub mod atrocious_kludges;
pub mod java_runtime_wrappers;

pub struct JClassWrapper<'a, 'b> {
    jni_env: JNIEnv<'a>,
    pub cls: JClass<'b>,
}

impl<'a, 'b> JClassWrapper<'a, 'b> {
    pub fn new(env: &JNIEnv<'a>, cls: JClass<'b>) -> Self {
        let jni_env = unsafe { env.unsafe_clone() };

        Self { jni_env, cls }
    }
}

impl<'a, 'b> Drop for JClassWrapper<'a, 'b> {
    fn drop(&mut self) {
        let res = self.jni_env.delete_local_ref(kludge_take(&mut self.cls));
        match res {
            Ok(()) => {}
            Err(e) => debug!("error dropping global ref: {:#?}", e),
        }
    }
}

pub(crate) fn kludge_take<'a>(arg: &mut JClass<'a>) -> JClass<'a> {
    std::mem::replace(arg, unsafe { JClass::from_raw(std::ptr::null_mut()) })
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

pub trait ConvertJValueToRust<'a>
where
    Self: std::marker::Sized,
{
    fn to_rust(je: &mut JNIEnv<'a>, val: JValueOwned<'a>) -> Result<Self, jni::errors::Error>;
}

impl<'a> ConvertJValueToRust<'a> for () {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.v()
    }
}

impl<'a> ConvertJValueToRust<'a> for bool {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.z()
    }
}
impl<'a> ConvertJValueToRust<'a> for char {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.c().and_then(|c| match std::char::from_u32(c as u32) {
            None => Err(java_exception()),
            Some(ch) => Ok(ch),
        })
    }
}

pub fn java_exception() -> Error {
    jni::errors::Error::JavaException
}

impl<'a> ConvertJValueToRust<'a> for i8 {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.b()
    }
}

impl<'a> ConvertJValueToRust<'a> for i16 {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.s()
    }
}

impl<'a> ConvertJValueToRust<'a> for i32 {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.i()
    }
}

impl<'a> ConvertJValueToRust<'a> for i64 {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.j()
    }
}

impl<'a> ConvertJValueToRust<'a> for f32 {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.f()
    }
}

impl<'a> ConvertJValueToRust<'a> for f64 {
    fn to_rust(_je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
        val.d()
    }
}

impl<'a> ConvertJValueToRust<'a> for String {
    fn to_rust(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
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

impl<'a> ConvertJValueToRust<'a> for Vec<bool> {
    fn to_rust(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
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

impl<'a> ConvertJValueToRust<'a> for Vec<char> {
    fn to_rust(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
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

impl<'a> ConvertJValueToRust<'a> for Vec<i8> {
    fn to_rust(je: &mut JNIEnv<'a>, val: JValueOwned<'a>) -> Result<Self, jni::errors::Error> {
        let tmp: Vec<u8> = Vec::<u8>::to_rust(je, val)?;

        Ok(vec_u8_into_i8(tmp))
    }
}

impl<'a> ConvertJValueToRust<'a> for Vec<u8> {
    fn to_rust(je: &mut JNIEnv<'a>, val: JValueOwned) -> Result<Self, jni::errors::Error> {
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

macro_rules! convert_jvalue_to_rust_impl_vec {
    ($scalar:ty, $j_array:ty, $j_get_function:ident) => {
        impl<'a> ConvertJValueToRust<'a> for Vec<$scalar> {
            fn to_rust(je: &mut JNIEnv, val: JValueOwned) -> Result<Self, jni::errors::Error> {
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
/// does not free the resources referenced by src
pub fn convert_jvalue_list_or_array_to_rust<'a, 'b, T>(
    je: &'b mut JNIEnv<'a>,
    src: JObject<'a>,
) -> Result<Vec<T>, jni::errors::Error>
where
    T: ConvertJValueToRust<'a>,
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
    T: ConvertJValueToRust<'a>,
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
    T: ConvertJValueToRust<'a>,
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

pub trait JValueNonScalar {}

impl JValueNonScalar for String {}
impl<T> JValueNonScalar for Vec<T> {}
impl<T> JValueNonScalar for &[T] {}

impl<'a, T: JValueNonScalar + ConvertJValueToRust<'a>> ConvertJValueToRust<'a> for Vec<T> {
    fn to_rust(je: &mut JNIEnv<'a>, val: JValueOwned<'a>) -> Result<Self, jni::errors::Error> {
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
    fn into_temporary(&self, je: &mut JNIEnv<'a>) -> Result<Self::T, jni::errors::Error>;
    // tmp is borrowed, so that the value doesn't get dropped before the temporary is used.
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's>;
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
    $( impl<'a> ConvertRustToJValue<'a> for $t
    {
        type T=$t;
        fn into_temporary(&self, _je: & mut JNIEnv<'a>) ->Result<$t, jni::errors::Error> { Ok(*self) }
        fn temporary_into_jvalue<'s>(tmp: &'s $t) -> JValue<'a,'s>
        {
        (*tmp).into()
        }
    }
    impl<'a> ConvertRustToJValue<'a> for &$t
        {
            type T=$t;
            fn into_temporary(&self, _je: & mut JNIEnv<'a>) ->Result<$t, jni::errors::Error> { Ok(**self) }
            fn temporary_into_jvalue<'s>(tmp: &'s $t) -> JValue<'a,'s>
            {
            (*tmp).into()
            }
        }
        ) *
    }
}

impl_convert_rust_to_jvalue! { i8, i16, i32, i64, f32, f64 }

impl<'a> ConvertRustToJValue<'a> for char {
    type T = char;
    fn into_temporary(&self, _je: &mut JNIEnv<'a>) -> Result<char, jni::errors::Error> {
        Ok(*self)
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::Char((*tmp) as u16)
    }
}

impl<'a> ConvertRustToJValue<'a> for bool {
    type T = bool;
    fn into_temporary(&self, _je: &mut JNIEnv<'a>) -> Result<bool, jni::errors::Error> {
        Ok(*self)
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::Bool((*tmp) as u8)
    }
}

/// I would like to get rid of these, but I haven't figured out exactly how to
/// generically delegate from something to a &'c[T] without rust becoming confused
macro_rules! impl_convert_rust_vec_to_jvalue {
( $($t:ty ), *) => {
$(
 impl<'a> ConvertRustToJValue<'a> for Vec<$t> {
     type T = AutoLocal<'a, JObject<'a>>;
     fn into_temporary(&self, je: & mut JNIEnv<'a>) -> Result<Self::T, jni::errors::Error> {
         <&[$t] as ConvertRustToJValue>::into_temporary(&self.as_slice(), je) // delegate to the slice
     }
     fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
         JValue::from(tmp)
     }
 }
 )*
};
}

impl_convert_rust_vec_to_jvalue! {bool, char, u8, i8, i16, i32, i64, f32, f64 }

impl<'a> ConvertRustToJValue<'a> for &[bool] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = copy_bool_array_to_jbooleanarray(je, self)?;
        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn wrap_jobject<'a>(rval: jobject) -> JObject<'a> {
    unsafe { JObject::from_raw(rval) }
}

impl<'a> ConvertRustToJValue<'a> for &[char] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = copy_char_array_to_jchararray(je, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for &[i8] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let shenanigans = unsafe { &*((*self) as *const [i8] as *const [u8]) };
        let rval = je.byte_array_from_slice(shenanigans)?;
        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for &[u8] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.byte_array_from_slice(self)?;
        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for &str {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_string(self)?;
        Ok(JNIEnv::auto_local(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for String {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_string(self)?;
        Ok(JNIEnv::auto_local(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for &String {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_string(self)?;
        Ok(JNIEnv::auto_local(je, JObject::from(rval)))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for &[i32] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_int_array(self.len() as jsize)?;
        je.set_int_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
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

impl<'a> ConvertRustToJValue<'a> for &[i16] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_short_array(self.len() as jsize)?;
        je.set_short_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for &[i64] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_long_array(self.len() as jsize)?;
        je.set_long_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for &[f32] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_float_array(self.len() as jsize)?;
        je.set_float_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a> ConvertRustToJValue<'a> for &[f64] {
    type T = AutoLocal<'a, JObject<'a>>;
    fn into_temporary(
        &self,
        je: &mut JNIEnv<'a>,
    ) -> Result<AutoLocal<'a, JObject<'a>>, jni::errors::Error> {
        let rval = je.new_double_array(self.len() as jsize)?;
        je.set_double_array_region(&rval, 0, self)?;

        Ok(JNIEnv::auto_local(je, rval.into()))
    }
    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
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

impl<'a, S> ConvertRustToJValue<'a> for Vec<S>
where
    S: ConvertRustToJValue<'a> + JavaClassNameFor + JValueNonScalar,
{
    type T = AutoLocal<'a, JObject<'a>>;

    fn into_temporary(&self, je: &mut JNIEnv<'a>) -> Result<Self::T, Error> {
        let cls = je.find_class(S::java_class_name())?;
        let rval = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        for (i, val) in self.iter().enumerate() {
            let tmp: <S as ConvertRustToJValue<'a>>::T =
                <S as ConvertRustToJValue<'a>>::into_temporary(val, je)?;
            let object = <S as ConvertRustToJValue<'a>>::temporary_into_jvalue(&tmp).l()?;
            je.set_object_array_element(&rval, i as i32, object)?;
        }
        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

impl<'a, S> ConvertRustToJValue<'a> for &'a Vec<S>
where
    &'a [S]: ConvertRustToJValue<'a>,
    Self: 'a,
{
    type T = <&'a [S] as ConvertRustToJValue<'a>>::T;

    fn into_temporary(&self, je: &mut JNIEnv<'a>) -> Result<Self::T, Error> {
        <&'a [S] as ConvertRustToJValue<'a>>::into_temporary(&self.as_slice(), je)
    }

    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        <&'a [S] as ConvertRustToJValue<'a>>::temporary_into_jvalue(tmp)
    }
}

impl<'a, S> ConvertRustToJValue<'a> for &[S]
where
    S: ConvertRustToJValue<'a>,
    S: JavaClassNameFor + JValueNonScalar, // I need JValueNonScalar to void conflicting with &[i8] and friends
                                           // Self: 'c,
{
    type T = AutoLocal<'a, JObject<'a>>;

    fn into_temporary(&self, je: &mut JNIEnv<'a>) -> Result<Self::T, Error> {
        let cls = je.find_class(S::java_class_name())?;
        let rval = je.new_object_array(self.len() as i32, cls, JObject::null())?;

        for (i, val) in self.iter().enumerate() {
            let x: &S = val;
            let tmp: <S as ConvertRustToJValue>::T =
                <S as ConvertRustToJValue>::into_temporary(x, je)?;
            let t2 = <S as ConvertRustToJValue>::temporary_into_jvalue(&tmp);
            let t3 = t2.l()?;
            je.set_object_array_element(&rval, i as i32, t3)?;
        }
        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        JValue::from(tmp)
    }
}

/// I need this because of how `<&[&[i8]] as ConvertRustToJValue>` depends on <&&[i8] as ConvertRustToJValue>
impl<'a, S> ConvertRustToJValue<'a> for &&'a [S]
where
    &'a [S]: ConvertRustToJValue<'a>,
{
    type T = <&'a [S] as ConvertRustToJValue<'a>>::T;

    fn into_temporary(&self, je: &mut JNIEnv<'a>) -> Result<Self::T, Error> {
        <&'a [S] as ConvertRustToJValue<'a>>::into_temporary(*self, je)
    }

    fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> JValue<'a, 's> {
        <&'a [S] as ConvertRustToJValue<'a>>::temporary_into_jvalue(tmp)
    }
}

/// I'm not entirely sure why the impl above doesn't match &[&str],
/// and I'm in too much of a hurry to think it through.
impl<'a> ConvertRustToJValue<'a> for &[&str] {
    type T = AutoLocal<'a, JObject<'a>>;

    // fn into_temporary(&self, je: &mut JNIEnv<'a>) -> Result<Self::T, Error> {
    fn into_temporary(&self, je: &mut JNIEnv<'a>) -> Result<Self::T, Error> {
        let cls = je.find_class(<&str as JavaClassNameFor>::java_class_name())?;
        let rval = je.new_object_array(self.len() as i32, cls, JObject::null())?;
        for (i, val) in self.iter().enumerate() {
            let tmp: <&str as ConvertRustToJValue>::T =
                <&str as ConvertRustToJValue>::into_temporary(val, je)?;
            let t2 = <&str as ConvertRustToJValue>::temporary_into_jvalue(&tmp);
            je.set_object_array_element(&rval, i as i32, t2.l()?)?;
        }
        Ok(JNIEnv::auto_local(je, rval.into()))
    }

    fn temporary_into_jvalue<'c>(tmp: &'c Self::T) -> JValue<'a, 'c> {
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
        pub struct $ty<'a> {
            #[allow(dead_code)]
            java_this: jni::objects::AutoLocal<'a, jni::objects::JObject<'a>>,
            #[allow(dead_code)]
            jni_env: $crate::atrocious_kludges::KludgeJNIEnv<'a>, //XXX This is probably bad, but I haven't found a way round it yet
        }

        impl<'a> $ty<'a> {
            pub fn null(jni_env: &jni::JNIEnv<'a>) -> $ty<'a> {
                let java_this = jni_env.auto_local(jni::objects::JObject::null());
                let jni_env = $crate::kludge_mut_jnienv!(jni_env); //XXX This is probably bad, but I haven't found a way round it yet
                $ty {
                    java_this: java_this,
                    jni_env,
                }
            }
        }

        impl<'a> $crate::JValueNonScalar for $ty<'a> {}

        impl<'a> jni_boilerplate_helper::JavaClassNameFor for $ty<'a> {
            fn java_class_name() -> &'static str {
                $java_class_slash
            }
        }

        impl<'a, 'b> $crate::JavaConstructible<'a, 'b> for $ty<'a> {
            fn wrap_jobject(
                jni_env: &'b jni::JNIEnv<'a>,
                java_this: jni::objects::AutoLocal<'a, jni::objects::JObject<'a>>,
            ) -> Self {
                let jni_env = $crate::kludge_mut_jnienv!(jni_env); //XXX This is probably bad, but I haven't found a way round it yet
                $ty { java_this, jni_env }
            }
        }

        impl<'a> $crate::JavaSignatureFor for $ty<'a> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a> $crate::JavaSignatureFor for &$ty<'a> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a: 'b, 'b> $crate::ConvertRustToJValue<'a> for &$ty<'a> {
            type T = jni::objects::AutoLocal<'a, jni::objects::JObject<'a>>;
            fn into_temporary(
                &self,
                je: &mut jni::JNIEnv<'a>,
            ) -> Result<Self::T, jni::errors::Error> {
                Ok(je.auto_local(je.new_local_ref(&self.java_this)?))
            }

            fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> jni::objects::JValue<'a, 's> {
                jni::objects::JValue::from(tmp)
            }
        }

        impl<'a: 'b, 'b> $crate::ConvertRustToJValue<'a> for $ty<'a> {
            type T = jni::objects::AutoLocal<'a, jni::objects::JObject<'a>>;
            fn into_temporary(
                &self,
                je: &mut jni::JNIEnv<'a>,
            ) -> Result<Self::T, jni::errors::Error> {
                Ok(je.auto_local(je.new_local_ref(&self.java_this)?))
            }

            fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> jni::objects::JValue<'a, 's> {
                jni::objects::JValue::from(tmp)
            }
        }

        impl<'a: 'b, 'b> $crate::ConvertJValueToRust<'a> for $ty<'a> {
            fn to_rust(
                jni_env: &mut jni::JNIEnv<'a>,
                val: jni::objects::JValueOwned<'a>,
            ) -> Result<Self, jni::errors::Error> {
                let java_this = jni_env.auto_local(val.l()?);
                let jni_env = $crate::kludge_mut_jnienv!(jni_env); //XXX This is probably bad, but I haven't found a way round it yet
                Ok($ty {
                    java_this: java_this,
                    jni_env,
                })
            }
        }
    };
}

#[macro_export]
macro_rules! jni_wrapper_cliche_impl_T {
    ($ty:ident, $java_class_slash:literal) => {
        pub struct $ty<'a, T: $crate::ConvertJValueToRust<'a>> {
            #[allow(dead_code)]
            java_this: jni::objects::AutoLocal<'a, jni::objects::JObject<'a>>,
            #[allow(dead_code)]
            jni_env: std::cell::RefCell<jni::JNIEnv<'a>>, //XXX This is probably bad, but I haven't found a way round it yet
            phantom: std::marker::PhantomData<T>,
        }

        impl<'a, T: $crate::ConvertJValueToRust<'a>> $ty<'a, T> {
            pub fn null(jni_env: &jni::JNIEnv<'a>) -> $ty<'a, T> {
                let java_this = jni_env.auto_local(jni::objects::JObject::null());
                let jni_env = $crate::kludge_mut_jnienv!(jni_env); //XXX This is probably bad, but I haven't found a way round it yet
                $ty {
                    java_this: java_this,
                    jni_env,
                    phantom: std::marker::PhantomData,
                }
            }
        }

        impl<'a, T: $crate::ConvertJValueToRust<'a>> $crate::JValueNonScalar for $ty<'a, T> {}

        impl<'a, T: $crate::ConvertJValueToRust<'a>> $crate::JavaClassNameFor for $ty<'a, T> {
            fn java_class_name() -> &'static str {
                $java_class_slash
            }
        }

        impl<'a, 'b, T: $crate::ConvertJValueToRust<'a>> $crate::JavaConstructible<'a, 'b>
            for $ty<'a, T>
        {
            fn wrap_jobject(
                jni_env: &jni::JNIEnv<'a>,
                java_this: jni::objects::AutoLocal<'a, jni::objects::JObject<'a>>,
            ) -> Self {
                let jni_env = $crate::kludge_mut_jnienv!(jni_env); //XXX This is probably bad, but I haven't found a way round it yet
                $ty {
                    java_this,
                    jni_env,
                    phantom: std::marker::PhantomData,
                }
            }
        }

        impl<'a, T: $crate::ConvertJValueToRust<'a>> $crate::JavaSignatureFor for $ty<'a, T> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a, T: $crate::ConvertJValueToRust<'a>> $crate::JavaSignatureFor for &$ty<'a, T> {
            fn signature_for() -> String {
                String::from(concat!("L", $java_class_slash, ";"))
            }
        }

        impl<'a, T: $crate::ConvertJValueToRust<'a>> $crate::ConvertRustToJValue<'a>
            for &$ty<'a, T>
        {
            type T = jni::objects::AutoLocal<'a, jni::objects::JObject<'a>>;
            fn into_temporary(
                &self,
                je: &mut jni::JNIEnv<'a>,
            ) -> Result<Self::T, jni::errors::Error> {
                Ok(je.auto_local(je.new_local_ref(&self.java_this)?))
            }

            fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> jni::objects::JValue<'a, 's> {
                jni::objects::JValue::from(tmp)
            }
        }

        impl<'a, T: $crate::ConvertJValueToRust<'a>> $crate::ConvertRustToJValue<'a>
            for $ty<'a, T>
        {
            type T = jni::objects::AutoLocal<'a, jni::objects::JObject<'a>>;
            fn into_temporary(
                &self,
                je: &mut jni::JNIEnv<'a>,
            ) -> Result<Self::T, jni::errors::Error> {
                Ok(je.auto_local(je.new_local_ref(&self.java_this)?))
            }

            fn temporary_into_jvalue<'s>(tmp: &'s Self::T) -> jni::objects::JValue<'a, 's> {
                jni::objects::JValue::from(tmp)
            }
        }

        impl<'a, T: $crate::ConvertJValueToRust<'a>> $crate::ConvertJValueToRust<'a>
            for $ty<'a, T>
        {
            fn to_rust(
                jni_env: &mut jni::JNIEnv<'a>,
                val: jni::objects::JValueOwned<'a>,
            ) -> Result<Self, jni::errors::Error> {
                let java_this = jni_env.auto_local(val.l()?);
                let jni_env = $crate::kludge_mut_jnienv!(jni_env); //XXX This is probably bad, but I haven't found a way round it yet
                Ok($ty {
                    java_this: java_this,
                    jni_env,
                    phantom: std::marker::PhantomData,
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
