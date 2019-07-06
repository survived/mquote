extern crate proc_macro;
extern crate proc_macro2;
extern crate proc_quote;
extern crate proc_macro_hack;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;

use parse::parse;
use compile::compile;

mod buffer;
mod error;
mod parse;
mod language;
mod compile;

#[proc_macro_hack]
pub fn mquote(input: TokenStream) -> TokenStream {
    match parse(input.into()) {
        Ok(qtokens) => compile(qtokens).into(),
        Err(error) => error.raise().into(),
    }
}
