//! This is Rust quasi-quoting library like [quote](quote) that gives you `mquote!` macro providing
//! several features aimed on better readability and usability. It supports:
//! * [expression insertion](#expression-insertion)
//! * [**if/else**](#if--elif--else) condition
//! * [**for**](#for) iteration
//! * [**match**](#matching)ing
//! * [**extend**](#extending)ing
//! 
//! ## Example
//! ```rust
//! # use mquote::mquote;
//! # let having_fun = true;
//! mquote!{
//!     #{if having_fun}
//!         fn funny_method() { ... }
//!     #{endif}
//!     fn regular_method() { ... }
//! };
//! ```
//! 
//! Crate contains `mquote!` and `mquote_spanned!`. Usage examples of the first one
//! are [below](#expression-insertion). The second one allow you to set
//! span of producing tokens stream by this syntax: `mquote_spanned!(span => ...)`.
//! 
//! ## Expression insertion
//! Turns given expression into tokens by using [`ToTokens`](../quote/trait.ToTokens.html).
//! ```rust
//! # use mquote::mquote;
//! # struct Person { name: &'static str, age: u16 }
//! fn put_filter(enabled: bool) ->  proc_macro2::TokenStream {
//!     let good_person = Person{ name: "Oleg", age: 20 };
//!     mquote!{
//!         assert!(!#{enabled} || person.name == #{good_person.name} 
//!             && person.age >= #{good_person.age})
//!     } 
//! }
//! ```
//! 
//! ## If / elif / else
//! ```rust
//! # use mquote::mquote;
//! fn define_container(amount: usize) ->  proc_macro2::TokenStream {
//!     mquote!{
//!         #{if amount > 1}
//!             struct People(Vec<Person>);
//!         #{elif amount == 1}
//!             struct Human(Person);
//!         #{else}
//!             struct NoneHuman;
//!         #{endif}
//!     }
//! }
//! ```
//! 
//! ## For
//! ```rust
//! # use mquote::mquote;
//! # use proc_macro2::Ident;
//! fn define_person(fields: Vec<(Ident, Ident)>) -> proc_macro2::TokenStream {
//!     mquote!{
//!         pub struct Person {
//!             #{for (name, ty) in fields}
//!                 #{name}: #{ty}
//!             #{endfor}
//!         }
//!     }
//! }
//! ```
//! 
//! ## Matching
//! ```rust
//! # use mquote::mquote;
//! # use proc_macro2::Ident;
//! fn hardcode_it(var: Ident, value: Option<&str>) -> proc_macro2::TokenStream {
//!     mquote!{
//!         static #var: &str = #{match value}
//!             #{of Some(x) if x.len() > 0}
//!                 #{x};
//!             #{of Some(_)}
//!                 "case for empty strings";
//!             #{of None}
//!                 "default value";
//!         #{endmatch}
//!     }
//! }
//! ```
//! 
//! ## Extending
//! Sometimes you want `mquote!` to consume an iterator of `TokenTree`s
//! without cloning. It's possible with special syntax `^{iterable}` that accepts
//! any `IntoIterator<Item=TokenTree>`.
//! 
//! ```rust
//! # use mquote::mquote;
//! # use proc_macro2::TokenStream;
//! fn assign_by_ref(stream: TokenStream) -> TokenStream {
//!     let tail = stream.into_iter().skip(5); //  here could be something
//!                                            //  more reasonable
//!     mquote!{
//!         let _ = ^{tail}
//!     }
//! }
//! ```
//! 
//! ## Escaping `#{}` or `^{}`
//! If you want to put either `#{abc}` or `^{abc}` as is, you should double braces:
//! ```rust
//! # use mquote::mquote;
//! fn it_works() {
//!     let tokens = mquote!(#{{abc}} ^{{abc}});
//!     assert_eq!(tokens.to_string(), "# { abc } ^ { abc }")
//! }
//! ```

extern crate mquote_impl;
extern crate proc_macro_hack;
extern crate quote;
extern crate proc_macro2;

use proc_macro_hack::proc_macro_hack;

/// Turns given directives into [TokenStream](../proc_macro2/struct.TokenStream.html).
///
/// You may learn syntax in [root level documentation](index.html).
#[proc_macro_hack]
pub use mquote_impl::mquote;

/// Same as [mquote!](macro.mquote.html), but applies a given span to all tokens originating within the macro invocation.
///
/// ## Example
/// ```rust
/// # use mquote::mquote_spanned;
/// let span = proc_macro2::Span::call_site();
/// mquote_spanned!(span => let _ = Some(12));
/// ```
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
    pub mod std {
        pub use ::std::*;
    }
}

#[cfg(mquote_span_testing)]
#[doc(hidden)]
#[path = "mocked_runtime/mod.rs"]
pub mod __rt;
