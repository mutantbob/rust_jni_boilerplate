#[macro_export]
macro_rules! kludge_mut_jnienv {
    //XXX This is probably bad, but I haven't found a way round it yet
    ($je:expr) => {
        std::cell::RefCell::new(unsafe { $je.unsafe_clone() })
    };
}

pub type KludgeJNIEnv<'a> = std::cell::RefCell<jni::JNIEnv<'a>>;
