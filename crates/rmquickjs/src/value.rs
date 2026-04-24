use core::{ffi::CStr, ptr::null_mut};

use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use rmquickjs_sys::{
    JSCStringBuf, JSGCRef, JSObjectClassEnum_JS_CLASS_ARRAY,
    JSObjectClassEnum_JS_CLASS_FLOAT32_ARRAY, JSObjectClassEnum_JS_CLASS_FLOAT64_ARRAY,
    JSObjectClassEnum_JS_CLASS_INT8_ARRAY, JSObjectClassEnum_JS_CLASS_INT16_ARRAY,
    JSObjectClassEnum_JS_CLASS_INT32_ARRAY, JSObjectClassEnum_JS_CLASS_TYPED_ARRAY,
    JSObjectClassEnum_JS_CLASS_UINT8_ARRAY, JSObjectClassEnum_JS_CLASS_UINT16_ARRAY,
    JSObjectClassEnum_JS_CLASS_UINT32_ARRAY, JSValue,
};

use crate::{Array, Context, Error, Function, Object, Result};

/// Represents a JavaScript value.
///
/// note: If this value points to a reference, it may be invalidated during garbage collection.
/// It is not recommended to store it in fields or similar locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value(JSValue);

impl Value {
    /// Creates a new `Value` from a raw JavaScript value.
    pub fn from_raw(value: JSValue) -> Self {
        Self(value)
    }

    /// Converts this `Value` into a raw JavaScript value.
    pub fn into_raw(self) -> JSValue {
        self.0
    }

    /// Converts this `Value` into a `Result`, treating exceptions as errors.
    pub fn to_result(self, ctx: &Context) -> Result<Self> {
        if self.is_exception() {
            Err(Error {
                message: self.to_string(ctx),
                exception: self,
            })
        } else {
            Ok(self)
        }
    }

    pub fn bool(value: bool) -> Self {
        Self(if value {
            rmquickjs_sys::JS_TRUE()
        } else {
            rmquickjs_sys::JS_FALSE()
        })
    }

    pub fn null() -> Self {
        Self(rmquickjs_sys::JS_NULL())
    }

    pub fn undefined() -> Self {
        Self(rmquickjs_sys::JS_UNDEFINED())
    }

    pub fn uninitialized() -> Self {
        Self(rmquickjs_sys::JS_UNINITIALIZED())
    }

    pub fn exception() -> Self {
        Self(rmquickjs_sys::JS_EXCEPTION())
    }

    pub const fn is_exception(&self) -> bool {
        self.0 == rmquickjs_sys::JS_EXCEPTION()
    }

    pub const fn is_null(&self) -> bool {
        self.0 == rmquickjs_sys::JS_NULL()
    }

    pub const fn is_undefined(&self) -> bool {
        self.0 == rmquickjs_sys::JS_UNDEFINED()
    }

    pub const fn is_uninitialized(&self) -> bool {
        self.0 == rmquickjs_sys::JS_UNINITIALIZED()
    }

    pub const fn is_true(&self) -> bool {
        self.0 == rmquickjs_sys::JS_TRUE()
    }

    pub const fn is_false(&self) -> bool {
        self.0 == rmquickjs_sys::JS_FALSE()
    }

    pub const fn is_bool(&self) -> bool {
        rmquickjs_sys::JS_IsBool(self.0)
    }

    pub const fn is_int(&self) -> bool {
        rmquickjs_sys::JS_IsInt(self.0)
    }

    pub const fn is_ptr(&self) -> bool {
        rmquickjs_sys::JS_IsPtr(self.0)
    }

    pub fn is_number(&self, ctx: &Context) -> bool {
        unsafe { rmquickjs_sys::JS_IsNumber(ctx.as_ptr(), self.0) == 1 }
    }

    pub fn is_string(&self, ctx: &Context) -> bool {
        unsafe { rmquickjs_sys::JS_IsString(ctx.as_ptr(), self.0) == 1 }
    }

    pub fn is_function(&self, ctx: &Context) -> bool {
        unsafe { rmquickjs_sys::JS_IsFunction(ctx.as_ptr(), self.0) == 1 }
    }

    pub fn is_object(&self, ctx: &Context) -> bool {
        unsafe { rmquickjs_sys::JS_GetClassID(ctx.as_ptr(), self.0) != -1 }
    }

    #[allow(non_upper_case_globals)]
    pub fn is_array(&self, ctx: &Context) -> bool {
        unsafe {
            let class_id = rmquickjs_sys::JS_GetClassID(ctx.as_ptr(), self.0);
            if class_id == -1 {
                return false;
            }

            matches!(
                class_id as u32,
                JSObjectClassEnum_JS_CLASS_ARRAY
                    | JSObjectClassEnum_JS_CLASS_INT8_ARRAY
                    | JSObjectClassEnum_JS_CLASS_INT16_ARRAY
                    | JSObjectClassEnum_JS_CLASS_INT32_ARRAY
                    | JSObjectClassEnum_JS_CLASS_UINT8_ARRAY
                    | JSObjectClassEnum_JS_CLASS_UINT16_ARRAY
                    | JSObjectClassEnum_JS_CLASS_UINT32_ARRAY
                    | JSObjectClassEnum_JS_CLASS_FLOAT32_ARRAY
                    | JSObjectClassEnum_JS_CLASS_FLOAT64_ARRAY
                    | JSObjectClassEnum_JS_CLASS_TYPED_ARRAY
            )
        }
    }

    pub fn is_error(&self, ctx: &Context) -> bool {
        unsafe { rmquickjs_sys::JS_IsError(ctx.as_ptr(), self.0) == 1 }
    }

    #[allow(non_snake_case)]
    pub fn to_bool(&self) -> Option<bool> {
        let tag = rmquickjs_sys::JS_VALUE_GET_SPECIAL_TAG(self.into_raw());
        let val = rmquickjs_sys::JS_VALUE_GET_SPECIAL_VALUE(self.into_raw());
        match (tag as u32, val) {
            (rmquickjs_sys::JS_TAG_BOOL, 1) => Some(true),
            (rmquickjs_sys::JS_TAG_BOOL, 0) => Some(false),
            _ => None,
        }
    }

    pub fn to_number(&self, ctx: &Context) -> Option<f64> {
        let mut result = 0.0;
        unsafe {
            if rmquickjs_sys::JS_ToNumber(ctx.as_ptr(), &mut result, self.0) == 0 {
                Some(result)
            } else {
                None
            }
        }
    }

    pub fn to_i32(&self, ctx: &Context) -> Option<i32> {
        let mut result = 0;
        unsafe {
            if rmquickjs_sys::JS_ToInt32(ctx.as_ptr(), &mut result, self.0) == 0 {
                Some(result)
            } else {
                None
            }
        }
    }

    pub fn to_i32_sat(&self, ctx: &Context) -> Option<i32> {
        let mut result = 0;
        unsafe {
            if rmquickjs_sys::JS_ToInt32Sat(ctx.as_ptr(), &mut result, self.0) == 0 {
                Some(result)
            } else {
                None
            }
        }
    }

    pub fn to_u32(&self, ctx: &Context) -> Option<u32> {
        let mut result = 0;
        unsafe {
            if rmquickjs_sys::JS_ToUint32(ctx.as_ptr(), &mut result, self.0) == 0 {
                Some(result)
            } else {
                None
            }
        }
    }

    pub fn to_string(&self, ctx: &Context) -> String {
        unsafe {
            let mut buf = JSCStringBuf { buf: [0; 5] };
            let mut len = 0;
            let str = rmquickjs_sys::JS_ToCStringLen(ctx.as_ptr(), &mut len, self.0, &mut buf);
            CStr::from_ptr(str).to_string_lossy().to_string()
        }
    }

    pub fn to_object<'a>(&self, ctx: &'a Context) -> Option<Object<'a>> {
        if self.is_object(ctx) {
            unsafe {
                let mut gc_ref = Box::new(JSGCRef {
                    val: self.0,
                    prev: null_mut(),
                });

                let slot = rmquickjs_sys::JS_AddGCRef(ctx.as_ptr(), gc_ref.as_mut());
                *slot = self.0;

                Some(Object::new(gc_ref, ctx))
            }
        } else {
            None
        }
    }

    pub fn to_array<'a>(&self, ctx: &'a Context) -> Option<Array<'a>> {
        if self.is_array(ctx) {
            unsafe {
                let mut gc_ref = Box::new(JSGCRef {
                    val: self.0,
                    prev: null_mut(),
                });

                let slot = rmquickjs_sys::JS_AddGCRef(ctx.as_ptr(), gc_ref.as_mut());
                *slot = self.0;

                Some(Array::new(gc_ref, ctx))
            }
        } else {
            None
        }
    }

    pub fn to_function<'a>(&self, ctx: &'a Context) -> Option<Function<'a>> {
        if self.is_function(ctx) {
            unsafe {
                let mut gc_ref = Box::new(JSGCRef {
                    val: self.0,
                    prev: null_mut(),
                });

                let slot = rmquickjs_sys::JS_AddGCRef(ctx.as_ptr(), gc_ref.as_mut());
                *slot = self.0;

                Some(Function::new(gc_ref, ctx))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw() {
        let ctx = Context::new();

        let raw = ctx.new_i32(100).into_raw();
        let v = Value::from_raw(raw);
        assert_eq!(v.into_raw(), raw);
    }

    #[test]
    fn test_special_values() {
        // null / undefined / uninitialized / exception
        let n = Value::null();
        assert!(n.is_null());
        let u = Value::undefined();
        assert!(u.is_undefined());
        let un = Value::uninitialized();
        assert!(un.is_uninitialized());
        let ex = Value::exception();
        assert!(ex.is_exception());
        assert!(ex.is_exception());
    }

    #[test]
    fn test_bool() {
        let t = Value::bool(true);
        assert!(t.is_bool());
        assert!(t.is_true());
        let f = Value::bool(false);
        assert!(f.is_bool());
        assert!(f.is_false());
    }

    #[test]
    fn test_i32() {
        let ctx = Context::new();

        let v = ctx.new_i32(42);
        assert_eq!(v.to_i32(&ctx), Some(42));
        assert_eq!(v.to_string(&ctx), "42".to_string());

        let v = ctx.new_i32(-123456);
        assert_eq!(v.to_i32(&ctx), Some(-123456));
        assert_eq!(v.to_string(&ctx), "-123456".to_string());
    }

    #[test]
    fn test_u32() {
        let ctx = Context::new();

        let v = ctx.new_u32(255);
        assert_eq!(v.to_u32(&ctx), Some(255));
        assert_eq!(v.to_string(&ctx), "255".to_string());
    }

    #[test]
    fn test_f64() {
        let ctx = Context::new();

        let v = ctx.new_f64(3.1415);
        assert_eq!(
            v.to_number(&ctx).map(|n| (n * 10000.0).round() / 10000.0),
            Some(3.1415)
        );

        // to_i32 on float should convert when possible
        let v = ctx.new_f64(7.0);
        assert_eq!(v.to_i32(&ctx), Some(7));
    }

    #[test]
    fn test_string_to_number() {
        let ctx = Context::new();

        let s = ctx.new_string("3.5");
        assert!(s.is_string(&ctx));
        assert_eq!(s.to_number(&ctx), Some(3.5));

        let s = ctx.new_string("notanumber");
        assert!(s.is_string(&ctx));
        assert!(s.to_number(&ctx).unwrap().is_nan());
    }

    #[test]
    fn test_is_checks_and_errors() {
        let ctx = Context::new();

        // is_number / is_string / is_function / is_object / is_array / is_error
        let n = ctx.new_f64(1.23);
        assert!(n.is_number(&ctx));

        let s = ctx.new_string("hello");
        assert!(s.is_string(&ctx));

        let f = ctx
            .new_function(|ctx, _, _| Value::null().to_result(&ctx))
            .unwrap();
        let fval: Value = f.into();
        assert!(fval.is_function(&ctx));

        let o = ctx.new_object().unwrap();
        let oval: Value = o.into();
        assert!(oval.is_object(&ctx));

        let a = ctx.new_array(0).unwrap();
        let aval: Value = a.into();
        assert!(aval.is_array(&ctx));

        // is_error via evaluating new Error(...)
        let err = ctx.eval("new Error('oops')").unwrap();
        assert!(err.is_error(&ctx));
    }

    #[test]
    fn test_i32_sat() {
        let ctx = Context::new();

        let v = ctx.new_i64(12345);
        assert_eq!(v.to_i32_sat(&ctx), Some(12345));
    }
}
