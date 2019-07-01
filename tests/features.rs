use mquote::mquote;

#[test]
fn does_nothing() {
    let q = mquote!(123 hehfhe df);
    assert_eq!(q, 3);
}
