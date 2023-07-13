/// This is just where I stash some helper functions for calling important stuff in the java runtime
use crate::jni;
use jni::objects::JObject;
use jni::JNIEnv;

/// class_object is an instance of java.lang.Class
pub fn class_is_array(je: &JNIEnv, class_object: &JObject) -> Result<bool, jni::errors::Error> {
    let rval = je.call_method(*class_object, "isArray", "()Z", &[])?;
    rval.z()
}
