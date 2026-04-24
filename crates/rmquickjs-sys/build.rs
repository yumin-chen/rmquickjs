use std::path::Path;

fn exec_process(cmd: &mut std::process::Command) -> std::process::Output {
    let status = cmd.status().unwrap();
    let output = cmd.output().unwrap();
    assert!(
        status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr).to_string(),
    );

    output
}

fn main() {
    // env
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let host = std::env::var("HOST").unwrap();
    let mquickjs_src = Path::new("./mquickjs");

    // rerun-if-changed
    println!("cargo:rerun-if-changed=./wrapper.h");
    println!(
        "cargo:rerun-if-changed={}/rmquickjs.c",
        mquickjs_src.display()
    );
    println!(
        "cargo:rerun-if-changed={}/mqjs_stdlib.c",
        mquickjs_src.display()
    );

    // compile stdlib generator
    let exe_name = if host.contains("windows") {
        "stdlib_gen.exe"
    } else {
        "stdlib_gen"
    };
    let exe_path = Path::new(&out_dir).join(exe_name);
    let compiler = cc::Build::new()
        .include(&mquickjs_src)
        .warnings(false)
        .flag_if_supported("-std=c99")
        .flag_if_supported("-O2")
        .get_compiler();

    // phase 1: generate atom
    let mut cmd = compiler.to_command();
    cmd.args([
        &format!("{}/mquickjs_build.c", mquickjs_src.display()),
        &format!("{}/mqjs_stdlib.c", mquickjs_src.display()),
    ]);
    if compiler.is_like_msvc() {
        cmd.arg(format!("/Fe:{}", exe_path.display()));
    } else {
        cmd.arg("-o").arg(&exe_path);
    }
    exec_process(&mut cmd);

    let mut cmd = std::process::Command::new(&exe_path);
    cmd.arg("-a").current_dir(&mquickjs_src);
    let output = exec_process(&mut cmd);
    std::fs::write(
        &mquickjs_src.join("mquickjs_atom.h"),
        String::from_utf8_lossy(&output.stdout).to_string(),
    )
    .unwrap();

    // phase 2: generate stdlib
    let mut cmd = compiler.to_command();
    cmd.args([
        "-DRMQUICKJS",
        &format!("{}/rmquickjs.c", mquickjs_src.display()),
        &format!("{}/mquickjs_build.c", mquickjs_src.display()),
        &format!("{}/mqjs_stdlib.c", mquickjs_src.display()),
        &format!("{}/cutils.c", mquickjs_src.display()),
        &format!("{}/dtoa.c", mquickjs_src.display()),
        &format!("{}/libm.c", mquickjs_src.display()),
    ]);
    if compiler.is_like_msvc() {
        cmd.arg(format!("/Fe:{}", exe_path.display()));
    } else {
        cmd.arg("-o").arg(&exe_path);
    }
    exec_process(&mut cmd);

    let mut cmd = std::process::Command::new(&exe_path);
    let opt = if cfg!(target_pointer_width = "64") {
        "-m64"
    } else {
        "-m32"
    };
    cmd.arg(opt).current_dir(&mquickjs_src);
    let output = cmd.output().unwrap();
    std::fs::write(
        &mquickjs_src.join("mqjs_stdlib.h"),
        String::from_utf8_lossy(&output.stdout).to_string(),
    )
    .unwrap();

    // compile mquickjs
    cc::Build::new()
        .warnings(false)
        .files([
            &format!("{}/rmquickjs.c", mquickjs_src.display()),
            &format!("{}/rmqjs.c", mquickjs_src.display()),
            &format!("{}/mqjs_stdlib.c", mquickjs_src.display()),
            &format!("{}/cutils.c", mquickjs_src.display()),
            &format!("{}/dtoa.c", mquickjs_src.display()),
            &format!("{}/libm.c", mquickjs_src.display()),
        ])
        .include(&mquickjs_src)
        .compile("mquickjs");

    // generate bindings
    bindgen::Builder::default()
        .header("./wrapper.h")
        .allowlist_type("JS.*")
        .allowlist_function("JS_.*")
        .allowlist_var("JS_.*")
        .allowlist_var("JSW")
        .allowlist_var("js_stdlib")
        .use_core()
        .generate()
        .unwrap()
        .write_to_file("src/sys.rs")
        .unwrap();
}
