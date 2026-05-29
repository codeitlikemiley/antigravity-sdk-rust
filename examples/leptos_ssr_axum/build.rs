use std::env;
fn main() {
    println!("cargo::rustc-check-cfg=cfg(runtime_spin)");
    println!("cargo::rustc-check-cfg=cfg(runtime_wasmtime)");
    let runtime = env::var("WASI_RUNTIME").unwrap_or_else(|_| "wasmtime".to_string());
    println!("cargo:rustc-env=WASI_RUNTIME={}", runtime);
    match runtime.as_str() {
        "spin" => {
            println!("cargo:rustc-cfg=runtime_spin");
        }
        "wasmtime" => {
            println!("cargo:rustc-cfg=runtime_wasmtime");
        }
        _ => {
            println!("cargo:rustc-cfg=runtime_wasmtime");
        }
    }
    if env::var("SPIN_BUILD").is_ok() {
        println!("cargo:rustc-cfg=runtime_spin");
    }
}
