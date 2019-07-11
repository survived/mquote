extern crate trybuild;

#[test]
fn cases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail_cases/*.rs");
}
