use std::cell::RefCell;

use proc_macro2::*;
use quote::{ToTokens, TokenStreamExt, quote_spanned};

use crate::language::*;
use crate::buffer::QTokens;

struct Scope {
    token_stream_var: Ident,
    requested_span: TokenStream,
    runtime: TokenStream,
}

pub fn compile(mquote: QTokens, requested_span: Option<TokenStream>) -> TokenStream {
    let runtime = quote_spanned!(Span::call_site() => ::mquote::__rt);
    let scope = Scope {
        token_stream_var: Ident::new("token_stream", Span::call_site()),
        requested_span: requested_span.unwrap_or(quote_spanned!(Span::call_site() => #runtime::proc_macro2::Span::call_site())),
        runtime: runtime.clone(),
    };
    let mut stream = TokenStream::new();
    compile_with(mquote, &mut stream, &scope);

    let ref token_stream_var = scope.token_stream_var;
    quote_spanned!{Span::call_site() => {
        let mut #token_stream_var = #runtime::proc_macro2::TokenStream::new();
        #stream
        #token_stream_var
    }}
}

fn compile_with<S>(mquote: S, stream: &mut TokenStream, scope: &Scope)
where S: IntoIterator<Item=TokenTreeQ>
{
    mquote.into_iter()
        .for_each(|token| put_qtoken(token, stream, scope))
}

fn put_qtoken(token: TokenTreeQ, stream: &mut TokenStream, scope: &Scope) {
    let Scope{ ref runtime, ref requested_span, ref token_stream_var} = scope;
    match token {
        TokenTreeQ::Plain(token) => put_token(token, stream, scope),
        TokenTreeQ::Insertion(span, tokens) => {
            let tokens = ToTokensHack::from(tokens);
            let insertion_var = Ident::new("insertion", Span::call_site());
            stream.append_all(quote_spanned!(span => {
                let ref #insertion_var = #tokens;
                #runtime::quote::ToTokens::to_tokens(#insertion_var, &mut #token_stream_var);
            }));
        },
        TokenTreeQ::Group(MQuoteGroup{ delimiter, tokens: group_tokens, span }) => {
            let delimiter = match delimiter {
                Delimiter::Brace       => Ident::new("Brace"      , span),
                Delimiter::Bracket     => Ident::new("Bracket"    , span),
                Delimiter::Parenthesis => Ident::new("Parenthesis", span),
                Delimiter::None        => Ident::new("None"       , span),
            };

            let inner_scope = Scope {
                token_stream_var: Ident::new("inner_stream", Span::call_site()),
                requested_span: requested_span.clone(),
                runtime: runtime.clone(),
            };
            let ref inner_stream_var = inner_scope.token_stream_var;
            let constructing_group_var = Ident::new("constructing_group", Span::call_site());

            let mut inner_stream = TokenStream::new();
            compile_with(group_tokens, &mut inner_stream, &inner_scope);

            stream.append_all(quote_spanned!(span => {
                let mut #constructing_group_var = #runtime::proc_macro2::Group::new(#runtime::proc_macro2::Delimiter::#delimiter, {
                    let mut #inner_stream_var = #runtime::proc_macro2::TokenStream::new();
                    #inner_stream
                    #inner_stream_var
                });
                #constructing_group_var.set_span(#requested_span);
                #token_stream_var.extend(::std::iter::once(#runtime::proc_macro2::TokenTree::Group(#constructing_group_var)))
            }));
        }
        TokenTreeQ::If(MQuoteIf{ span, condition, then, else_}) => {
            let condition = ToTokensHack::from(condition);

            let mut then_stream = TokenStream::new();
            compile_with(then, &mut then_stream, scope);
            let mut then_branch = Group::new(Delimiter::Brace, then_stream);
            then_branch.set_span(span);

            let mut else_stream = TokenStream::new();
            if let Some(else_) = else_ {
                compile_with(else_, &mut else_stream, scope);
            }
            let mut else_branch = Group::new(Delimiter::Brace, else_stream);
            else_branch.set_span(span);

            stream.append_all(quote_spanned!( span => if #condition #then_branch else #else_branch ))
        }
        TokenTreeQ::For(MQuoteFor{ span, over, body }) => {
            let over = ToTokensHack::from(over);

            let mut body_stream = TokenStream::new();
            compile_with(body, &mut body_stream, scope);
            let mut body = Group::new(Delimiter::Brace, body_stream);
            body.set_span(span);

            stream.append_all(quote_spanned!( span => for #over #body ))
        }
        TokenTreeQ::Match(MQuoteMatch{ span, of, patterns }) => {
            let of = ToTokensHack::from(of);

            let patterns = patterns.into_iter()
                .map(|(of_span, pattern, body)| {
                    let mut stream = TokenStream::new();
                    compile_with(body, &mut stream, scope);
                    let mut body = Group::new(Delimiter::Brace, stream);
                    body.set_span(of_span);
                    (of_span, ToTokensHack::from(pattern), body)});
            let mut match_body = TokenStream::new();
            for (of_span, pattern, body) in patterns {
                match_body.append_all(quote_spanned!( of_span => #pattern => #body ));
            }
            let mut match_body = Group::new(Delimiter::Brace, match_body);
            match_body.set_span(span);
            stream.append_all(quote_spanned!( span => match #of #match_body));
        }
    }
}

fn put_token(token: TokenTree, stream: &mut TokenStream, scope: &Scope) {
    let Scope{ ref runtime, ref requested_span, ref token_stream_var} = scope;
    match token {
        TokenTree::Literal(lit) => {
            let span = lit.span();
            let stringed_lit = lit.to_string();
            let parsed_lit_var = Ident::new("s", Span::call_site());

            stream.append_all(quote_spanned!(span => {
                let #parsed_lit_var: #runtime::proc_macro2::TokenStream = #stringed_lit.parse().expect("invalid token stream");
                #token_stream_var.extend(#parsed_lit_var.into_iter().map(|mut t| {
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
            let punct_var = Ident::new("p", Span::call_site());
            stream.append_all(quote_spanned!(span => {
                let mut #punct_var = #runtime::proc_macro2::Punct::new(#op, #runtime::proc_macro2::Spacing::#spacing);
                #punct_var.set_span(#requested_span);
                #token_stream_var.extend(::std::iter::once(#runtime::proc_macro2::TokenTree::Punct(#punct_var)));
            }));
        }
        TokenTree::Ident(ident) => {
            let stringed_ident = ident.to_string();
            let span = ident.span();
            let ident_var = Ident::new("i", Span::call_site());
            stream.append_all(quote_spanned!(span => {
                let #ident_var = #runtime::proc_macro2::Ident::new(#stringed_ident, #requested_span);
                #token_stream_var.extend(::std::iter::once(#runtime::proc_macro2::TokenTree::Ident(#ident_var)));
            }));
        }
        TokenTree::Group(group) => {
            let qtoken = MQuoteGroup {
                span: group.span(),
                delimiter: group.delimiter(),
                tokens: group.stream().into_iter().map(TokenTreeQ::Plain).collect(),
            };
            put_qtoken(TokenTreeQ::Group(qtoken), stream, scope)
        }
    }
}

/// Wraps TokenTree iterator and implements ToTokens. Be careful in use.
struct ToTokensHack<I>(RefCell<Option<I>>);

impl<I> From<I> for ToTokensHack<I> {
    fn from(iter: I) -> Self {
        ToTokensHack(Some(iter).into())
    }
}

impl<I> ToTokens for ToTokensHack<I> where I: Iterator<Item=TokenTree> {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        token_stream.extend(self.0.borrow_mut().take().expect("tokens are already taken"))
    }
}
