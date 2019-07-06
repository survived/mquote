extern crate proc_macro;
extern crate proc_macro2;
extern crate proc_quote;
extern crate proc_macro_hack;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use proc_quote::quote;

use crate::language::*;
use crate::buffer::QTokens;

mod buffer;
//mod parse;
mod language;
mod compile;

#[proc_macro_hack]
pub fn mquote(input: TokenStream) -> TokenStream {
    use std::iter;
    use proc_macro2::*;
    let mut input = proc_macro2::TokenStream::from(input).into_iter();
    let a = input.next().unwrap();
    let b = input.next().unwrap();
    let stream: Vec<TokenTreeQ> = vec![
        TokenTreeQ::Plain(Literal::i32_suffixed(1).into()),
        TokenTreeQ::Plain(Punct::new('+', Spacing::Alone).into()),
        TokenTreeQ::If(MQuoteIf {
            condition: quote!(#a),
            then: QTokens::from(vec![
                TokenTreeQ::Insertion(iter::once(b).collect()),
            ]),
            else_: Some(QTokens::from(vec![
                TokenTreeQ::Plain(Literal::i32_suffixed(2).into()),
            ])),
        }),
        TokenTreeQ::Plain(Punct::new('+', Spacing::Alone).into()),
        TokenTreeQ::Plain(Literal::i32_suffixed(3).into()),
    ];
    compile::compile(QTokens::from(stream)).into()
}
