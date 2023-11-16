use std::ffi::CStr;

use quickjs_ng_sys::*;

fn main() {
    let rt = unsafe { JS_NewRuntime() };
    if rt.is_null() {
        panic!("JS_NewRuntime failed");
    }

    let ctx = unsafe { JS_NewContext(rt) };
    if ctx.is_null() {
        panic!("JS_NewContext failed");
    }

    const INPUT: &[u8] = b"
        throw new Error('test');
    ";
    let result = unsafe {
        JS_Eval(
            ctx,
            INPUT.as_ptr() as *const i8,
            INPUT.len(),
            b"".as_ptr() as *const i8,
            JS_EVAL_TYPE_MODULE as i32,
        )
    };

    assert!(unsafe { JS_IsException(result) } != 0);
    let exception_str = unsafe { CStr::from_ptr(JS_ToCString(ctx, JS_GetException(ctx))) };
    assert_eq!(exception_str.to_str().unwrap(), "Error: test");

    unsafe {
        JS_FreeValue(ctx, result);
        JS_RunGC(rt);
    }
}
