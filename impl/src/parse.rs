use std::iter;

use proc_macro2::{self, TokenTree, Delimiter, Spacing, Span};

use crate::error::{Result, Error};
use crate::buffer::QTokens;
use crate::language::*;
use std::hint::unreachable_unchecked;

type TokenStream = iter::Peekable<<proc_macro2::TokenStream as IntoIterator>::IntoIter>;


enum ContextItem {
    ZeroPoint { tokens: QTokens },
    If { branches: Vec<(TokenStream, QTokens)>, else_: Option<QTokens> },
    For { over: TokenStream, body: QTokens },
    LevelHolder { hold: TokenStream, span: Span, delimiter: Delimiter, tokens: QTokens, end_span: Span },
}


struct Context {
    token_stream: TokenStream,
    stack: Vec<ContextItem>,
}

impl Context {
    pub fn new(token_stream: proc_macro2::TokenStream) -> Self {
        Self {
            token_stream: token_stream.into_iter().peekable(),
            stack: vec![ContextItem::ZeroPoint { tokens: QTokens::new() }],
        }
    }

    pub fn next_token(&mut self) -> Result<Option<TokenTree>> {
        if let Some(token) = self.token_stream.next() {
            return Ok(Some(token));
        }

        let level_is_held = match self.stack.last() {
            Some(ContextItem::LevelHolder { .. }) => true,
            _ => false,
        };

        if level_is_held {
            let (held, span, delimiter, tokens) = match self.stack.pop() {
                Some(ContextItem::LevelHolder { hold, span, delimiter, tokens}) =>
                    (hold, span, delimiter, tokens),
                _ => unreachable!("guaranteed by level_is_held"),
            };

            self.put_qtoken(TokenTreeQ::Group(MQuoteGroup {
                delimiter,
                span,
                tokens,
            }));
            self.token_stream = held;

            return self.next_token();
        }

        if self.stack.len() == 1 {
            return Ok(None)
        }

        let mut unclosed_tags = vec![];
        let mut eof = None;

        for item in self.stack.iter().rev() {
            match item {
                ContextItem::If { branches, .. } => unclosed_tags.push((branches[0].0, "endif")),
                ContextItem::For { tag_span, .. } => unclosed_tags.push((*tag_span, "endfor")),
                ContextItem::LevelHolder { end_span, .. } => { eof = Some(*end_span); break },
                ContextItem::ZeroPoint { .. } => break,
            }
        }

        let msg = unclosed_tags.join("}, #{");
        return Err(Error::new(eof.unwrap_or(Span::call_site()), format!("expected: #{{{}}}", msg)))
    }

    pub fn put_qtoken(&mut self, token: TokenTreeQ) {
        match self.stack.last_mut() {
            Some(ContextItem::If { branches, else_}) => {
                match else_.as_mut() {
                    Some(tokens) => tokens.push(token),
                    None => branches.last_mut()
                        .expect("guaranteed by put_if")
                        .1.push(token)
                }
            }
            Some(ContextItem::For { body, .. }) =>
                body.push(token),
            Some(ContextItem::LevelHolder { tokens, .. }) =>
                tokens.push(token),
            Some(ContextItem::ZeroPoint { tokens }) =>
                tokens.push(token),
            None => panic!("at least ZeroPoint must be in the context stack"),
        }
    }

    pub fn put_if(&mut self, _span: Span, condition: TokenStream) -> Result<()> {
        self.stack.push(ContextItem::If {
            branches: vec![(condition, QTokens::new())],
            else_: None,
        });
        Ok(())
    }
    pub fn put_elif(&mut self, span: Span, condition: TokenStream) -> Result<()> {
        match self.stack.last_mut() {
            Some(ContextItem::If { branches, .. }) => {
                branches.push((condition, QTokens::new()));
                Ok(())
            }
            _ => Err(Error::new(span, "elif is only acceptable in context \
                    of #{if ...} ... #{endif}")),
        }
    }
    pub fn put_else(&mut self, span: Span) -> Result<()> {
        match self.stack.last_mut() {
            Some(ContextItem::If { else_, .. }) => {
                if else_.is_some() {
                    return Err(Error::new(span, "duplicated else branch"))
                }
                *else_ = Some(QTokens::new());
                Ok(())
            }
            _ => Err(Error::new(span, "else is only acceptable in context \
                    of #{if ...} ... #{endif}")),
        }
    }
    pub fn put_endif(&mut self, span: Span) -> Result<()> {
        match self.lookup_end_tag() {
            Some((0, "endif")) => (),
            Some((_, "endif")) =>
                return Err(Error::new(span, "#{if .. } is on different nesting level from #{endif}, that is not permitted")),
            Some((0, expected)) =>
                return Err(Error::new(span, format!("expected {}, got endif", expected))),
            _ =>
                return Err(Error::new(span, "unexpected endif")),
        }

        let (mut branches, else_) = match self.stack.pop() {
            Some(ContextItem::If{ branches, else_ }) => (branches, else_),
            _ => unreachable!("guaranteed by lookup_end_tag matching"),
        };

        let (condition, then) = branches.pop().expect("guaranteed by put_if");
        let mut mquote_if = MQuoteIf {
            condition: condition.collect(),
            then,
            else_,
        };

        while let Some((condition, then)) = branches.pop() {
            let next_if = MQuoteIf {
                condition: condition.collect(),
                then,
                else_: Some(QTokens::from(vec![TokenTreeQ::If(mquote_if)])),
            };
            mquote_if = next_if;
        }

        self.put_qtoken(TokenTreeQ::If(mquote_if));

        Ok(())
    }

    pub fn put_group(&mut self, group: )


    pub fn lookup_end_tag(&self) -> Option<(usize, &'static str)> {
        let tag = self.stack.iter().rev()
            .enumerate()
            .filter(|(_, tag)| match tag {
                ContextItem::ZeroPoint { .. } => false,
                ContextItem::LevelHolder { .. } => false,
                _ => true })
            .next();
        match tag {
            Some((i, ContextItem::If { .. })) => Some((i, "endif")),
            Some((i, ContextItem::For { .. })) => Some((i, "endfor")),
            None => None,
            Some((_, ContextItem::ZeroPoint { .. })) | Some((_, ContextItem::LevelHolder { .. }))
                => unreachable!("guaranteed by tag filter"),
        }
    }
}

fn parse(token_stream: proc_macro2::TokenStream) -> Result<TokenStreamQ> {
    let mut context = Context::new(token_stream);

    while let Some(token) = context.next_token()? {
        // Check if it's reserved #{...}
        match token {
            TokenTree::Punct(punct) => {
                let next_is_group = || token_stream.peek().map(TokenTreeExt::is_group).unwrap_or(false);
                if punct.as_char() == '#' && punct.spacing() == Spacing::Alone && next_is_group() {
                    let group = match token_stream.next() {
                        TokenTree::Group(group) => group,
                        _ => unreachable!("guaranteed by if")
                    };
                    let mut inner_stream = group.stream().into_iter();

                    match inner_stream.next() {
                        Some(TokenTree::Group(escaping)) => {
                            if let Some(token) = inner_steam.next() {
                                let mut err = Error::new(token.span(), "invalid escaping");
                                let last_span = inner_stream
                                    .map(|token| token.span())
                                    .last();
                                if let Some(span) = last_span {
                                    err.end_span(span);
                                }
                                return Err(err);
                            }
                            output.push(TokenTreeQ::Plain(punct.into()));
                            output.push(TokenTreeQ::Plain(group.into()));
                        }
                        Some(TokenTree::Ident(ident)) => {
                            let span = ident.span();
                            match ident.to_string().as_str() {
                                "if" => context.put_if(span, inner_stream)?,
                                "elif" => context.put_elif(span, inner_stream)?,
                                "else" => context.put_else(span)?,
                                "endif" => context.put_endif(span)?,
                                _ => {
                                    let mut token_stream = proc_macro2::TokenStream::new();
                                    token_stream.append(TokenTree::Ident(ident));
                                    token_stream.append_all(inner_stream);
                                    context.put_qtoken(TokenTreeQ::Insertion(token_stream));
                                },
                            }
                        }
                        Some(token) =>
                            return Err(Error::new(token.span(), "invalid mquote syntax, expected tag")),
                        None =>
                            return Err(Error::new(group.span(), "expected tag or expression, got nothing"))
                    }
                } else {
                    context.put_qtoken(TokenTreeQ::Plain(punct.into()))
                }
            }
        }
    }

    unimplemented!()
}

trait TokenTreeExt {
    fn is_group(&self) -> bool;
}

impl TokenTreeExt for TokenTree {
    fn is_group(&self) -> bool {
        match self {
            &TokenTree::Group(_) => true,
            _ => false,
        }
    }
}
