use jni::objects::{JObject, JValue};
use jni::sys::{jintArray, jsize};
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
        Ok( ArrayCopyBackInt { array, src, env } )
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
