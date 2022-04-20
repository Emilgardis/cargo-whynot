# Why Not?

Cargo subcommand to find out why a function is unsafe.


## What is unsafety?

https://doc.rust-lang.org/nomicon/what-unsafe-does.html


    Dereference raw pointers
    Call unsafe functions (including C functions, compiler intrinsics, and the raw allocator)
    Implement unsafe traits
    Mutate statics
    Access fields of unions
