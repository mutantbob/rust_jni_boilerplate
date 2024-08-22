/// This is just where I stash some helper functions for calling important stuff in the java runtime
use crate::wrap_jobject;
use jni::objects::{AutoLocal, JObject, JValue, JValueOwned};
use jni::sys::jobject;
use jni::JNIEnv;

/// class_object is an instance of java.lang.Class
pub fn class_is_array(je: &mut JNIEnv, class_object: &JObject) -> Result<bool, jni::errors::Error> {
    let rval = je.call_method(class_object, "isArray", "()Z", &[])?;
    rval.z()
}

pub fn jni_workaround_jvalue<'a>(val: jobject) -> JValueOwned<'a> {
    JValueOwned::Object(wrap_jobject(val))
}

//

pub struct Throwable<'a> {
    #[allow(dead_code)]
    java_this: jni::objects::AutoLocal<'a, JObject<'a>>,
    // #[allow(dead_code)]
    // jni_env: &'b jni::JNIEnv<'a>,
}

impl<'a> Throwable<'a> {
    pub fn null(jni_env: &jni::JNIEnv<'a>) -> Throwable<'a> {
        Throwable {
            java_this: jni_env.auto_local(jni::objects::JObject::null()),
            // jni_env,
        }
    }
}

impl<'a> crate::JValueNonScalar for Throwable<'a> {}

impl<'a> crate::JavaClassNameFor for Throwable<'a> {
    fn java_class_name() -> &'static str {
        "java/lang/Throwable"
    }
}

impl<'a, 'b> crate::JavaConstructible<'a, 'b> for Throwable<'a> {
    fn wrap_jobject(
        jni_env: &'b jni::JNIEnv<'a>,
        java_this: jni::objects::AutoLocal<'a, JObject<'a>>,
    ) -> Self {
        Throwable {
            java_this, /*jni_env*/
        }
    }
}

impl<'a> crate::JavaSignatureFor for Throwable<'a> {
    fn signature_for() -> String {
        String::from(concat!("L", "java/lang/Throwable", ";"))
    }
}

impl<'a2: 'b2, 'b2> crate::ConvertRustToJValue for Throwable<'a2> {
    type T<'a: 'b, 'b> = AutoLocal<'a, JObject<'a>>;
    fn into_temporary<'a1, 'b1>(
        &self,
        je: &'b1 mut JNIEnv<'a1>,
    ) -> Result<Self::T<'a1, 'b1>, jni::errors::Error> {
        let x = je.new_local_ref(&self.java_this)?;
        Ok(je.auto_local(x))
    }

    fn temporary_into_jvalue<'a1: 'b1, 'b1, 's>(tmp: &'s Self::T<'a1, 'b1>) -> JValue<'a1, 's> {
        JValue::from(tmp)
    }
}

impl<'c> crate::ConvertJValueToRust for Throwable<'c> {
    fn to_rust<'a, 'b>(
        jni_env: &'b mut jni::JNIEnv<'a>,
        val: jni::objects::JValueOwned<'a>,
    ) -> Result<Self, jni::errors::Error> {
        Ok(Throwable {
            java_this: jni_env.auto_local(val.l()?.into()),
            // jni_env,
        })
    }
}

impl Throwable<'_> {
    #[allow(non_snake_case)]
    pub fn printStackTrace(&self, je: &mut JNIEnv) -> Result<(), jni::errors::Error> {
        use crate::{ConvertJValueToRust, JavaSignatureFor};
        #[cfg(debug_assertions)]
        crate::panic_if_bad_sigs(&[<() as JavaSignatureFor>::signature_for()]);
        let sig = String::from("(") + ")" + &<() as JavaSignatureFor>::signature_for();
        let results = je.call_method(&self.java_this, "printStackTrace", sig, &[])?;
        <() as ConvertJValueToRust>::to_rust(je, results)
    }
}
