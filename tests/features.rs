use mquote::mquote;

#[test]
fn if_branches() {
    let cond = true;
    let a = 6i32;
    let q = mquote!(1i32 + #{if cond} #{a} #{else} 2i32 #{endif} + 3i32);
    assert_eq!(q.to_string(), "1i32 + 6i32 + 3i32");

    let cond = false;
    let q = mquote!(1i32 + #{if cond} #{a} #{else} 2i32 #{endif} + 3i32);
    assert_eq!(q.to_string(), "1i32 + 2i32 + 3i32");

    let (a, b, c) = (1, 2, 3);
    let n = 5;
    let q = mquote!(
        #{if n > 6}
            #{a}
        #{elif n > 3}
            #{b}
        #{else}
            #{c}
        #{endif}
    );
    assert_eq!(q.to_string(), "2i32");
}

#[test]
fn escaping() {
    let q = mquote!(#{{123}});
    assert_eq!(q.to_string(), "# { 123 }");
}

#[test]
fn insertion_via_expression() {
    let pair = (1, "abc");
    let q = mquote!(#{pair.0} #{pair.1});
    assert_eq!(q.to_string(), "1i32 \"abc\"");
}

#[test]
fn using_same_variable_twice() {
    let str: String = "123".into();
    let q = mquote!(#{str} #{str});
    assert_eq!(q.to_string(), "\"123\" \"123\"");
}

#[test]
fn nesting_if() {
    let q = mquote!{{
        #{if 2 + 2 == 4}
            123
        #{endif}
    }};
    assert_eq!(q.to_string(), "{ 123 }");
}
