use colored::Colorize;

#[unsafe(no_mangle)]
pub extern "C" fn print_u64(x: u64) {
    let msg = format!("{}", x).green();
    println!("{}", msg);
}
#[unsafe(no_mangle)]
pub extern "C" fn print_i64(x: i64) {
    let msg = format!("{}", x).green();
    println!("{}", msg);
}
#[unsafe(no_mangle)]
pub extern "C" fn print_f64(x: f64) {
    let msg = format!("{}", x).green();
    println!("{}", msg);
}
#[unsafe(no_mangle)]
pub extern "C" fn print_str(ptr: *const u8, len: i64) {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };

    let s = std::str::from_utf8(slice).unwrap().green();
    println!("{}", s);
}
