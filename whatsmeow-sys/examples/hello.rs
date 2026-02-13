fn main() {
    unsafe {
        let r = whatsmeow_sys::CWmInit(c"./prof".as_ptr() as *mut _, c"".as_ptr() as *mut _, 1);
        println!("{r}")
    }
}
