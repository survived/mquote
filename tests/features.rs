#[macro_use]
extern crate quote;

use mquote::mquote;

#[test]
fn does_nothing() {
    let a = 2;
    let q = mquote!(a);
    assert_eq!(q.to_string(), quote!(1 + 2i32 + 3).to_string());
}
