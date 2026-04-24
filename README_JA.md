# rmquickjs

High-level MicroQuickJS bindings for Rust

[![Crates.io version](https://img.shields.io/crates/v/rmquickjs.svg?style=flat-square)](https://crates.io/crates/rmquickjs)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rmquickjs)

rmquickjsはQuickJSの作者による組み込み向けのJSランタイム[MicroQuickJS](https://github.com/bellard/mquickjs)の高レベルなAPIを提供するライブラリです。

このライブラリは既存のRustバインディングである[mquickjs-rs](https://github.com/fcoury/mquickjs-rs)にインスパイアされていますが、rmquickjsはより多くの機能と人間工学的なAPIを備えてます。

## 特徴

- MicroQuickJSのC APIに対応した高レベルAPI
- Rust ↔ JS間の関数呼び出しに対応
- no_std対応

## インストール

```bash
$ cargo add rmquickjs
```

## クイックスタート

```rs
use rmquickjs::*;

fn main() -> Result<()> {
    // MicroQuickJSエンジンの初期化
    let ctx = Context::new();
    
    // eval()でJSを実行
    let result = ctx.eval("1 + 2")?;
    assert_eq!(result.to_i32(&ctx), Some(3));

    // globals()でグローバル変数へアクセス
    ctx.eval("var x = 'hello'")?;
    let x = ctx.globals().get("x")?;
    assert_eq!(x.to_string(&ctx), "hello".to_string());

    // JSの関数をRustから呼び出す
    ctx.eval(
        r#"
function add(x, y) {
    return x + y;
}"#,
    )?;

    let add = ctx.globals().get("add")?.to_function(&ctx).unwrap();

    let result = add.call(&[ctx.new_i32(1), ctx.new_i32(2)])?;
    assert_eq!(result.to_i32(&ctx), Some(3));

    // Rustの関数をJSから呼び出す
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

## 制約

- ユーザー定義のクラスには対応していません
  - MicroQuickJSではユーザー定義の関数・クラスがコンパイル時に既知である必要があります。これをFFIで実現するのは困難です
  - 関数についてはMicroQuickJSのソースコードに拡張を追加することで実現しています
- 以下の標準ライブラリ関数は未実装です。呼び出しに対して`undefined`を返します
  - `print()`, `gc()`, `Date.now()`, `performance.now()`, `setTimeout()`, `clearTimeout()`

## ライセンス

このライブラリは[MIT License](LICENSE)の下で提供されています。