extern crate proc_macro2;

use proc_macro2::{Ident, Span};
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
fn for_expr() {
    let idents = vec![
        Ident::new("ident1", Span::call_site()),
        Ident::new("ident2", Span::call_site()),
        Ident::new("ident3", Span::call_site()),
        Ident::new("ident4", Span::call_site()),
    ];

    let q = mquote!(
        #{for ident in idents}
            #{ident}
        #{endfor}
    );
    assert_eq!(q.to_string(), "ident1 ident2 ident3 ident4");
}

#[test]
fn match_expr() {
    let a = Some("abc");
    let q = mquote!(
        #{match &a}
            #{of Some(s) if s.is_empty()}
                "empty case"
            #{of Some(s)}
                #{s}
            #{of None}
                "default value"
        #{endmatch}
    );
    assert_eq!(q.to_string(), "\"abc\"");
}

#[test]
fn escaping() {
    let q = mquote!(#{{123}});
    assert_eq!(q.to_string(), "# { 123 }");
}

#[test]
fn insertion_via_expression() {
    let pair = (1, "abc");
    let foo = |i: i64| i * 2;
    let q = mquote!(#{pair.0} #{pair.1} #{foo(12)});
    assert_eq!(q.to_string(), "1i32 \"abc\" 24i64");
}

#[test]
fn insertion_via_variable_with_corresponding_name_to_used_in_expanded_macro() {
    let test_str = "test_str";
    let (i, p, s, inner_stream, insertion, token_stream) = (test_str,test_str,test_str,test_str,test_str,test_str);
    let q = mquote!(
        #{i} #{p} #{s} #{inner_stream} #{insertion} #{token_stream}
    );
    assert_eq!(q.to_string(), "\"test_str\" \"test_str\" \"test_str\" \"test_str\" \"test_str\" \"test_str\"");
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
            { 123 }
        #{endif}
    }};
    assert_eq!(q.to_string(), "{ { 123 } }");
}

#[test]
fn extending_insertion() {
    let insertion = mquote!(h e l l o);
    let q = mquote!(let _ = [ ^{insertion} ]);
    assert_eq!(q.to_string(), "let _ = [ h e l l o ]");
}

#[test]
fn complex_case() {
    let enum_name = Ident::new("MyEnum", Span::call_site());
    let variants_str = vec![
        "Insertion",
        "Extend",
        "If",
        "For",
        "Match",
        "Group",
        "Plain",
    ];
    let variants = variants_str.iter()
        .map(|v| Ident::new(v, Span::call_site()))
        .collect::<Vec<_>>();
    let impl_default = Some(2);
    let q = mquote!{
        pub enum #{enum_name} {
            #{for variant in &variants}
                #{variant},
            #{endfor}
        }

        impl #{enum_name} {
            fn as_str(&self) -> &'static str {
                match self {
                    #{for (variant, variant_str) in variants.iter().zip(variants_str.iter())}
                        #{enum_name}::#{variant} => #{variant_str},
                    #{endfor}
                }
            }
        }

        #{match impl_default}
        #{of None}
        #{of Some(n)}
            impl Default for #{enum_name} {
                fn default() -> Self {
                    #{enum_name}::#{variants[n]}
                }
            }
        #{endmatch}
    };
    let expected =
        "pub enum MyEnum { \
            Insertion , \
            Extend , \
            If , \
            For , \
            Match , \
            Group , \
            Plain , \
        } \
        \
        impl MyEnum { \
            fn as_str ( & self ) -> & 'static str { \
                match self { \
                    MyEnum ::Insertion => \"Insertion\" , \
                    MyEnum ::Extend => \"Extend\" , \
                    MyEnum ::If => \"If\" , \
                    MyEnum ::For => \"For\" , \
                    MyEnum ::Match => \"Match\" , \
                    MyEnum ::Group => \"Group\" , \
                    MyEnum ::Plain => \"Plain\" , \
                } \
            } \
        } \
        \
        impl Default for MyEnum { \
            fn default ( ) -> Self { \
                MyEnum ::If \
            } \
        }";
    assert_eq!(q.to_string(), expected);
}
