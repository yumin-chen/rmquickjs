use alloc::string::ToString;
use alloc::vec;
use alloc::{boxed::Box, ffi::CString, vec::Vec};
use core::ffi::{c_char, c_int};
use core::ptr::null_mut;
use core::{ffi::c_void, ptr::NonNull};
use rmquickjs_sys::{
    JS_AddGCRef, JS_EVAL_JSON, JS_EVAL_REGEXP, JS_EVAL_REPL, JS_EVAL_RETVAL, JS_EVAL_STRIP_COL,
    JS_NewCFunctionParams, JS_Parse, JSCFunctionEnum_JS_CFUNCTION_USER, JSContext, JSGCRef,
    JSValue, js_stdlib,
};

use crate::{Array, Error, Function, Object, Opaque, Result, Value};

/// A JavaScript engine context.
pub struct Context {
    mem: Option<Vec<u8>>,
    ctx: NonNull<rmquickjs_sys::JSContext>,
}

unsafe extern "C" fn host_callback(
    ctx: *mut JSContext,
    this_val: *mut JSValue,
    argc: i32,
    argv: *mut JSValue,
    params: JSValue,
) -> JSValue {
    unsafe {
        if argc < 0 {
            return Value::exception().into_raw();
        }

        let args_raw = core::slice::from_raw_parts(argv, argc as usize);
        let mut args = Vec::with_capacity(args_raw.len());
        for arg in args_raw {
            args.push(Value::from_raw(*arg));
        }

        let ctx = Context::from_raw(ctx);
        let this = if this_val.is_null() {
            None
        } else {
            Some(Value::from_raw(*this_val))
        };

        let opaque = ctx.get_opaque();
        let Some(index) = Value::from_raw(params).to_i32(&ctx) else {
            return Value::exception().into_raw();
        };

        let Some(func) = opaque.funcs.get(index as usize) else {
            return Value::exception().into_raw();
        };

        let result = func(&ctx, this, &args);

        match result {
            Ok(value) => value,
            Err(error) => error.exception,
        }
        .into_raw()
    }
}

unsafe extern "C" fn interrupt_callback(ctx: *mut JSContext, _: *mut c_void) -> c_int {
    let ctx = Context::from_raw(ctx);
    let opaque = ctx.get_opaque();
    if let Some(handler) = &opaque.interrupt_handler {
        if handler(&ctx) {
            return 1;
        }
    }
    0
}

#[derive(Default)]
pub struct EvalFlags(u32);

impl EvalFlags {
    pub fn inner(&self) -> u32 {
        self.0
    }

    pub fn with_retval(self) -> Self {
        Self(self.0 | JS_EVAL_RETVAL)
    }

    pub fn with_repl(self) -> Self {
        Self(self.0 | JS_EVAL_REPL)
    }

    pub fn with_strip_col(self) -> Self {
        Self(self.0 | JS_EVAL_STRIP_COL)
    }

    pub fn with_regexp(self) -> Self {
        Self(self.0 | JS_EVAL_REGEXP)
    }

    pub fn with_json(self) -> Self {
        Self(self.0 | JS_EVAL_JSON)
    }
}

/// Options for evaluating JavaScript code.
pub struct EvalOptions<'file_name> {
    file_name: Option<&'file_name str>,
    flags: EvalFlags,
}

impl Default for EvalOptions<'_> {
    fn default() -> Self {
        Self {
            file_name: Default::default(),
            flags: EvalFlags::default().with_retval(),
        }
    }
}

impl Context {
    /// Creates a new JavaScript context.
    pub fn new() -> Self {
        const DEFAULT_SIZE: usize = 65536;
        Self::with_memory_size(DEFAULT_SIZE)
    }

    /// Creates a new JavaScript context with the specified memory size.
    pub fn with_memory_size(memory_size: usize) -> Self {
        unsafe {
            let mut mem = vec![0; memory_size];
            let ctx = rmquickjs_sys::JS_NewContext(
                mem.as_mut_ptr() as *mut c_void,
                memory_size,
                &js_stdlib,
            );
            let opaque = Box::into_raw(Opaque::default().into());

            rmquickjs_sys::JS_SetContextOpaque(ctx, opaque as *mut c_void);
            rmquickjs_sys::JS_SetHostCallback(Some(host_callback));

            Self {
                mem: Some(mem),
                ctx: NonNull::new(ctx).expect("ctx should not be null"),
            }
        }
    }

    /// Creates a new JavaScript context with the specified memory size and prepare compilation option.
    pub fn with_prepare_compilation(memory_size: usize, prepare_compilation: bool) -> Self {
        unsafe {
            let mut mem = vec![0; memory_size];
            let ctx = rmquickjs_sys::JS_NewContext2(
                mem.as_mut_ptr() as *mut c_void,
                memory_size,
                &js_stdlib,
                prepare_compilation as i32,
            );
            let opaque = Box::into_raw(Opaque::default().into());

            rmquickjs_sys::JS_SetContextOpaque(ctx, opaque as *mut c_void);
            rmquickjs_sys::JS_SetHostCallback(Some(host_callback));

            Self {
                mem: Some(mem),
                ctx: NonNull::new(ctx).expect("ctx should not be null"),
            }
        }
    }

    pub(crate) fn from_raw(ctx: *mut rmquickjs_sys::JSContext) -> Self {
        Self {
            mem: None,
            ctx: NonNull::new(ctx).expect("ctx should not be null"),
        }
    }

    /// Acquires a raw pointer to the underlying `JSContext`.
    pub fn as_ptr(&self) -> *mut rmquickjs_sys::JSContext {
        self.ctx.as_ptr()
    }

    pub(crate) fn get_opaque(&self) -> &mut Opaque {
        unsafe { &mut *(rmquickjs_sys::JS_GetContextOpaque(self.ctx.as_ptr()) as *mut Opaque) }
    }

    /// Acquires the global object.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::{Context, Object};
    ///
    /// let ctx = Context::new();
    ///
    /// let global = ctx.globals();
    /// global.set("x", ctx.new_i32(42));
    /// let x = global.get("x").unwrap().to_i32(&ctx).unwrap();
    /// assert_eq!(x, 42);
    /// ```
    pub fn globals<'ctx>(&'ctx self) -> Object<'ctx> {
        let mut gc_ref = Box::new(JSGCRef {
            val: 0,
            prev: null_mut(),
        });
        unsafe {
            let slot = rmquickjs_sys::JS_AddGCRef(self.ctx.as_ptr(), &mut *gc_ref);
            *slot = rmquickjs_sys::JS_GetGlobalObject(self.ctx.as_ptr());
            Object::new(gc_ref, self)
        }
    }

    /// Evaluates JavaScript code.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let result = ctx.eval("1 + 2")
    ///     .unwrap()
    ///     .to_i32(&ctx)
    ///     .unwrap();
    /// assert_eq!(result, 3);
    /// ```
    pub fn eval(&self, input: &str) -> Result<Value> {
        self.eval_with_options(input, &EvalOptions::default())
    }

    /// Evaluates JavaScript code with the specified options.
    pub fn eval_with_options(&self, input: &str, options: &EvalOptions<'_>) -> Result<Value> {
        let c_input = CString::new(input).unwrap();
        let c_filename = CString::new(options.file_name.unwrap_or("index.js")).unwrap();

        let result = unsafe {
            Value::from_raw(rmquickjs_sys::JS_Eval(
                self.ctx.as_ptr(),
                c_input.as_ptr(),
                input.len(),
                c_filename.as_ptr(),
                options.flags.inner().try_into().unwrap(),
            ))
        };

        if result.is_exception() {
            let exception = self.get_exception().unwrap();
            let message = exception.to_string(self);
            Err(Error { message, exception })
        } else {
            Ok(result)
        }
    }

    /// Parses JavaScript code and generates bytecode.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let bytecode = ctx.parse("1 + 2")
    ///     .unwrap();
    /// let result = ctx.run(bytecode)
    ///     .unwrap()
    ///     .to_i32(&ctx)
    ///     .unwrap();
    /// assert_eq!(result, 3);
    /// ```
    pub fn parse(&self, input: &str) -> Result<Value> {
        self.parse_with_options(input, &EvalOptions::default())
    }

    /// Parses JavaScript code and generates bytecode with the specified options.
    pub fn parse_with_options(&self, input: &str, options: &EvalOptions<'_>) -> Result<Value> {
        let c_input = CString::new(input).unwrap();
        let c_filename = CString::new(options.file_name.unwrap_or("index.js")).unwrap();

        let result = unsafe {
            Value::from_raw(JS_Parse(
                self.ctx.as_ptr(),
                c_input.as_ptr(),
                input.len(),
                c_filename.as_ptr(),
                options.flags.inner().try_into().unwrap(),
            ))
        };

        if result.is_exception() {
            let exception = self.get_exception().unwrap();
            let message = exception.to_string(self);
            Err(Error { message, exception })
        } else {
            Ok(result)
        }
    }

    /// Relocates bytecode for the current context.
    pub fn relocate_bytecode(&self, bytecode: &mut [u8]) -> Result<()> {
        unsafe {
            let code = rmquickjs_sys::JS_RelocateBytecode(
                self.ctx.as_ptr(),
                bytecode.as_mut_ptr(),
                bytecode.len() as u32,
            );
            if code != 0 {
                Err(Error {
                    message: "Could not relocate bytecode".to_string(),
                    exception: Value::exception(),
                })
            } else {
                Ok(())
            }
        }
    }

    /// Loads bytecode into the current context.
    pub fn load_bytecode(&self, bytecode: &[u8]) -> Result<Value> {
        unsafe {
            let result = Value::from_raw(rmquickjs_sys::JS_LoadBytecode(
                self.ctx.as_ptr(),
                bytecode.as_ptr(),
            ));

            if result.is_exception() {
                let exception = self.get_exception().unwrap();
                let message = exception.to_string(self);
                Err(Error { message, exception })
            } else {
                Ok(result)
            }
        }
    }

    /// Runs bytecode in the current context.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let bytecode = ctx.parse("1 + 2")
    ///     .unwrap();
    /// let result = ctx.run(bytecode)
    ///     .unwrap()
    ///     .to_i32(&ctx)
    ///     .unwrap();
    /// assert_eq!(result, 3);
    /// ```
    pub fn run(&self, bytecode: Value) -> Result<Value> {
        unsafe {
            let result = Value::from_raw(rmquickjs_sys::JS_Run(
                self.ctx.as_ptr(),
                bytecode.into_raw(),
            ));

            if result.is_exception() {
                let exception = self.get_exception().unwrap();
                let message = exception.to_string(self);
                Err(Error { message, exception })
            } else {
                Ok(result)
            }
        }
    }

    /// Performs garbage collection.
    pub fn gc(&self) {
        unsafe {
            rmquickjs_sys::JS_GC(self.as_ptr());
        }
    }

    /// Gets the current exception object, if any.
    pub fn get_exception(&self) -> Option<Value> {
        unsafe {
            let ex = Value::from_raw(rmquickjs_sys::JS_GetException(self.ctx.as_ptr()));
            if ex.is_undefined() { None } else { Some(ex) }
        }
    }

    /// Throws a JavaScript exception with the given value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let result = ctx.throw(ctx.new_string("an error occurred"));
    /// assert!(result.is_err());
    /// ```
    pub fn throw(&self, value: Value) -> Result<Value> {
        unsafe {
            Value::from_raw(rmquickjs_sys::JS_Throw(self.ctx.as_ptr(), value.into_raw()))
                .to_result(self)
        }
    }

    fn throw_error(&self, class: rmquickjs_sys::JSObjectClassEnum, message: &str) -> Result<Value> {
        Value::from_raw(unsafe {
            rmquickjs_sys::JS_ThrowError(
                self.ctx.as_ptr(),
                class,
                CString::new(message).unwrap().as_ptr(),
            )
        })
        .to_result(self)
    }

    /// Throws a JavaScript `TypeError`
    pub fn throw_type_error(&self, message: &str) -> Result<Value> {
        self.throw_error(
            rmquickjs_sys::JSObjectClassEnum_JS_CLASS_TYPE_ERROR,
            message,
        )
    }

    /// Throws a JavaScript `ReferenceError`
    pub fn throw_reference_error(&self, message: &str) -> Result<Value> {
        self.throw_error(
            rmquickjs_sys::JSObjectClassEnum_JS_CLASS_REFERENCE_ERROR,
            message,
        )
    }

    /// Throws a JavaScript `InternalError`
    pub fn throw_internal_error(&self, message: &str) -> Result<Value> {
        self.throw_error(
            rmquickjs_sys::JSObjectClassEnum_JS_CLASS_INTERNAL_ERROR,
            message,
        )
    }

    /// Throws a JavaScript `RangeError`
    pub fn throw_range_error(&self, message: &str) -> Result<Value> {
        self.throw_error(
            rmquickjs_sys::JSObjectClassEnum_JS_CLASS_RANGE_ERROR,
            message,
        )
    }

    /// Throws a JavaScript `SyntaxError`
    pub fn throw_syntax_error(&self, message: &str) -> Result<Value> {
        self.throw_error(
            rmquickjs_sys::JSObjectClassEnum_JS_CLASS_SYNTAX_ERROR,
            message,
        )
    }

    /// Throws an out of memory exception.
    pub fn throw_out_of_memory(&self) -> Result<Value> {
        unsafe {
            Value::from_raw(rmquickjs_sys::JS_ThrowOutOfMemory(self.as_ptr())).to_result(self)
        }
    }

    /// Creates a new JavaScript value from an `i32`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let value = ctx.new_i32(42);
    /// assert_eq!(value.to_i32(&ctx).unwrap(), 42);
    /// ```
    pub fn new_i32(&self, value: i32) -> Value {
        unsafe { Value::from_raw(rmquickjs_sys::JS_NewInt32(self.as_ptr(), value)) }
    }

    /// Creates a new JavaScript value from an `i64`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let value = ctx.new_i64(42);
    /// assert_eq!(value.to_i32_sat(&ctx).unwrap(), 42);
    /// ```
    pub fn new_i64(&self, value: i64) -> Value {
        unsafe { Value::from_raw(rmquickjs_sys::JS_NewInt64(self.as_ptr(), value)) }
    }

    /// Creates a new JavaScript value from a `f64`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let value = ctx.new_f64(3.14);
    /// assert_eq!(value.to_number(&ctx).unwrap(), 3.14);
    /// ```
    pub fn new_f64(&self, value: f64) -> Value {
        unsafe { Value::from_raw(rmquickjs_sys::JS_NewFloat64(self.as_ptr(), value)) }
    }

    /// Creates a new JavaScript value from a `u32`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let value = ctx.new_u32(42);
    /// assert_eq!(value.to_u32(&ctx).unwrap(), 42);
    /// ```
    pub fn new_u32(&self, value: u32) -> Value {
        unsafe { Value::from_raw(rmquickjs_sys::JS_NewUint32(self.as_ptr(), value)) }
    }

    /// Creates a new JavaScript value from a `str`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let value = ctx.new_string("Hello, world!");
    /// assert_eq!(value.to_string(&ctx), "Hello, world!".to_string());
    /// ```
    pub fn new_string(&self, value: &str) -> Value {
        unsafe {
            Value::from_raw(rmquickjs_sys::JS_NewStringLen(
                self.as_ptr(),
                value.as_ptr() as *const c_char,
                value.len(),
            ))
        }
    }

    /// Creates a new JavaScript array with the specified length.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let array = ctx.new_array(3).unwrap();
    /// array.set(0, ctx.new_i32(1)).unwrap();
    /// array.set(1, ctx.new_i32(2)).unwrap();
    /// array.set(2, ctx.new_i32(3)).unwrap();
    ///
    /// let value = array.get(1).unwrap();
    /// assert_eq!(value.to_i32(&ctx).unwrap(), 2);
    /// ```
    pub fn new_array<'ctx>(&'ctx self, len: usize) -> Result<Array<'ctx>> {
        unsafe {
            let mut gc_ref = Box::new(JSGCRef {
                val: 0,
                prev: null_mut(),
            });
            let slot = rmquickjs_sys::JS_AddGCRef(self.as_ptr(), &mut *gc_ref);

            gc_ref.val = rmquickjs_sys::JS_NewArray(self.as_ptr(), len as i32);
            *slot = gc_ref.val.into();

            let value = Value::from_raw(*slot);
            if value.is_exception() {
                rmquickjs_sys::JS_DeleteGCRef(self.as_ptr(), &mut *gc_ref);
                Err(Error {
                    message: value.to_string(self),
                    exception: value,
                })
            } else {
                Ok(Array::new(gc_ref, self))
            }
        }
    }

    /// Creates a new JavaScript object.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let obj = ctx.new_object().unwrap();
    /// obj.set("x", ctx.new_i32(42)).unwrap();
    /// let x = obj.get("x").unwrap().to_i32(&ctx).unwrap();
    /// assert_eq!(x, 42);
    /// ```
    pub fn new_object<'ctx>(&'ctx self) -> Result<Object<'ctx>> {
        unsafe {
            let mut gc_ref = Box::new(JSGCRef {
                val: 0,
                prev: null_mut(),
            });
            let slot = rmquickjs_sys::JS_AddGCRef(self.as_ptr(), &mut *gc_ref);

            gc_ref.val = rmquickjs_sys::JS_NewObject(self.as_ptr());
            *slot = gc_ref.val.into();

            let value = Value::from_raw(*slot);
            if value.is_exception() {
                rmquickjs_sys::JS_DeleteGCRef(self.as_ptr(), &mut *gc_ref);
                Err(Error {
                    message: value.to_string(self),
                    exception: value,
                })
            } else {
                Ok(Object::new(gc_ref, self))
            }
        }
    }

    /// Creates a new JavaScript function from a Rust closure.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rmquickjs::Context;
    ///
    /// let ctx = Context::new();
    /// let func = ctx.new_function(|ctx, this, args| {
    ///     if args.len() < 2 {
    ///        ctx.throw(ctx.new_string("expected 2 arguments"))?;
    ///     }
    ///
    ///     let a = args[0].to_i32(ctx).unwrap(0);
    ///     let b = args[1].to_i32(ctx).unwrap(0);
    ///     Ok(ctx.new_i32(a + b))
    /// }).unwrap();
    ///
    /// ctx.globals().set("add", func).unwrap();
    /// let result = ctx.eval("add(1, 2)")
    ///     .unwrap()
    ///     .to_i32(&ctx)
    ///     .unwrap();
    /// assert_eq!(result, 3);
    /// ```
    pub fn new_function<'ctx, F>(&'ctx self, func: F) -> Result<Function<'ctx>>
    where
        F: Fn(&Context, Option<Value>, &[Value]) -> Result<Value> + 'static,
    {
        unsafe {
            let opaque = self.get_opaque();
            let index = opaque.funcs.len();
            opaque.funcs.push(Box::new(func));

            let mut gc_ref = Box::new(JSGCRef {
                val: 0,
                prev: null_mut(),
            });
            let slot = JS_AddGCRef(self.as_ptr(), &mut *gc_ref);

            gc_ref.val = JS_NewCFunctionParams(
                self.as_ptr(),
                (JSCFunctionEnum_JS_CFUNCTION_USER + 0) as c_int, // js_rmquickjs_callback
                self.new_u32(index as u32).into_raw(),
            );
            *slot = gc_ref.val;

            let value = Value::from_raw(*slot);
            if value.is_exception() {
                rmquickjs_sys::JS_DeleteGCRef(self.as_ptr(), &mut *gc_ref);
                Err(Error {
                    message: value.to_string(self),
                    exception: value,
                })
            } else {
                Ok(Function::new(gc_ref, self))
            }
        }
    }

    /// Checks if the stack has enough space.
    pub fn stack_check(&self, len: u32) -> bool {
        unsafe { rmquickjs_sys::JS_StackCheck(self.as_ptr(), len) == 0 }
    }

    /// Sets the random seed for the context.
    pub fn set_random_seed(&self, seed: u64) {
        unsafe { rmquickjs_sys::JS_SetRandomSeed(self.as_ptr(), seed) }
    }

    /// Sets the interrupt handler for the context.
    pub fn set_interrupt_handler<F>(&self, handler: F)
    where
        F: Fn(&Context) -> bool + 'static,
    {
        unsafe {
            let opaque = self.get_opaque();
            opaque.interrupt_handler = Some(Box::new(handler));
            rmquickjs_sys::JS_SetInterruptHandler(self.as_ptr(), Some(interrupt_callback));
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            if let Some(_) = self.mem.take() {
                let ctx = self.ctx.as_ptr();
                let opaque = self.get_opaque();
                rmquickjs_sys::JS_FreeContext(ctx);
                drop(Box::from_raw(opaque));
            }
        };
    }
}
