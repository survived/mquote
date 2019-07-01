#[macro_use]
extern crate quote;

use mquote::mquote;

#[test]
fn does_nothing() {
    let q = mquote!(true);
//    assert_eq!(q.to_string(), quote!(1 + 2 + 3).to_string());

    let q = mquote!(false);
//    assert_eq!(q.to_string(), quote!(1 + 3 + 3).to_string());
}
