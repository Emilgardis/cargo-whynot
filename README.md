WIP

# Why Not?

Cargo subcommand to find out why a function is unsafe.

## What is unsafety?

https://doc.rust-lang.org/nomicon/what-unsafe-does.html


    Dereference raw pointers
    Call unsafe functions (including C functions, compiler intrinsics, and the raw allocator)
    Implement unsafe traits
    Mutate statics
    Access fields of unions


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
  ┌─ src\lib.rs:3:1
  │
3 │ pub unsafe fn foo() {
  │ ^^^^^^^^^^^^^^^^^^^ function is unsafe because:
4 │     let a = unsafety();
  │             ---------- call to unsafe function `unsafe_mod::unsafety`

help: 
   ┌─ src\lib.rs:10:5
   │
10 │     pub unsafe fn unsafety() -> u32 {
   │     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ function is unsafe because:
   ·
14 │         let b = *a;
   │                 ^^ dereference of raw pointer
```