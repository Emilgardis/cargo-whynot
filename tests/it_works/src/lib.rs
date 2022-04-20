pub unsafe fn foo1() {
    let a = unsafety();
    eprintln!("a: {}", a);
}

pub fn foo2() {
    let a = unsafe { unsafety() };
    eprintln!("a: {}", a);
}

pub unsafe fn unsafety() -> u32 {
    let mut a = 1;
    let a = std::ptr::addr_of_mut!(a);
    // this is the unsafe part
    let b = *a;
    b
}
