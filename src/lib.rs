extern crate mquote_impl;
extern crate proc_macro_hack;
extern crate quote;
extern crate proc_macro2;

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use mquote_impl::mquote;

#[proc_macro_hack]
pub use mquote_impl::mquote_spanned;

#[cfg(not(mquote_span_testing))]
#[doc(hidden)]
pub mod __rt {
    pub mod quote {
        pub use crate::quote::*;
    }
    pub mod proc_macro2 {
        pub use crate::proc_macro2::*;
    }
}

#[cfg(mquote_span_testing)]
#[doc(hidden)]
#[path = "mocked_runtime/mod.rs"]
pub mod __rt;
