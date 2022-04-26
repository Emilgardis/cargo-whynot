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
