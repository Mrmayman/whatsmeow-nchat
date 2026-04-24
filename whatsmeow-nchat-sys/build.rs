use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::cross_compile::{get_cross_compilation_details, get_target_env, get_target_os};

mod cross_compile;

fn main() {
    let go_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("go");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    run_go(&go_dir);

    generate_bindings(&go_dir, &out_path);

    configure_cargo(&go_dir);
}

fn generate_bindings(go_dir: &Path, out_path: &Path) {
    let bindings_path = out_path.join("bindings.rs");

    let res = std::panic::catch_unwind(|| {
        let mut builder =
            bindgen::Builder::default().header(go_dir.join("libwhatsmeow.h").to_str().unwrap());

        if get_target_os() == "linux" {
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
            .write_to_file(&bindings_path)
            .expect("Couldn't write bindings");
    });
    if let Some(err) = res.err() {
        if let Some(err) = err.downcast_ref::<String>() {
            if err.contains("LIBCLANG_PATH") {
                // Warning: if you ever update these bindings in the future update this file too.
                std::fs::copy(go_dir.join("bindings_cached.rs"), &bindings_path).unwrap();
                println!("Used fallback bindings");
                return;
            }
        }
        std::panic::panic_any(err)
    }
}

fn configure_cargo(go_dir: &Path) {
    let target_os = get_target_os();

    println!("cargo:rustc-link-search=native={}", go_dir.display());
    if target_os == "windows" && get_target_env() == "msvc" {
        println!("cargo:rustc-link-lib=static=libwhatsmeow");
    } else {
        println!("cargo:rustc-link-lib=static=whatsmeow");
    }

    if target_os == "windows" {
        // println!(
        //     "cargo:rustc-link-arg={}",
        //     go_dir.join("libwhatsmeow.lib").display()
        // );
    } else {
        println!(
            "cargo:rustc-link-arg={}",
            go_dir.join("libwhatsmeow.a").display()
        );
    }

    if target_os != "windows" && target_os != "macos" {
        println!("cargo:rustc-link-lib=dylib=pthread");
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=rt");
    }

    println!("cargo:rerun-if-changed={}/gowm.go", go_dir.display());
    println!("cargo:rerun-if-changed={}/cgowm.go", go_dir.display());
    println!("cargo:rerun-if-changed={}/ext", go_dir.display());
}

fn run_go(go_dir: &Path) {
    let is_go_present = Command::new("go").arg("version").output().is_ok();

    assert!(
        is_go_present,
        "The Go programming language's compiler (go) isn't installed or couldn't be found (not in PATH)\nGet from https://go.dev/doc/install or your package manager"
    );

    let (target_os, target_arch) = get_cross_compilation_details();

    #[allow(unused_mut)]
    let mut paths: Vec<_> = env::split_paths(&env::var_os("PATH").unwrap()).collect();
    #[cfg(target_os = "windows")]
    {
        paths.push(r"C:\msys64\mingw64\bin".into());
        paths.push(r"C:\msys64\mingw32\bin".into());
        paths.push(r"C:\mingw\bin".into());
    }
    let new_path = env::join_paths(paths).unwrap();

    run_go_cmd(
        &target_os,
        &target_arch,
        Command::new("go").current_dir(go_dir).args(["mod", "tidy"]),
    );

    run_go_cmd(
        &target_os,
        &target_arch,
        Command::new("go")
            .current_dir(go_dir)
            .arg("build")
            .args(["-buildmode=c-archive", "-modcacherw", "-buildvcs=false"])
            .args(["-ldflags", "-s -w"]) // Reduces file size from 60 mb -> _ mb but may worsen debuggability
            .args([
                "-o",
                if get_target_os() == "windows" && get_target_env() == "msvc" {
                    "libwhatsmeow.lib"
                } else {
                    "libwhatsmeow.a"
                },
            ])
            .env("PATH", &new_path),
    );
}

fn run_go_cmd(target_os: &str, target_arch: &str, cmd: &mut Command) {
    let mut envs = vec![
        ("GOOS", target_os.to_owned()),
        ("GOARCH", target_arch.to_owned()),
        ("CGO_ENABLED", "1".to_owned()),
    ];

    // Compiling *for* windows... but not *from* windows
    if target_os == "windows" && !cfg!(target_os = "windows") {
        let mingw_prefix = match target_arch {
            "amd64" => Some("x86_64"),
            "386" => Some("i686"),
            "arm64" => Some("aarch64"),
            _ => None,
        };

        if let Some(prefix) = mingw_prefix {
            envs.push(("CC", format!("{prefix}-w64-mingw32-gcc")));
            envs.push(("CXX", format!("{prefix}-w64-mingw32-g++")));
        } else {
            panic!("Unsupported Windows target architecture: {target_arch}");
        }
    }

    let out = cmd.envs(envs).output().unwrap();
    if !out.status.success() {
        println!("{}", String::from_utf8_lossy(&out.stdout));
        eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        panic!()
    }
}
