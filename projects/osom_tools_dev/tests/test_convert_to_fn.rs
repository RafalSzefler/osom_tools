use osom_tools_dev::{macros, traits};

#[test]
fn test_convert_to_fn() {
    struct Test;
    unsafe impl traits::Pointerable for Test {
        unsafe fn as_ptr(&self) -> *mut u8 {
            self as *const Test as *mut u8
        }
    }

    let x = Test;

    let _ = macros::convert_to_fn!("C", x);
    let _ = macros::convert_to_fn!("C", x, fn() -> i64);
    let _ = macros::convert_to_fn_with_offset!("C", x, 25);
    let _ = macros::convert_to_fn_with_offset!("C", x, 0, fn(bool, &str) -> i64);
}
