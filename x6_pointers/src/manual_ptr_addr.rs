fn main() {
    let a:i64 = 123;

    // create a raw pointer to a
    let a_ptr = &a as *const i64;

    // use transmute to manually read the value of a_ptr
    let a_addr: usize = unsafe { std::mem::transmute(a_ptr) };

    // here we can provide that :p and manual read of a_ptr give us the exact same value
    println!("{:p}, 0x{:x}", a_ptr, a_addr);
}
