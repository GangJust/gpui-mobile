//! Shared raw JNI helpers for Android package implementations.
//!
//! Uses the same raw function-table pattern as `crate::android::jni::jni_fns`
//! to avoid pulling in the `jni` crate for simple calls.

#![allow(unsafe_code, non_snake_case, dead_code)]

use std::ffi::c_void;
use super::jni as app_jni;

// ── JNI function table types ─────────────────────────────────────────────────

pub type FindClassFn = unsafe extern "C" fn(*mut c_void, *const i8) -> *mut c_void;
pub type GetMethodIDFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *const i8, *const i8) -> *mut c_void;
pub type GetStaticMethodIDFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *const i8, *const i8) -> *mut c_void;
pub type GetFieldIDFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *const i8, *const i8) -> *mut c_void;
pub type GetStaticFieldIDFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *const i8, *const i8) -> *mut c_void;
pub type CallObjectMethodAFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const i64) -> *mut c_void;
pub type CallStaticObjectMethodAFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const i64) -> *mut c_void;
pub type CallBooleanMethodAFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const i64) -> u8;
pub type CallIntMethodAFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const i64) -> i32;
pub type CallLongMethodAFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const i64) -> i64;
pub type CallVoidMethodAFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const i64);
pub type NewObjectAFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const i64) -> *mut c_void;
pub type GetStaticObjectFieldFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
pub type GetIntFieldFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> i32;
pub type GetObjectFieldFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
pub type NewStringUTFFn = unsafe extern "C" fn(*mut c_void, *const i8) -> *mut c_void;
pub type GetStringUTFCharsFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut u8) -> *const i8;
pub type ReleaseStringUTFCharsFn =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *const i8);
pub type DeleteLocalRefFn = unsafe extern "C" fn(*mut c_void, *mut c_void);
pub type ExceptionCheckFn = unsafe extern "C" fn(*mut c_void) -> u8;
pub type ExceptionClearFn = unsafe extern "C" fn(*mut c_void);

// ── JNI function table offsets ───────────────────────────────────────────────

// From JNINativeInterface (same offsets as used in jni.rs)
const IDX_FIND_CLASS: usize = 6;
const IDX_EXCEPTION_CLEAR: usize = 17;
const IDX_DELETE_LOCAL_REF: usize = 23;
const IDX_NEW_OBJECT_A: usize = 30;
const IDX_GET_METHOD_ID: usize = 33;
const IDX_CALL_OBJECT_METHOD_A: usize = 36;
const IDX_CALL_BOOLEAN_METHOD_A: usize = 39;
const IDX_CALL_INT_METHOD_A: usize = 49;
const IDX_CALL_LONG_METHOD_A: usize = 55;
const IDX_CALL_VOID_METHOD_A: usize = 61;
const IDX_GET_FIELD_ID: usize = 94;
const IDX_GET_OBJECT_FIELD: usize = 95;
const IDX_GET_INT_FIELD: usize = 100;
const IDX_CALL_STATIC_OBJECT_METHOD_A: usize = 116;
const IDX_GET_STATIC_METHOD_ID: usize = 113;
const IDX_GET_STATIC_FIELD_ID: usize = 144;
const IDX_GET_STATIC_OBJECT_FIELD: usize = 145;
const IDX_NEW_STRING_UTF: usize = 167;
const IDX_GET_STRING_UTF_CHARS: usize = 169;
const IDX_RELEASE_STRING_UTF_CHARS: usize = 170;
const IDX_EXCEPTION_CHECK: usize = 228;

/// A loaded JNI function table with pre-resolved function pointers.
///
/// Create via [`JniEnv::obtain()`]. All methods are unsafe because they
/// call into the JVM through raw function pointers.
pub struct JniEnv {
    pub env: *mut c_void,
    fn_table: *const *const c_void,
}

impl JniEnv {
    /// Attach the current thread and obtain a JNI environment.
    pub fn obtain() -> Result<Self, String> {
        let vm = app_jni::java_vm();
        if vm.is_null() {
            return Err("JavaVM not available".into());
        }
        let env = unsafe { app_jni::jni_fns::get_env_from_vm(vm) };
        if env.is_null() {
            return Err("Failed to obtain JNIEnv".into());
        }
        let fn_table = unsafe { *(env as *const *const *const c_void) };
        Ok(Self { env, fn_table })
    }

    /// Get the raw Activity jobject pointer.
    pub fn activity(&self) -> *mut c_void {
        app_jni::activity_as_ptr()
    }

    unsafe fn jni_fn<T>(&self, idx: usize) -> T {
        std::mem::transmute_copy::<*const c_void, T>(&*self.fn_table.add(idx))
    }

    pub unsafe fn find_class(&self, name: &[u8]) -> *mut c_void {
        let f: FindClassFn = self.jni_fn(IDX_FIND_CLASS);
        let cls = f(self.env, name.as_ptr() as *const i8);
        if cls.is_null() {
            self.clear_exception();
        }
        cls
    }

    pub unsafe fn get_method_id(
        &self, cls: *mut c_void, name: &[u8], sig: &[u8],
    ) -> *mut c_void {
        let f: GetMethodIDFn = self.jni_fn(IDX_GET_METHOD_ID);
        let id = f(self.env, cls, name.as_ptr() as *const i8, sig.as_ptr() as *const i8);
        if id.is_null() { self.clear_exception(); }
        id
    }

    pub unsafe fn get_static_method_id(
        &self, cls: *mut c_void, name: &[u8], sig: &[u8],
    ) -> *mut c_void {
        let f: GetStaticMethodIDFn = self.jni_fn(IDX_GET_STATIC_METHOD_ID);
        let id = f(self.env, cls, name.as_ptr() as *const i8, sig.as_ptr() as *const i8);
        if id.is_null() { self.clear_exception(); }
        id
    }

    pub unsafe fn get_field_id(
        &self, cls: *mut c_void, name: &[u8], sig: &[u8],
    ) -> *mut c_void {
        let f: GetFieldIDFn = self.jni_fn(IDX_GET_FIELD_ID);
        let id = f(self.env, cls, name.as_ptr() as *const i8, sig.as_ptr() as *const i8);
        if id.is_null() { self.clear_exception(); }
        id
    }

    pub unsafe fn get_static_field_id(
        &self, cls: *mut c_void, name: &[u8], sig: &[u8],
    ) -> *mut c_void {
        let f: GetStaticFieldIDFn = self.jni_fn(IDX_GET_STATIC_FIELD_ID);
        let id = f(self.env, cls, name.as_ptr() as *const i8, sig.as_ptr() as *const i8);
        if id.is_null() { self.clear_exception(); }
        id
    }

    pub unsafe fn call_object_method(
        &self, obj: *mut c_void, method: *mut c_void, args: &[i64],
    ) -> *mut c_void {
        let f: CallObjectMethodAFn = self.jni_fn(IDX_CALL_OBJECT_METHOD_A);
        let result = f(self.env, obj, method, args.as_ptr());
        if self.exception_check() { self.clear_exception(); return std::ptr::null_mut(); }
        result
    }

    pub unsafe fn call_static_object_method(
        &self, cls: *mut c_void, method: *mut c_void, args: &[i64],
    ) -> *mut c_void {
        let f: CallStaticObjectMethodAFn = self.jni_fn(IDX_CALL_STATIC_OBJECT_METHOD_A);
        let result = f(self.env, cls, method, args.as_ptr());
        if self.exception_check() { self.clear_exception(); return std::ptr::null_mut(); }
        result
    }

    pub unsafe fn call_boolean_method(
        &self, obj: *mut c_void, method: *mut c_void, args: &[i64],
    ) -> bool {
        let f: CallBooleanMethodAFn = self.jni_fn(IDX_CALL_BOOLEAN_METHOD_A);
        let result = f(self.env, obj, method, args.as_ptr());
        if self.exception_check() { self.clear_exception(); return false; }
        result != 0
    }

    pub unsafe fn call_int_method(
        &self, obj: *mut c_void, method: *mut c_void, args: &[i64],
    ) -> i32 {
        let f: CallIntMethodAFn = self.jni_fn(IDX_CALL_INT_METHOD_A);
        let result = f(self.env, obj, method, args.as_ptr());
        if self.exception_check() { self.clear_exception(); return 0; }
        result
    }

    pub unsafe fn call_long_method(
        &self, obj: *mut c_void, method: *mut c_void, args: &[i64],
    ) -> i64 {
        let f: CallLongMethodAFn = self.jni_fn(IDX_CALL_LONG_METHOD_A);
        let result = f(self.env, obj, method, args.as_ptr());
        if self.exception_check() { self.clear_exception(); return 0; }
        result
    }

    pub unsafe fn call_void_method(
        &self, obj: *mut c_void, method: *mut c_void, args: &[i64],
    ) {
        let f: CallVoidMethodAFn = self.jni_fn(IDX_CALL_VOID_METHOD_A);
        f(self.env, obj, method, args.as_ptr());
        if self.exception_check() { self.clear_exception(); }
    }

    pub unsafe fn new_object(
        &self, cls: *mut c_void, ctor: *mut c_void, args: &[i64],
    ) -> *mut c_void {
        let f: NewObjectAFn = self.jni_fn(IDX_NEW_OBJECT_A);
        let result = f(self.env, cls, ctor, args.as_ptr());
        if self.exception_check() { self.clear_exception(); return std::ptr::null_mut(); }
        result
    }

    pub unsafe fn get_static_object_field(
        &self, cls: *mut c_void, field: *mut c_void,
    ) -> *mut c_void {
        let f: GetStaticObjectFieldFn = self.jni_fn(IDX_GET_STATIC_OBJECT_FIELD);
        f(self.env, cls, field)
    }

    pub unsafe fn get_int_field_value(
        &self, obj: *mut c_void, field: *mut c_void,
    ) -> i32 {
        let f: GetIntFieldFn = self.jni_fn(IDX_GET_INT_FIELD);
        f(self.env, obj, field)
    }

    pub unsafe fn get_object_field(
        &self, obj: *mut c_void, field: *mut c_void,
    ) -> *mut c_void {
        let f: GetObjectFieldFn = self.jni_fn(IDX_GET_OBJECT_FIELD);
        f(self.env, obj, field)
    }

    pub unsafe fn new_string_utf(&self, s: &str) -> *mut c_void {
        let mut buf = Vec::with_capacity(s.len() + 1);
        buf.extend_from_slice(s.as_bytes());
        buf.push(0);
        let f: NewStringUTFFn = self.jni_fn(IDX_NEW_STRING_UTF);
        let result = f(self.env, buf.as_ptr() as *const i8);
        if self.exception_check() { self.clear_exception(); return std::ptr::null_mut(); }
        result
    }

    /// Extract a Java String as a Rust String. Returns empty string on null.
    pub unsafe fn get_string(&self, jstr: *mut c_void) -> String {
        if jstr.is_null() {
            return String::new();
        }
        let get_chars: GetStringUTFCharsFn = self.jni_fn(IDX_GET_STRING_UTF_CHARS);
        let release_chars: ReleaseStringUTFCharsFn = self.jni_fn(IDX_RELEASE_STRING_UTF_CHARS);

        let chars = get_chars(self.env, jstr, std::ptr::null_mut());
        if chars.is_null() {
            return String::new();
        }
        let s = std::ffi::CStr::from_ptr(chars as *const std::ffi::c_char).to_string_lossy().into_owned();
        release_chars(self.env, jstr, chars);
        s
    }

    /// Read a static String field from a class.
    pub unsafe fn get_static_string_field(
        &self, cls: *mut c_void, name: &[u8],
    ) -> String {
        let field_id = self.get_static_field_id(cls, name, b"Ljava/lang/String;\0");
        if field_id.is_null() { return String::new(); }
        let jstr = self.get_static_object_field(cls, field_id);
        let s = self.get_string(jstr);
        self.delete_local_ref(jstr);
        s
    }

    /// Call a no-arg method that returns a String on `obj`.
    pub unsafe fn call_string_method(
        &self, obj: *mut c_void, method_name: &[u8], sig: &[u8],
    ) -> String {
        let cls = self.get_object_class(obj);
        if cls.is_null() { return String::new(); }
        let method = self.get_method_id(cls, method_name, sig);
        self.delete_local_ref(cls);
        if method.is_null() { return String::new(); }
        let result = self.call_object_method(obj, method, &[]);
        let s = self.get_string(result);
        self.delete_local_ref(result);
        s
    }

    pub unsafe fn get_object_class(&self, obj: *mut c_void) -> *mut c_void {
        // GetObjectClass is at index 31
        type GetObjectClassFn = unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
        let f: GetObjectClassFn = self.jni_fn(31);
        f(self.env, obj)
    }

    pub unsafe fn delete_local_ref(&self, obj: *mut c_void) {
        if !obj.is_null() {
            let f: DeleteLocalRefFn = self.jni_fn(IDX_DELETE_LOCAL_REF);
            f(self.env, obj);
        }
    }

    pub unsafe fn exception_check(&self) -> bool {
        let f: ExceptionCheckFn = self.jni_fn(IDX_EXCEPTION_CHECK);
        f(self.env) != 0
    }

    pub unsafe fn clear_exception(&self) {
        let f: ExceptionClearFn = self.jni_fn(IDX_EXCEPTION_CLEAR);
        f(self.env);
    }
}
