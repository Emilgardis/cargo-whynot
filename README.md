# Why Not?

Cargo subcommand to discover why a function is unsafe.

Requires a recent enough nightly rust toolchain.

`cargo whynot safe foo`

## What is unsafety?

[Nomicon definition](https://doc.rust-lang.org/nomicon/what-unsafe-does.html)

  Dereference raw pointers
  Call unsafe functions (including C functions, compiler intrinsics, and the raw allocator)
  Implement unsafe traits
  Mutate statics
  Access fields of unions

## Why this tool?

Because it's a fun experiment, hooking into rustc to query the drivers.
You should not use this tool because unsafe code is generally bad (it's not),
but you can use it to figure out if there is an opportunity to make a function "safe".

## How to install

1. Ensure the `rustc-dev` component is installed for your nightly toolchain

   ```text
   rustup component add rustc-dev --toolchain nightly
   ```

2. `cargo +nightly install cargo-whynot`

## How does it work?

  1. Call `cargo check` with `env:RUSTC_WORKSPACE_WRAPPER` set to this binary.

## It doesn't work!

### Can't find rustc_driver

If you get a non-zero exit code, and no output, make sure that your rust sysroot is visible.

On Linux:

```
export LD_LIBRARY_PATH=$(rustc --print sysroot)/lib:$LD_LIBRARY_PATH
```

On Windows:

```
$env:LD_LIBRARY_PATH="$(rustc --print sysroot)/lib:$($env:LD_LIBRARY_PATH)"
```

On MacOS:

```
export DYLD_LIBRARY_PATH=$(rustc --print sysroot)/lib:$DYLD_LIBRARY_PATH
```

## Examples

With the following code

```rust
pub use unsafe_mod::unsafety;

pub unsafe fn foo() {
    let a = unsafety();
    eprintln!("a: {}", a);
}

pub mod unsafe_mod {
    pub unsafe fn unsafety() -> u32 {
        let mut a = 1;
        let a = std::ptr::addr_of_mut!(a);
        // this is the unsafe part
        let b = *a;
        b
    }
}
```

`cargo whynot safe foo`

will report

```
note: Function is unsafe
  ┌─ src/lib.rs:3:1
  │
3 │ pub unsafe fn foo() {
  │ ^^^^^^^^^^^^^^^^^^^ function is unsafe because:
4 │     let a = unsafety();
  │             ---------- call to unsafe function `unsafe_mod::unsafety`

help:
   ┌─ src/lib.rs:9:5
   │
 9 │     pub unsafe fn unsafety() -> u32 {
   │     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ function is unsafe because:
   ·
13 │         let b = *a;
   │                 ^^ dereference of raw pointer
   │
   = this function does a fundamentally unsafe operation
```


<h5> License </h5>

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

