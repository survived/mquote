/// mquote_span_testing is a special mode of crate that breaks everything but allows
/// detecting span cases (targeted on finding out problems in mquote_spanned)
///
/// To correctly run this test you should type:
/// ```bash
/// $ RUSTFLAGS='--cfg mquote_span_testing' cargo test --test spanned
/// ```

use mquote::mquote_spanned;

#[cfg(not(mquote_span_testing))]
extern crate proc_macro2;
#[cfg(mquote_span_testing)]
use mquote::__rt::proc_macro2::{TokenStream, TokenTree, Span, Ident};


#[cfg(not(mquote_span_testing))]
#[test]
fn spanned_simplest() {
    let span = proc_macro2::Span::call_site();
    let q = mquote_spanned!{span => #{if true} "abc" #{else} 123 #{endif}};
    assert_eq!(q.to_string(), "\"abc\"");
}

#[cfg(mquote_span_testing)]
#[test]
fn spanned_term() {
    // The term all following tests depend on. This is how we actually can ensure of correctness
    // of the span stuff.
    let call_site = Span::call_site();
    let different_span = Span::different();

    assert!(!call_site.eq(&different_span))
}

#[cfg(mquote_span_testing)]
fn assert_span_of_tokens(token_stream: TokenStream, should_be: Span) {
    token_stream.into_iter().for_each(|token| assert_span_of_token(token, should_be))
}

#[cfg(mquote_span_testing)]
fn assert_span_of_token(token: TokenTree, should_be: Span) {
    match token {
        TokenTree::Group(group) => {
            assert!(group.span().eq(&should_be));
            group.stream().into_iter()
                .for_each(|token| assert_span_of_token(token, should_be))
        }
        token => assert!(token.span().eq(&should_be)),
    }
}

#[cfg(mquote_span_testing)]
#[test]
fn spanned_insertion() {
    let span = Span::different();
    let ident = Ident::new("abc", span);
    let q = mquote_spanned!(span => #{ident});
    assert_span_of_tokens(q, span);
}

#[cfg(mquote_span_testing)]
#[test]
fn spanned_if() {
    let span = Span::different();
    let a = 2;
    let q = mquote_spanned!(span => #{if a > 1} 1 #{elif a == 0} 0 #{else} -1 #{endif});
    assert_span_of_tokens(q, span);
}

#[cfg(mquote_span_testing)]
#[test]
fn spanned_for() {
    use std::iter;
    let span = Span::different();
    let iterable = iter::repeat(Ident::new("test", span)).take(10);
    let q = mquote_spanned!(span => #{for i in iterable} #{i} #{endfor});
    assert_span_of_tokens(q, span);
}

#[cfg(mquote_span_testing)]
#[test]
fn spanned_match() {
    let span = Span::different();
    let e = Some(12);
    let ident = Ident::new("abc", span);
    let q = mquote_spanned!(span =>
        #{match e}
            #{of Some(_)}
                #{ident}
            #{of None}
                ident2
        #{endmatch}
    );
    assert_span_of_tokens(q, span);
}

#[cfg(mquote_span_testing)]
#[test]
fn spanned_escaping() {
    let span = Span::different();
    let q = mquote_spanned!(span => #{{ident}});
    assert_span_of_tokens(q, span);
}

#[cfg(mquote_span_testing)]
#[test]
fn spanned_nested() {
    let span = Span::different();
    let ident = Ident::new("abc", span);
    let q = mquote_spanned!(span => {
        #{if 2 + 2 == 5}
            123
        #{else}
            { let b = #{ident}; b * 2 }
        #{endif}
    });
    assert_span_of_tokens(q, span);
}

#[cfg(mquote_span_testing)]
#[test]
fn complex_case() {
    let span = Span::different();
    let enum_name = Ident::new("MyEnum", span);
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
        .map(|v| Ident::new(v, span))
        .collect::<Vec<_>>();
    let impl_default = Some(2);
    let q = mquote_spanned!{span =>
        pub enum #{enum_name} {
            #{for variant in &variants}
                #{variant},
            #{endfor}
        }

        impl #{enum_name} {
            fn as_str(&self) -> &'static str {
                match self {
                    #{for (variant, variant_str) in variants.iter().zip(variants_str.iter())}
                        #{enum_name}::#{variant} => unimplemented!(),
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

    assert_span_of_tokens(q, span);
}
