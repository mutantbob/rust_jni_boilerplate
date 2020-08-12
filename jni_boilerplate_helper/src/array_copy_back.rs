use jni::objects::{JObject, JValue};
use jni::sys::{jbyteArray, jintArray, jshortArray, jsize};
use jni::JNIEnv;

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
        println!("ArrayCopyBackByte::new()");
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
        println!("ArrayCopyBackByte drop()");
        self.env
            .get_byte_array_region(self.array, 0, self.src)
            .expect("how did the get_int_array_region fail?");
        self.env
            .delete_local_ref(JObject::from(self.array))
            .expect("how did delete_local_ref() fail?");
    }
}
