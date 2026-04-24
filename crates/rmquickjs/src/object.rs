use crate::{Context, Error, Result, Value};
use alloc::{boxed::Box, ffi::CString};
use rmquickjs_sys::JSGCRef;

/// Represents a JavaScript object.
pub struct Object<'ctx> {
    gc_ref: Box<JSGCRef>,
    ctx: &'ctx Context,
}

impl Into<Value> for Object<'_> {
    fn into(self) -> Value {
        Value::from_raw(self.gc_ref.val)
    }
}

impl<'ctx> Drop for Object<'ctx> {
    fn drop(&mut self) {
        unsafe {
            rmquickjs_sys::JS_DeleteGCRef(self.ctx.as_ptr(), &mut *self.gc_ref);
        }
    }
}

impl<'ctx> Object<'ctx> {
    pub(crate) fn new(gc_ref: Box<JSGCRef>, ctx: &'ctx Context) -> Self {
        Object { gc_ref, ctx }
    }

    /// Gets the property of the object.
    /// 
    /// ## Examples
    /// 
    /// ```rust
    /// use rmquickjs::Context;
    /// 
    /// let ctx = Context::new();
    /// let obj = ctx
    ///     .eval("({ a: 1, b: 2 })")
    ///     .unwrap()
    ///     .to_object(&ctx)
    ///     .unwrap();
    /// let value = obj.get("a").unwrap();
    /// assert_eq!(value, ctx.new_i32(1));
    /// let value = obj.get("b").unwrap();
    /// assert_eq!(value, ctx.new_i32(2));
    /// ```
    pub fn get(&self, key: &str) -> Result<Value> {
        unsafe {
            let value = Value::from_raw(rmquickjs_sys::JS_GetPropertyStr(
                self.ctx.as_ptr(),
                self.gc_ref.val.into(),
                CString::new(key).unwrap().as_ptr(),
            ));

            if value.is_exception() {
                Err(Error {
                    message: value.to_string(self.ctx),
                    exception: value,
                })
            } else {
                Ok(value)
            }
        }
    }

    /// Sets the property of the object.
    /// 
    /// ## Examples
    //// 
    /// ```rust
    /// use rmquickjs::Context;
    /// 
    /// let ctx = Context::new();
    /// let obj = ctx
    ///     .eval("({ a: 1, b: 2 })")
    ///     .unwrap()
    ///     .to_object(&ctx)
    ///     .unwrap();
    /// let value = obj.set("c", ctx.new_i32(3));
    /// assert_eq!(value, Value::undefined());
    /// let value = obj.get("c").unwrap();
    /// assert_eq!(value, ctx.new_i32(3));
    /// ```
    pub fn set(&self, key: &str, value: impl Into<Value>) -> Value {
        unsafe {
            let value = rmquickjs_sys::JS_SetPropertyStr(
                self.ctx.as_ptr(),
                self.gc_ref.val.into(),
                CString::new(key).unwrap().as_ptr(),
                value.into().into_raw(),
            );
            Value::from_raw(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_object() {
        let ctx = Context::new();

        let obj = ctx
            .eval("({ a: 1, b: 2 })")
            .unwrap()
            .to_object(&ctx)
            .unwrap();

        let value = obj.get("a").unwrap();
        assert_eq!(value, ctx.new_i32(1));
        let value = obj.get("b").unwrap();
        assert_eq!(value, ctx.new_i32(2));

        let value = obj.set("c", ctx.new_i32(3));
        assert_eq!(value, Value::undefined());
        let value = obj.get("c").unwrap();
        assert_eq!(value, ctx.new_i32(3));
    }
}
