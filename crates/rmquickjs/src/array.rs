use alloc::{boxed::Box, ffi::CString};
use rmquickjs_sys::JSGCRef;

use crate::{Context, Error, Result, Value};

/// Represents a JavaScript array.
pub struct Array<'ctx> {
    gc_ref: Box<JSGCRef>,
    ctx: &'ctx Context,
}

impl<'ctx> Into<Value> for Array<'ctx> {
    fn into(self) -> Value {
        Value::from_raw(self.gc_ref.val)
    }
}

impl<'ctx> Drop for Array<'ctx> {
    fn drop(&mut self) {
        unsafe {
            rmquickjs_sys::JS_DeleteGCRef(self.ctx.as_ptr(), &mut *self.gc_ref);
        }
    }
}

impl<'ctx> Array<'ctx> {
    pub(crate) fn new(gc_ref: Box<JSGCRef>, ctx: &'ctx Context) -> Self {
        Self { gc_ref, ctx }
    }

    /// Returns the length of the array.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rmquickjs::{Context, Result};
    /// 
    /// let ctx = Context::new();
    /// let arr = ctx.eval("[1, 2, 3]")
    ///     .unwrap()
    ///     .to_array(&ctx)
    ///     .unwrap();
    /// assert_eq!(arr.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        unsafe {
            let len = Value::from_raw(rmquickjs_sys::JS_GetPropertyStr(
                self.ctx.as_ptr(),
                self.gc_ref.val,
                CString::new("length").unwrap().as_ptr(),
            ));
            len.to_i32(self.ctx).unwrap_or_default() as usize
        }
    }

    /// Gets the element of the array.
    /// 
    /// # Examples
    //// 
    /// ```rust
    /// use rmquickjs::Context;
    /// 
    /// let ctx = Context::new();
    /// let arr = ctx.eval("[1, 2, 3]")
    ///     .unwrap()
    ///     .to_array(&ctx)
    ///     .unwrap();
    /// assert_eq!(arr.get(0).unwrap().to_i32(&ctx), Some(1));
    /// assert_eq!(arr.get(1).unwrap().to_i32(&ctx), Some(2));
    /// assert_eq!(arr.get(2).unwrap().to_i32(&ctx), Some(3));
    /// ```
    pub fn get(&self, index: usize) -> Result<Value> {
        unsafe {
            let value = Value::from_raw(rmquickjs_sys::JS_GetPropertyUint32(
                self.ctx.as_ptr(),
                self.gc_ref.val,
                index as u32,
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

    /// Sets the element of the array.
    /// 
    /// ## Examples
    //// 
    /// ```rust
    /// use rmquickjs::Context;
    /// 
    /// let ctx = Context::new();
    /// let arr = ctx.eval("[1, 2, 0]")
    ///     .unwrap()
    ///     .to_array(&ctx)
    ///     .unwrap();
    /// arr.set(2, ctx.new_i32(3)).unwrap();
    /// assert_eq!(arr.get(2).unwrap().to_i32(&ctx), Some(3));
    /// ```
    pub fn set(&self, index: usize, value: Value) -> Result<Value> {
        unsafe {
            let value = Value::from_raw(rmquickjs_sys::JS_SetPropertyUint32(
                self.ctx.as_ptr(),
                self.gc_ref.val,
                index as u32,
                value.into_raw(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array() {
        let ctx = Context::new();
        let arr = ctx.eval("[1, 2, 0]").unwrap().to_array(&ctx).unwrap();
        assert_eq!(arr.len(), 3);

        arr.set(0, ctx.new_i32(1)).unwrap();
        arr.set(1, ctx.new_i32(2)).unwrap();
        arr.set(2, ctx.new_i32(3)).unwrap();

        assert_eq!(arr.get(0).unwrap().to_i32(&ctx), Some(1));
        assert_eq!(arr.get(1).unwrap().to_i32(&ctx), Some(2));
        assert_eq!(arr.get(2).unwrap().to_i32(&ctx), Some(3));
    }
}
