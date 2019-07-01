extern crate mquote_impl;
extern crate proc_macro_hack;

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use mquote_impl::mquote;
