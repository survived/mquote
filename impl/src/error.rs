use proc_macro2::{TokenStream, Span};
use quote::quote_spanned;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    span_s: Span,
    span_e: Option<Span>,
    msg: String,
}

impl Error {
    /// Creates a new Error at the given span with the given
    /// message.
    pub fn new<E: Into<String>>(span: Span, msg: E) -> Self {
        Self {
            span_s: span,
            span_e: None,
            msg: msg.into(),
        }
    }

    /// Sets the end of the span of this error. The start of
    /// the span is set in `Error::new`.
    pub fn end_span(mut self, span: Span) -> Self {
        self.span_e = Some(span);
        self
    }

    /// Raises into a `TokenStream` that calls `compile_error!`
    /// with the given message, at the given span.
    pub fn raise(self) -> TokenStream {
        let Error {
            span_s,
            span_e,
            msg,
        } = self;

        let compile_error = quote_spanned! { span_s=>
            compile_error!(#msg)
        };

        quote_spanned! { span_e.unwrap_or(span_s)=>
            #compile_error ;
        }
    }
}
