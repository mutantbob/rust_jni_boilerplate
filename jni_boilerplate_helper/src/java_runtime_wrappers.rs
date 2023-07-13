/// This is just where I stash some helper functions for calling important stuff in the java runtime
use crate::{jni, wrap_jobject};
use jni::objects::{JObject, JValue};
use jni::sys::jobject;
use jni::JNIEnv;

/// class_object is an instance of java.lang.Class
pub fn class_is_array(je: &JNIEnv, class_object: &JObject) -> Result<bool, jni::errors::Error> {
    let rval = je.call_method(*class_object, "isArray", "()Z", &[])?;
    rval.z()
}

#[cfg(not(feature = "jni_0_20"))]
pub fn jni_workaround_jvalue<'a>(val: jobject) -> JValue<'a> {
    JValue::from(val)
}

#[cfg(feature = "jni_0_20")]
pub fn jni_workaround_jvalue<'a>(val: jobject) -> JValue<'a> {
    JValue::Object(wrap_jobject(val))
}

//

pub struct Throwable<'a: 'b, 'b> {
    #[allow(dead_code)]
    java_this: jni::objects::AutoLocal<'a, 'b>,
    #[allow(dead_code)]
    jni_env: &'b jni::JNIEnv<'a>,
}

impl<'a, 'b> Throwable<'a, 'b> {
    pub fn null(jni_env: &'b jni::JNIEnv<'a>) -> Throwable<'a, 'b> {
        Throwable {
            java_this: jni::objects::AutoLocal::new(jni_env, jni::objects::JObject::null()),
            jni_env,
        }
    }
}

impl<'a, 'b> crate::JValueNonScalar for Throwable<'a, 'b> {}

impl<'a, 'b> crate::JavaClassNameFor for Throwable<'a, 'b> {
    fn java_class_name() -> &'static str {
        "java/lang/Throwable"
    }
}

impl<'a, 'b> crate::JavaConstructible<'a, 'b> for Throwable<'a, 'b> {
    fn wrap_jobject(
        jni_env: &'b jni::JNIEnv<'a>,
        java_this: jni::objects::AutoLocal<'a, 'b>,
    ) -> Self {
        Throwable { java_this, jni_env }
    }
}

impl<'a, 'b> crate::JavaSignatureFor for Throwable<'a, 'b> {
    fn signature_for() -> String {
        String::from(concat!("L", "java/lang/Throwable", ";"))
    }
}

impl<'a: 'b, 'b> crate::ConvertRustToJValue<'a, 'b> for Throwable<'a, 'b> {
    type T = jni::sys::jobject;
    fn into_temporary(
        &self,
        _je: &'b jni::JNIEnv<'a>,
    ) -> Result<jni::sys::jobject, jni::errors::Error> {
        Ok(*self.java_this.as_obj())
    }

    fn temporary_into_jvalue(tmp: &Self::T) -> jni::objects::JValue<'a> {
        JValue::from(wrap_jobject(*tmp))
    }
}

impl<'a: 'b, 'b> crate::ConvertJValueToRust<'a, 'b> for Throwable<'a, 'b> {
    fn to_rust(
        jni_env: &'b jni::JNIEnv<'a>,
        val: jni::objects::JValue<'a>,
    ) -> Result<Self, jni::errors::Error> {
        Ok(Throwable {
            java_this: jni::objects::AutoLocal::new(jni_env, val.l()?),
            jni_env,
        })
    }
}

impl Throwable<'_, '_> {
    #[allow(non_snake_case)]
    pub fn printStackTrace(&self) -> Result<(), jni::errors::Error> {
        use crate::{ConvertJValueToRust, JavaSignatureFor};
        #[cfg(debug_assertions)]
        crate::panic_if_bad_sigs(&[<() as JavaSignatureFor>::signature_for()]);
        let sig = String::from("(") + ")" + &<() as JavaSignatureFor>::signature_for();
        let results =
            self.jni_env
                .call_method(self.java_this.as_obj(), "printStackTrace", sig, &[])?;
        <() as ConvertJValueToRust>::to_rust(self.jni_env, results)
    }
}
