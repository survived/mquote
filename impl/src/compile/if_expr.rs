use proc_macro2::{TokenStream, Ident};
use proc_quote::quote;

use crate::language::*;
use super::IdentGenerator;

pub(super) fn compile(if_expr: MQuoteIf, gen: &mut IdentGenerator) -> TokenStream {
    let condition = if_expr.condition;
    let then = super::compile_with(*if_expr.then, gen);
    let else_ = match if_expr.else_ {
        Some(else_) => super::compile_with(*else_, gen),
        None => TokenStream::new(),
    };
    quote!{{
        if #condition { #then } else { #else_ }
    }}
}