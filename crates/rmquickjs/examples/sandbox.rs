use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let ctx = rmquickjs::Context::new();
    let func_add = ctx.new_function(|ctx, _, args| {
        let a = args[0].to_i32(ctx).unwrap();
        let b = args[1].to_i32(ctx).unwrap();
        Ok(ctx.new_i32(a + b))
    })?;

    ctx.globals().set("add", func_add);

    let ret = ctx.eval(r#"add(1, 2)"#)?;
    println!("{}", ret.to_string(&ctx));
    Ok(())
}
