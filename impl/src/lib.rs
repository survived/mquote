extern crate proc_macro;
extern crate proc_macro2;
extern crate proc_quote;
extern crate proc_macro_hack;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use proc_quote::quote;

mod intermediate;
mod convert;

#[proc_macro_hack]
pub fn mquote(input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        1 + 2
    })
}
