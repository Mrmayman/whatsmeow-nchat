use std::{env, path::PathBuf, process::Command};

fn main() {
    let go_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("go");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let is_linux = if let Ok(target_os) = target_os {
        target_os == "linux"
    } else {
        cfg!(target_os = "linux")
    };

    if !go_dir.join("libwhatsmeow.h").exists() {
        runcmd(
            Command::new("go")
                .args([
                    "build",
                    "-buildmode=c-archive",
                    "-modcacherw -buildvcs=false",
                    "-o",
                    "libwhatsmeow.a",
                ])
                .current_dir(&go_dir),
        );
    }

    let mut builder =
        bindgen::Builder::default().header(go_dir.join("libwhatsmeow.h").to_str().unwrap());
    if is_linux {
        builder = builder.clang_args(&[
            "-I/usr/include",
            "-I/usr/lib/clang/16/include",
            "-I/usr/lib/clang/17/include",
            "-I/usr/lib/clang/18/include",
            "-I/usr/lib/clang/19/include",
            "-I/usr/lib/clang/20/include",
            "-I/usr/lib/clang/21/include",
        ]);
    }
    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");

    println!("cargo:rustc-link-search=native={}", go_dir.display());
    println!("cargo:rustc-link-lib=static=whatsmeow");
    println!("cargo:rerun-if-changed={}/libwhatsmeow.a", go_dir.display());
    println!(
        "cargo:rustc-link-arg={}",
        go_dir.join("libwhatsmeow.a").display()
    );

    println!("cargo:rustc-link-lib=dylib=pthread");
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=dylib=rt");
}

fn runcmd(cmd: &mut Command) {
    let out = cmd.output().unwrap();
    if !out.status.success() {
        println!("{}", String::from_utf8_lossy(&out.stdout));
        eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        panic!()
    }
}
