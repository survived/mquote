extern crate proc_macro;
extern crate proc_macro2;
extern crate proc_quote;
extern crate proc_macro_hack;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use proc_quote::quote;

use crate::language::*;

mod language;
mod compile;

#[proc_macro_hack]
pub fn mquote(input: TokenStream) -> TokenStream {
    let a = 4;
    let mquote = MQuote::Binding(MQuoteBinding {
        start: quote!(1 + ),
        cons: vec![(BindWith::MQuote(Box::new(MQuote::If(MQuoteIf {
            condition: input.into(),
            then: Box::new(MQuote::Binding(MQuoteBinding { start: quote!( 2 ), cons: vec![] })),
            else_: Some(Box::new(MQuote::Binding(MQuoteBinding { start: quote!( 3 ), cons: vec![] }))),
        }))), quote!(+ 3))],
    });
    compile::compile(mquote).into()
}
