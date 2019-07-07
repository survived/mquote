extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate proc_macro_hack;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;

use parse::{parse_spanned, parse_unspanned};
use compile::compile;

mod buffer;
mod error;
mod parse;
mod language;
mod compile;

#[proc_macro_hack]
pub fn mquote(input: TokenStream) -> TokenStream {
    match parse_unspanned(input.into()) {
        Ok(qtokens) => compile(qtokens, None).into(),
        Err(error) => error.raise().into(),
    }
}

#[proc_macro_hack]
pub fn mquote_spanned(input: TokenStream) -> TokenStream {
    match parse_spanned(input.into()) {
        Ok((requested_span, qtokens)) => compile(qtokens, Some(requested_span)).into(),
        Err(error) => error.raise().into(),
    }
}
