#![allow(clippy::missing_safety_doc)]
pub use unsafe_mod::unsafety;

pub unsafe fn internal() {
    let a = unsafety();

    eprintln!("a: {}", a);
}

pub fn external() {
    let a = unsafe { String::from_utf8_unchecked(b"Hello world!".to_vec()) };
    eprintln!("a: {}", a);
}

pub fn safe() {
    println!("eyy")
}

pub struct Hello {}
impl Hello {
    pub fn world(&self) {
        unsafe { self.unsf() }
    }

    pub unsafe fn unsf(&self) {}
    pub unsafe fn unsaf(&self) {}
}

pub mod unsafe_mod {
    pub unsafe fn unsafety() -> u32 {
        let mut a = 1;
        let a = std::ptr::addr_of_mut!(a);
        // this is the unsafe part
        let b = *a;
        // this is also unsafe
        let _ = String::from_utf8_unchecked(b"Hello world!".to_vec());
        unsafety_not_really();
        b
    }
    pub unsafe fn unsafety_not_really() {}
}
