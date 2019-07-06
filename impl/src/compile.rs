use proc_macro2::*;
use proc_quote::{TokenStreamExt, quote, quote_spanned};

use crate::language::*;
use crate::buffer::QTokens;

pub fn compile(mquote: QTokens) -> TokenStream {
    let mut stream = TokenStream::new();
    let ref token_stream = Ident::new("__token_stream", Span::call_site());
    compile_with(mquote, &mut stream, token_stream);
    quote!{{
        let mut #token_stream = ::proc_macro2::TokenStream::new();
        #stream
        #token_stream
    }}
}

fn compile_with<S>(mquote: S, stream: &mut TokenStream, token_stream: &Ident)
where S: IntoIterator<Item=TokenTreeQ>
{
    mquote.into_iter()
        .for_each(|token| process_token(token, stream, token_stream))
}

fn process_token(token: TokenTreeQ, stream: &mut TokenStream, token_stream: &Ident) {
    match token {
        TokenTreeQ::Plain(token) => put_token(token, stream, token_stream),
        TokenTreeQ::Insertion(tokens) => {
            assert_ne!(token_stream.to_string(), "__insertion");
            stream.append_all(quote!({
                let ref __insertation = #tokens;
                ::quote::ToTokens::to_tokens(__insertation, &mut #token_stream);
            }));
        },
        TokenTreeQ::Group(MQuoteGroup{ delimiter, tokens: group_tokens, span }) => {
            let mut inner_stream = TokenStream::new();
            compile_with(group_tokens, &mut inner_stream, token_stream);

            let mut group = proc_macro2::Group::new(delimiter, inner_stream);
            group.set_span(span);

            put_token(group.into(), stream, token_stream);
        }
        TokenTreeQ::If(MQuoteIf{ condition, then, else_}) => {
            let mut then_stream = TokenStream::new();
            compile_with(then, &mut then_stream, token_stream);
            let then_branch = Group::new(Delimiter::Brace, then_stream);

            let mut else_stream = TokenStream::new();
            if let Some(else_) = else_ {
                compile_with(else_, &mut else_stream, token_stream);
            }
            let else_branch = Group::new(Delimiter::Brace, else_stream);

            stream.append_all(quote!( if #condition #then_branch else #else_branch))
        }
    }
}

fn put_token(token: TokenTree, stream: &mut TokenStream, token_stream: &Ident) {
    let requested_span = quote!(::proc_macro2::Span::call_site());
    match token {
        TokenTree::Literal(lit) => {
            let span = lit.span();
            let stringed_lit = lit.to_string();

            stream.append_all(quote_spanned!(span => {
                let s: ::proc_macro2::TokenStream = #stringed_lit.parse().expect("invalid token stream");
                #token_stream.extend(s.into_iter().map(|mut t| {
                    t.set_span(#requested_span);
                    t
                }))
            }));
        }
        TokenTree::Punct(punct) => {
            let op = punct.as_char();
            let span = punct.span();
            let spacing = match punct.spacing() {
                Spacing::Alone => Ident::new("Alone", span),
                Spacing::Joint => Ident::new("Joint", span),
            };
            stream.append_all(quote_spanned!(span => {
                let mut p = ::proc_macro2::Punct::new(#op, ::proc_macro2::Spacing::#spacing);
                p.set_span(#requested_span);
                #token_stream.extend(::std::iter::once(::proc_macro2::TokenTree::Punct(p)));
            }));
        }
        TokenTree::Ident(ident) => {
            let stringed_ident = ident.to_string();
            let span = ident.span();
            stream.append_all(quote_spanned!(span => {
                let i = ::proc_macro2::Ident::new(#stringed_ident, #requested_span);
                #token_stream.extend(::std::iter::once(::proc_macro2::TokenTree::Ident(i)));
            }));
        }
        TokenTree::Group(group) => {
            let span = group.span();
            let delimiter = match group.delimiter() {
                Delimiter::Brace       => Ident::new("Brace"      , span),
                Delimiter::Bracket     => Ident::new("Bracket"    , span),
                Delimiter::Parenthesis => Ident::new("Parenthesis", span),
                Delimiter::None        => Ident::new("None"       , span),
            };

            let inner_stream_var = Ident::new("inner_stream", span);
            let constructing_group_var = Ident::new("constructing_group", span);

            let mut inner_stream = TokenStream::new();
            inner_stream.append_all(quote_spanned!(span =>
                let mut #inner_stream_var = ::proc_macro2::TokenStream::new();
            ));
            compile_with(group.stream().into_iter().map(TokenTreeQ::Plain), &mut inner_stream, &inner_stream_var);
            inner_stream.append_all(quote_spanned!(span =>
                let mut #constructing_group_var = ::proc_macro2::Group::new(::proc_macro2::Delimiter::#delimiter, #inner_stream_var);
                #constructing_group_var.set_span(#requested_span);
                #token_stream.extend(::std::iter::once(::proc_macro2::TokenTree::Group(#constructing_group_var)))
            ));

            stream.append_all(quote_spanned!(span => { #inner_stream }));

        }
    }
}
