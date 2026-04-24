# rmquickjs

High-level MicroQuickJS bindings for Rust

[![Crates.io version](https://img.shields.io/crates/v/rmquickjs.svg?style=flat-square)](https://crates.io/crates/rmquickjs)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs)

rmquickjs is a library that provides a high-level API for [MicroQuickJS](https://github.com/bellard/mquickjs), an embedded JS runtime created by the author of QuickJS.

This library is inspired by the existing Rust bindings [mquickjs-rs](https://github.com/fcoury/mquickjs-rs), but rmquickjs offers more features and a more ergonomic API.

## Features

- High-level API for the MicroQuickJS C API
- Supports function calls between Rust and JS
- no_std support

## Installation

```bash
$ cargo add rmquickjs
```

## Quick Start

```rs
use rmquickjs::*;

fn main() -> Result<()> {
    // Initialize the MicroQuickJS engine
    let ctx = Context::new();
    
    // Execute JS with eval()
    let result = ctx.eval("1 + 2")?;
    assert_eq!(result.to_i32(&ctx), Some(3));

    // Access global variables with globals()
    ctx.eval("var x = 'hello'")?;
    let x = ctx.globals().get("x")?;
    assert_eq!(x.to_string(&ctx), "hello".to_string());

    // Call JS functions from Rust
    ctx.eval(
        r#"
function add(x, y) {
    return x + y;
}"#,
    )?;

    let add = ctx.globals().get("add")?.to_function(&ctx).unwrap();

    let result = add.call(&[ctx.new_i32(1), ctx.new_i32(2)])?;
    assert_eq!(result.to_i32(&ctx), Some(3));

    // Call Rust functions from JS
    let sub = ctx.new_function(|ctx, this, args| {
        if args.len() != 2 {
            ctx.throw(ctx.new_string("invalid number of arguments"))?;
        }
        let a = args[0].to_i32(ctx).unwrap();
        let b = args[1].to_i32(ctx).unwrap();
        Ok(ctx.new_i32(a - b))
    })?;
    ctx.globals().set("sub", sub);

    let result = ctx.eval("sub(1, 2)");
    assert_eq!(result.to_i32(&ctx), Some(-1));

    Ok(())
}
```

## Constraints

- User-defined classes are not supported.
  - In MicroQuickJS, user-defined functions and classes must be known at compile time. Implementing this via FFI is difficult.
  - Support for functions is achieved by adding extensions to the MicroQuickJS source code.
- The following standard library functions are not implemented. They return `undefined` when called:
  - `print()`, `gc()`, `Date.now()`, `performance.now()`, `setTimeout()`, `clearTimeout()`

## License

This library is provided under the [MIT License](LICENSE).