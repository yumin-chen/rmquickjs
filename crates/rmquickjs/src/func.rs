use alloc::{boxed::Box, string::ToString};
use core::ffi::c_int;
use rmquickjs_sys::{JS_Call, JS_DeleteGCRef, JS_PushArg, JSGCRef};

use crate::{Context, Error, Result, Value};

/// Represents a JavaScript function.
pub struct Function<'ctx> {
    gc_ref: Box<JSGCRef>,
    ctx: &'ctx Context,
}

impl Into<Value> for Function<'_> {
    fn into(self) -> Value {
        Value::from_raw(self.gc_ref.val)
    }
}

impl<'ctx> Drop for Function<'ctx> {
    fn drop(&mut self) {
        unsafe {
            JS_DeleteGCRef(self.ctx.as_ptr(), &mut *self.gc_ref);
        }
    }
}

impl<'ctx> Function<'ctx> {
    pub(crate) fn new(gc_ref: Box<JSGCRef>, ctx: &'ctx Context) -> Self {
        Self { gc_ref, ctx }
    }

    /// Calls the function with the given arguments.
    /// 
    /// ## Examples
    /// 
    /// ```rust
    /// use rmquickjs::Context;
    ///     
    /// let ctx = Context::new();
    /// let func = ctx
    ///     .eval("function add(a, b) { return a + b; } add")
    ///     .unwrap()
    ///     .to_function(&ctx)
    ///     .unwrap();
    /// let result = func
    ///     .call(&[ctx.new_i32(1), ctx.new_i32(2)])
    ///     .unwrap()
    ///     .to_i32(&ctx);
    /// assert_eq!(result, Some(3));
    /// ```
    pub fn call(&self, args: &[Value]) -> Result<Value> {
        unsafe {
            if !self.ctx.stack_check((args.len() + 2) as u32) {
                return Err(Error {
                    message: "stack overflow".to_string(),
                    exception: Value::exception(),
                });
            }

            for arg in args.iter().rev() {
                JS_PushArg(self.ctx.as_ptr(), arg.into_raw());
            }

            JS_PushArg(self.ctx.as_ptr(), self.gc_ref.val);
            JS_PushArg(self.ctx.as_ptr(), Value::null().into_raw());

            let ret: Value = Value::from_raw(JS_Call(self.ctx.as_ptr(), args.len() as c_int));
            if ret.is_exception() {
                Err(Error {
                    message: ret.to_string(self.ctx),
                    exception: ret,
                })
            } else {
                Ok(ret)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_call_js_func_from_rust() {
        let ctx = Context::new();
        let func = ctx
            .eval("function add(a, b) { return a + b; } add")
            .unwrap()
            .to_function(&ctx)
            .unwrap();
        let result = func
            .call(&[ctx.new_i32(1), ctx.new_i32(2)])
            .unwrap()
            .to_i32(&ctx);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_func_call_rust_closure_from_js() {
        let ctx = Context::new();

        let func = ctx
            .new_function(|ctx, _, args| {
                if args.len() != 2 {
                    ctx.throw(ctx.new_string("invalid number of arguments"))?;
                }

                let a = args[0].to_i32(ctx).unwrap();
                let b = args[1].to_i32(ctx).unwrap();
                Ok(ctx.new_i32(a + b))
            })
            .unwrap();
        ctx.globals().set("add", func);

        let func = ctx
            .new_function(|ctx, _, args| {
                if args.len() != 2 {
                    ctx.throw(ctx.new_string("invalid number of arguments"))?;
                }

                let a = args[0].to_i32(ctx).unwrap();
                let b = args[1].to_i32(ctx).unwrap();

                Ok(ctx.new_i32(a - b))
            })
            .unwrap();
        ctx.globals().set("sub", func);

        let result = ctx.eval("add(1, 2)").unwrap();
        assert_eq!(result, ctx.new_i32(3));

        let result = ctx.eval("sub(3, 2)").unwrap();
        assert_eq!(result, ctx.new_i32(1));

        let result = ctx
            .globals()
            .get("add")
            .unwrap()
            .to_function(&ctx)
            .unwrap()
            .call(&[ctx.new_i32(1), ctx.new_i32(2)])
            .unwrap()
            .to_i32(&ctx);
        assert_eq!(result, Some(3));

        let result = ctx
            .globals()
            .get("sub")
            .unwrap()
            .to_function(&ctx)
            .unwrap()
            .call(&[ctx.new_i32(3), ctx.new_i32(2)])
            .unwrap()
            .to_i32(&ctx);
        assert_eq!(result, Some(1));
    }
}
