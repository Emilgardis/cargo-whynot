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


## Examples WIP

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

would report something like

```
`foo` is marked unsafe due to calling `unsafe_mod::unsafety` which dereferences a raw pointer.
```