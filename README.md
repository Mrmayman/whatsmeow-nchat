# `whatsmeow-nchat`

Custom Rust bindings to the `whatsmeow` Go library,
based on [nchat's](https://github.com/d99kris/nchat/) code.

This uses many custom modifications to `whatsmeow`'s API and implementation,
intended for use in a chat client, and may not give you the full
default experience.

You can interact with the raw C bindings in `whatsmeow-nchat-sys`
or the safe-ish Rust API in `whatsmeow-nchat`.

> **NOTE**: This is very incomplete.
> Don't rely on this unless you know what you're doing!

# Dependencies

- [Go compiler](https://go.dev/dl/)
- GCC/Clang (MinGW/MSYS2 if on Windows)
- `libclang` (optional)
- `x86_64-pc-windows-gnu` target if on Windows (MSVC unsupported!)
