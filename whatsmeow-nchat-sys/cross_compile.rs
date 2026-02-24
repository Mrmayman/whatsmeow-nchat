use std::env::{self, consts};

pub fn get_target_os() -> String {
    env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| consts::OS.to_owned())
}

fn get_target_arch() -> String {
    env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| consts::ARCH.to_owned())
}

pub fn get_target_env() -> String {
    env::var("CARGO_CFG_TARGET_ENV").unwrap()
}

fn is_target_little_endian() -> bool {
    if let Ok(e) = env::var("CARGO_CFG_TARGET_ENDIAN") {
        e == "little"
    } else {
        cfg!(target_endian = "little")
    }
}

pub fn get_cross_compilation_details() -> (String, String) {
    let target_os = get_target_os();
    let target_os = if target_os == "macos" {
        "darwin".to_owned()
    } else {
        target_os
    };

    let target_arch = get_target_arch();
    let target_arch = if target_arch == "x86_64" {
        "amd64".to_owned()
    } else if target_arch == "i586" || target_arch == "i686" {
        "386".to_owned()
    } else if target_arch == "aarch64" {
        "arm64".to_owned()
    } else if target_arch == "loongarch64" {
        "loong64".to_owned()
    } else if target_arch == "powerpc64" {
        if is_target_little_endian() {
            "ppc64le".to_owned()
        } else {
            "ppc64".to_owned()
        }
    } else {
        target_arch
    };
    (target_os, target_arch)
}
