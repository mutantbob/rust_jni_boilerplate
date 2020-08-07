# rust_jni_boilerplate
A set of procedural macros for generating rust boilerplate for invoking java functions

As of 2020-Aug this project is still very much in flux.

usage examples:

```
struct ArrayToys
{

}

impl ArrayToys
{
    jni_static_method! { sum(&[i32]) -> i32 }
    jni_static_method! { sum_short=sum(&[i16]) -> i32 }
    jni_static_method! { incr_all(&mut [i32]) }
}

impl JavaClassNameFor for ArrayToys
{
    fn java_class_name() -> &'static str {
        "com/purplefrog/rust_callables/ArrayToys"
    }
}

struct Widget<'a> {
    jni_env: &'a AttachGuard<'a>,
    java_this: AutoLocal<'a,'a>,
}

impl<'a> JavaConstructible<'a> for Widget<'a>
{
    fn wrap_jobject(jni_env:&'a AttachGuard<'a>, java_this: AutoLocal<'a,'a>) -> Widget<'a>
    {
        Widget {
            jni_env,
            java_this,
        }
    }
}

impl<'a> Widget<'a> {

    // define a rust function named new
    jni_constructor! { com.purplefrog.rust_callables.Widget () }
    jni_constructor! { new_one=com.purplefrog.rust_callables.Widget (&str) }

    jni_instance_method! { count () -> i32 }
    jni_instance_method! { sumLen () -> i32 }

    jni_instance_method! { add(&str) }

    jni_instance_method! { echo_str=echo(&str)->String }
    jni_instance_method! { echo_char=echo(char)->char }
    jni_instance_method! { echo_byte=echo(i8)->i8 }
    jni_instance_method! { echo_short=echo(i16)->i16 }
    jni_instance_method! { echo_int=echo(i32)->i32 }
    jni_instance_method! { echo_long=echo(i64)->i64 }
}


```
