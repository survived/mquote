use proc_macro2::*;
use quote::{TokenStreamExt, quote, quote_spanned};

use crate::language::*;
use crate::buffer::QTokens;

struct Scope {
    token_stream_var: Ident,
    requested_span: TokenStream,
    runtime: TokenStream,
}

pub fn compile(mquote: QTokens, requested_span: Option<TokenStream>) -> TokenStream {
    let runtime = quote!(::mquote::__rt);
    let scope = Scope {
        token_stream_var: Ident::new("__token_stream", Span::call_site()),
        requested_span: requested_span.unwrap_or(quote!(#runtime::proc_macro2::Span::call_site())),
        runtime: runtime.clone(),
    };
    let mut stream = TokenStream::new();
    compile_with(mquote, &mut stream, &scope);

    let ref token_stream_var = scope.token_stream_var;
    quote!{{
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
        TokenTreeQ::Insertion(tokens) => {
            assert_ne!(token_stream_var.to_string(), "__insertion");
            stream.append_all(quote!({
                let ref __insertation = #tokens;
                #runtime::quote::ToTokens::to_tokens(__insertation, &mut #token_stream_var);
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
                token_stream_var: Ident::new("inner_stream", span),
                requested_span: requested_span.clone(),
                runtime: runtime.clone(),
            };
            let ref inner_stream_var = inner_scope.token_stream_var;
            let constructing_group_var = Ident::new("constructing_group", span);

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
        TokenTreeQ::If(MQuoteIf{ condition, then, else_}) => {
            let mut then_stream = TokenStream::new();
            compile_with(then, &mut then_stream, scope);
            let then_branch = Group::new(Delimiter::Brace, then_stream);

            let mut else_stream = TokenStream::new();
            if let Some(else_) = else_ {
                compile_with(else_, &mut else_stream, scope);
            }
            let else_branch = Group::new(Delimiter::Brace, else_stream);

            stream.append_all(quote!( if #condition #then_branch else #else_branch ))
        }
        TokenTreeQ::For(MQuoteFor{ over, body }) => {
            let mut body_stream = TokenStream::new();
            compile_with(body, &mut body_stream, scope);
            let body = Group::new(Delimiter::Brace, body_stream);

            stream.append_all(quote!( for #over #body ))
        }
        TokenTreeQ::Match(MQuoteMatch{ of, patterns }) => {
            let patterns = patterns.into_iter()
                .map(|(pattern, body)| {
                    let mut stream = TokenStream::new();
                    compile_with(body, &mut stream, scope);
                    let body = Group::new(Delimiter::Brace, stream);
                    (pattern, body)});
            let mut match_body = TokenStream::new();
            for (pattern, body) in patterns {
                match_body.append_all(quote!(#pattern => #body));
            }
            let match_body = Group::new(Delimiter::Brace, match_body);
            stream.append_all(quote!(match #of #match_body));
        }
    }
}

fn put_token(token: TokenTree, stream: &mut TokenStream, scope: &Scope) {
    let Scope{ ref runtime, ref requested_span, ref token_stream_var} = scope;
    match token {
        TokenTree::Literal(lit) => {
            let span = lit.span();
            let stringed_lit = lit.to_string();

            stream.append_all(quote_spanned!(span => {
                let s: #runtime::proc_macro2::TokenStream = #stringed_lit.parse().expect("invalid token stream");
                #token_stream_var.extend(s.into_iter().map(|mut t| {
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
                let mut p = #runtime::proc_macro2::Punct::new(#op, #runtime::proc_macro2::Spacing::#spacing);
                p.set_span(#requested_span);
                #token_stream_var.extend(::std::iter::once(#runtime::proc_macro2::TokenTree::Punct(p)));
            }));
        }
        TokenTree::Ident(ident) => {
            let stringed_ident = ident.to_string();
            let span = ident.span();
            stream.append_all(quote_spanned!(span => {
                let i = #runtime::proc_macro2::Ident::new(#stringed_ident, #requested_span);
                #token_stream_var.extend(::std::iter::once(#runtime::proc_macro2::TokenTree::Ident(i)));
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
