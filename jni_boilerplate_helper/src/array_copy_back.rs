use crate::u32_to_char;
use jni::objects::{JObject, JValue};
use jni::sys::{
    jboolean, jbooleanArray, jbyteArray, jchar, jcharArray, jdoubleArray, jfloatArray, jintArray,
    jlongArray, jshortArray, jsize,
};
use jni::JNIEnv;
use log::debug;

//

pub struct ArrayCopyBackBool<'a, 'b, 'c> {
    array: jbyteArray,
    src: &'c mut [bool],
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b, 'c> ArrayCopyBackBool<'a, 'b, 'c> {
    pub fn new(
        src: &'c mut [bool],
        env: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackBool<'a, 'b, 'c>, jni::errors::Error> {
        //println!("ArrayCopyBackBool::new()");
        let array = copy_bool_array_to_jbooleanarray(env, src)?;
        Ok(ArrayCopyBackBool { array, src, env })
    }

    pub fn as_jvalue(&self) -> JValue<'a> {
        JValue::from(self.array)
    }
}

impl<'a, 'b, 'c> Drop for ArrayCopyBackBool<'a, 'b, 'c> {
    fn drop(&mut self) {
        //println!("ArrayCopyBackBool drop()");
        move_jbooleanarray_to_bool_array(self.env, self.array, self.src);
    }
}

//

pub fn copy_bool_array_to_jbooleanarray(
    env: &JNIEnv,
    src: &[bool],
) -> Result<jbooleanArray, jni::errors::Error> {
    let array = env.new_boolean_array(src.len() as jsize)?;
    let tmp: Vec<jboolean> = src.iter().map(|x| if *x { 1 } else { 0 }).collect();
    env.set_boolean_array_region(array, 0, &tmp)?;
    Ok(array)
}

pub fn move_jbooleanarray_to_bool_array(je: &JNIEnv, src: jbooleanArray, dst: &mut [bool]) {
    let mut tmp: Vec<jboolean> = vec![0; dst.len()];
    je.get_boolean_array_region(src, 0, &mut tmp)
        .expect("how did the get_boolean_array_region fail?");
    for (i, x) in tmp.iter().enumerate() {
        dst[i] = 0 != *x;
    }
    je.delete_local_ref(JObject::from(src))
        .expect("how did delete_local_ref() fail?");
}

//

pub struct ArrayCopyBackChar<'a, 'b, 'c> {
    array: jbyteArray,
    src: &'c mut [char],
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b, 'c> ArrayCopyBackChar<'a, 'b, 'c> {
    pub fn new(
        src: &'c mut [char],
        env: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackChar<'a, 'b, 'c>, jni::errors::Error> {
        //println!("ArrayCopyBackChar::new()");
        let array = copy_char_array_to_jchararray(env, src)?;
        Ok(ArrayCopyBackChar { array, src, env })
    }

    pub fn as_jvalue(&self) -> JValue<'a> {
        JValue::from(self.array)
    }
}

impl<'a, 'b, 'c> Drop for ArrayCopyBackChar<'a, 'b, 'c> {
    fn drop(&mut self) {
        //println!("ArrayCopyBackChar drop()");
        if let Err(e) = move_jchararray_to_char_array(self.env, self.array, self.src) {
            debug!("how did move_jchararray_to_char_array fail? {:?}", e);
        }
    }
}

//

//

pub fn copy_char_array_to_jchararray(
    je: &JNIEnv,
    src: &[char],
) -> Result<jcharArray, jni::errors::Error> {
    let rval: jcharArray = je.new_char_array(src.len() as jsize)?;
    let other: Vec<jchar> = src.iter().map(|x| *x as jchar).collect();
    je.set_char_array_region(rval, 0, &other)?;
    Ok(rval)
}

pub fn move_jchararray_to_char_array(
    je: &JNIEnv,
    src: jcharArray,
    dst: &mut [char],
) -> Result<(), jni::errors::Error> {
    let mut tmp: Vec<jchar> = vec![0; dst.len()];
    je.get_char_array_region(src, 0, &mut tmp)?;

    for (i, x) in tmp.iter().enumerate() {
        dst[i] = u32_to_char(*x as u32)?;
    }
    je.delete_local_ref(JObject::from(src))
}

//

pub struct ArrayCopyBackInt<'a, 'b, 'c> {
    array: jintArray,
    src: &'c mut [i32],
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b, 'c> ArrayCopyBackInt<'a, 'b, 'c> {
    pub fn new(
        src: &'c mut [i32],
        env: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackInt<'a, 'b, 'c>, jni::errors::Error> {
        let array = env.new_int_array(src.len() as jsize)?;
        env.set_int_array_region(array, 0, src)?;
        Ok(ArrayCopyBackInt { array, src, env })
    }

    pub fn as_jvalue(&self) -> JValue<'a> {
        JValue::from(self.array)
    }
}

impl<'a, 'b, 'c> Drop for ArrayCopyBackInt<'a, 'b, 'c> {
    fn drop(&mut self) {
        self.env
            .get_int_array_region(self.array, 0, self.src)
            .expect("how did the get_int_array_region fail?");
        self.env
            .delete_local_ref(JObject::from(self.array))
            .expect("how did delete_local_ref() fail?");
    }
}

//

pub struct ArrayCopyBackShort<'a, 'b, 'c> {
    array: jshortArray,
    src: &'c mut [i16],
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b, 'c> ArrayCopyBackShort<'a, 'b, 'c> {
    pub fn new(
        src: &'c mut [i16],
        env: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackShort<'a, 'b, 'c>, jni::errors::Error> {
        let array = env.new_short_array(src.len() as jsize)?;
        env.set_short_array_region(array, 0, src)?;
        Ok(ArrayCopyBackShort { array, src, env })
    }

    pub fn as_jvalue(&self) -> JValue<'a> {
        JValue::from(self.array)
    }
}

impl<'a, 'b, 'c> Drop for ArrayCopyBackShort<'a, 'b, 'c> {
    fn drop(&mut self) {
        self.env
            .get_short_array_region(self.array, 0, self.src)
            .expect("how did the get_int_array_region fail?");
        self.env
            .delete_local_ref(JObject::from(self.array))
            .expect("how did delete_local_ref() fail?");
    }
}

//

pub struct ArrayCopyBackByte<'a, 'b, 'c> {
    array: jbyteArray,
    src: &'c mut [i8],
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b, 'c> ArrayCopyBackByte<'a, 'b, 'c> {
    pub fn new(
        src: &'c mut [i8],
        env: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackByte<'a, 'b, 'c>, jni::errors::Error> {
        //println!("ArrayCopyBackByte::new()");
        let array = env.new_byte_array(src.len() as jsize)?;
        env.set_byte_array_region(array, 0, src)?;
        Ok(ArrayCopyBackByte { array, src, env })
    }

    pub fn as_jvalue(&self) -> JValue<'a> {
        JValue::from(self.array)
    }
}

impl<'a, 'b, 'c> Drop for ArrayCopyBackByte<'a, 'b, 'c> {
    fn drop(&mut self) {
        //println!("ArrayCopyBackByte drop()");
        self.env
            .get_byte_array_region(self.array, 0, self.src)
            .expect("how did the get_int_array_region fail?");
        self.env
            .delete_local_ref(JObject::from(self.array))
            .expect("how did delete_local_ref() fail?");
    }
}

//

pub struct ArrayCopyBackLong<'a, 'b, 'c> {
    array: jlongArray,
    src: &'c mut [i64],
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b, 'c> ArrayCopyBackLong<'a, 'b, 'c> {
    pub fn new(
        src: &'c mut [i64],
        env: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackLong<'a, 'b, 'c>, jni::errors::Error> {
        //println!("ArrayCopyBackLong::new()");
        let array = env.new_long_array(src.len() as jsize)?;
        env.set_long_array_region(array, 0, src)?;
        Ok(ArrayCopyBackLong { array, src, env })
    }

    pub fn as_jvalue(&self) -> JValue<'a> {
        JValue::from(self.array)
    }
}

impl<'a, 'b, 'c> Drop for ArrayCopyBackLong<'a, 'b, 'c> {
    fn drop(&mut self) {
        //println!("ArrayCopyBackLong drop()");
        self.env
            .get_long_array_region(self.array, 0, self.src)
            .expect("how did the get_int_array_region fail?");
        self.env
            .delete_local_ref(JObject::from(self.array))
            .expect("how did delete_local_ref() fail?");
    }
}

//

pub struct ArrayCopyBackFloat<'a, 'b, 'c> {
    array: jfloatArray,
    src: &'c mut [f32],
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b, 'c> ArrayCopyBackFloat<'a, 'b, 'c> {
    pub fn new(
        src: &'c mut [f32],
        env: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackFloat<'a, 'b, 'c>, jni::errors::Error> {
        //println!("ArrayCopyBackFloat::new()");
        let array = env.new_float_array(src.len() as jsize)?;
        env.set_float_array_region(array, 0, src)?;
        Ok(ArrayCopyBackFloat { array, src, env })
    }

    pub fn as_jvalue(&self) -> JValue<'a> {
        JValue::from(self.array)
    }
}

impl<'a, 'b, 'c> Drop for ArrayCopyBackFloat<'a, 'b, 'c> {
    fn drop(&mut self) {
        //println!("ArrayCopyBackFloat drop()");
        self.env
            .get_float_array_region(self.array, 0, self.src)
            .expect("how did the get_int_array_region fail?");
        self.env
            .delete_local_ref(JObject::from(self.array))
            .expect("how did delete_local_ref() fail?");
    }
}

//

pub struct ArrayCopyBackDouble<'a, 'b, 'c> {
    array: jdoubleArray,
    src: &'c mut [f64],
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b, 'c> ArrayCopyBackDouble<'a, 'b, 'c> {
    pub fn new(
        src: &'c mut [f64],
        env: &'b JNIEnv<'a>,
    ) -> Result<ArrayCopyBackDouble<'a, 'b, 'c>, jni::errors::Error> {
        //println!("ArrayCopyBackDouble::new()");
        let array = env.new_double_array(src.len() as jsize)?;
        env.set_double_array_region(array, 0, src)?;
        Ok(ArrayCopyBackDouble { array, src, env })
    }

    pub fn as_jvalue(&self) -> JValue<'a> {
        JValue::from(self.array)
    }
}

impl<'a, 'b, 'c> Drop for ArrayCopyBackDouble<'a, 'b, 'c> {
    fn drop(&mut self) {
        //println!("ArrayCopyBackDouble drop()");
        self.env
            .get_double_array_region(self.array, 0, self.src)
            .expect("how did the get_int_array_region fail?");
        self.env
            .delete_local_ref(JObject::from(self.array))
            .expect("how did delete_local_ref() fail?");
    }
}
