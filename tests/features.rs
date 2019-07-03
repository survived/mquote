extern crate quote;
extern crate proc_macro2;

use mquote::mquote;

#[test]
fn does_nothing() {
    let b = 6;
    let q = mquote!(true b);
    assert_eq!(q.to_string(), "1i32 + 6i32 + 3i32");

    let q = mquote!(false b);
    assert_eq!(q.to_string(), "1i32 + 2i32 + 3i32");
}
