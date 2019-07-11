use std::iter::{self, FromIterator};

use proc_macro2::{self, TokenTree, Delimiter, Spacing, Span};
use quote::TokenStreamExt;

use crate::error::{Result, Error};
use crate::buffer::QTokens;
use crate::language::*;

type TokenStream = iter::Peekable<<proc_macro2::TokenStream as IntoIterator>::IntoIter>;


enum ContextItem {
    ZeroPoint { tokens: QTokens },
    If { branches: Vec<(Span, TokenStream, QTokens)>, else_: Option<QTokens> },
    For { span: Span, over: TokenStream, body: QTokens },
    Match { span: Span, of: TokenStream, patterns: Vec<(Span, TokenStream, QTokens)> },
    LevelHolder { hold: TokenStream, span: Span, delimiter: Delimiter, tokens: QTokens },
}


struct Context {
    stack: Vec<ContextItem>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            stack: vec![ContextItem::ZeroPoint { tokens: QTokens::new() }],
        }
    }

    pub fn put_qtoken(&mut self, token: TokenTreeQ) -> Result<()> {
        match self.stack.last_mut() {
            Some(ContextItem::If { branches, else_}) => {
                match else_.as_mut() {
                    Some(tokens) => tokens.push(token),
                    None => branches.last_mut()
                        .expect("guaranteed by put_if")
                        .2.push(token)
                }
            }
            Some(ContextItem::For { body, .. }) =>
                body.push(token),
            Some(ContextItem::Match { patterns, .. }) => {
                match patterns.last_mut() {
                    Some(pattern) => pattern.2.push(token),
                    None => return Err(Error::new(token.span(), "Expected #{of ..}")),
                };
            }
            Some(ContextItem::LevelHolder { tokens, .. }) =>
                tokens.push(token),
            Some(ContextItem::ZeroPoint { tokens }) =>
                tokens.push(token),
            None => panic!("at least ZeroPoint must be in the context stack"),
        }
        Ok(())
    }

    pub fn put_if(&mut self, span: Span, mut condition: TokenStream) -> Result<()> {
        expect_something_in_stream(span, "#{if .. }", "condition", &mut condition)?;
        self.stack.push(ContextItem::If {
            branches: vec![(span, condition, QTokens::new())],
            else_: None,
        });
        Ok(())
    }
    pub fn put_elif(&mut self, span: Span, mut condition: TokenStream) -> Result<()> {
        expect_something_in_stream(span, "#{elif .. }", "condition", &mut condition)?;
        match self.stack.last_mut() {
            Some(ContextItem::If { branches, .. }) => {
                branches.push((span, condition, QTokens::new()));
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
                return Err(Error::new(span, format!("expected #{{{}}}, got #{{endif}}", expected))),
            _ =>
                return Err(Error::new(span, "unexpected endif")),
        }

        let (mut branches, else_) = match self.stack.pop() {
            Some(ContextItem::If{ branches, else_ }) => (branches, else_),
            _ => unreachable!("guaranteed by lookup_end_tag matching"),
        };

        let (if_span, condition, then) = branches.pop().expect("guaranteed by put_if");
        let mut mquote_if = MQuoteIf {
            span: if_span,
            condition: condition.collect(),
            then,
            else_,
        };

        while let Some((if_span, condition, then)) = branches.pop() {
            let next_if = MQuoteIf {
                span: if_span,
                condition: condition.collect(),
                then,
                else_: Some(QTokens::from(vec![TokenTreeQ::If(mquote_if)])),
            };
            mquote_if = next_if;
        }

        self.put_qtoken(TokenTreeQ::If(mquote_if))?;

        Ok(())
    }

    pub fn put_for(&mut self, span: Span, mut over: TokenStream) -> Result<()> {
        expect_something_in_stream(span, "#{for var in expression}", "var and expression", &mut over)?;
        self.stack.push(ContextItem::For {
            span,
            over,
            body: QTokens::new(),
        });
        Ok(())
    }

    pub fn put_endfor(&mut self, span: Span) -> Result<()> {
        match self.lookup_end_tag() {
            Some((0, "endfor")) => (),
            Some((_, "endfor")) =>
                return Err(Error::new(span, "#{for .. } is on different nesting level from #{endfor}, that is not permitted")),
            Some((0, expected)) =>
                return Err(Error::new(span, format!("expected #{{{}}}, got #{{endfor}}", expected))),
            _ =>
                return Err(Error::new(span, "unexpected endfor")),
        }

        let (span, over, body) = match self.stack.pop() {
            Some(ContextItem::For{ span, over, body }) => (span, over, body),
            _ => unreachable!("guaranteed by lookup_end_tag matching"),
        };

        self.put_qtoken(TokenTreeQ::For(MQuoteFor{
            span,
            over: over.collect(),
            body,
        }))
    }

    pub fn put_match(&mut self, span: Span, mut of: TokenStream) -> Result<()> {
        expect_something_in_stream(span, "#{match .. }", "matching expression", &mut of)?;
        self.stack.push(ContextItem::Match {
            span,
            of,
            patterns: vec![],
        });
        Ok(())
    }

    pub fn put_of(&mut self, span: Span, mut pattern: TokenStream) -> Result <()> {
        expect_something_in_stream(span, "#{of .. }", "pattern", &mut pattern)?;
        match self.stack.last_mut() {
            Some(ContextItem::Match { patterns, .. }) => {
                patterns.push((span, pattern, QTokens::new()));
                Ok(())
            }
            _ => Err(Error::new(span, "#{of .. } is only acceptable in context of \
                    #{match .. } ... #{endmatch}")),
        }
    }

    pub fn put_endmatch(&mut self, span: Span) -> Result<()> {
        match self.lookup_end_tag() {
            Some((0, "endmatch")) => (),
            Some((_, "endmatch")) =>
                return Err(Error::new(span, "#{match .. } is on different nesting level from #{endmatch}, that is not permitted")),
            Some((0, expected)) =>
                return Err(Error::new(span, format!("expected #{{{}}}, got #{{endmatch}}", expected))),
            _ =>
                return Err(Error::new(span, "unexpected endmatch")),
        }

        let (span, of, patterns) = match self.stack.pop() {
            Some(ContextItem::Match {span, of, patterns}) => (span, of, patterns),
            _ => unreachable!("guaranteed by above match"),
        };

        self.put_qtoken(TokenTreeQ::Match(MQuoteMatch{
            span,
            of: of.collect(),
            patterns: patterns.into_iter()
                .map(|(of_span, pattern, body)| (of_span, pattern.collect(), body))
                .collect()
        }))
    }

    pub fn put_holder(&mut self, held_tokens: TokenStream, span: Span, delimiter: Delimiter) -> Result<()> {
        self.stack.push(ContextItem::LevelHolder {
            hold: held_tokens,
            span,
            delimiter,
            tokens: QTokens::new(),
        });
        Ok(())
    }

    /// Releases holder on the top of stack
    pub fn try_release_holder(&mut self) -> Option<Result<TokenStream>> {
        match self.stack.last() {
            Some(ContextItem::LevelHolder { .. }) => (),
            _ => return None,
        }
        match self.stack.pop() {
            Some(ContextItem::LevelHolder { hold, span, delimiter, tokens }) => {
                let group = MQuoteGroup { span, delimiter, tokens };
                match self.put_qtoken(TokenTreeQ::Group(group)) {
                    Ok(()) => Some(Ok(hold)),
                    Err(e) => Some(Err(e)),
                }
            }
            _ => unreachable!("guaranteed by above match"),
        }
    }

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
            Some((i, ContextItem::Match { .. })) => Some((i, "endmatch")),
            None => None,
            Some((_, ContextItem::ZeroPoint { .. })) | Some((_, ContextItem::LevelHolder { .. }))
                => unreachable!("guaranteed by tag filter"),
        }
    }

    pub fn pick_result(mut self) -> Result<QTokens> {
        match self.stack.len() {
            0 => panic!("inconsistent context: at least ZeroPoint must be in the context stack"),
            1 => match self.stack.pop() {
                Some(ContextItem::ZeroPoint { tokens }) => return Ok(tokens),
                Some(_) => panic!("inconsistent context: the last element in stack must be ZeroPoint"),
                _ => unreachable!("guaranteed by outer match"),
            },
            _ => (),
        }

        let mut unclosed_tags = vec![];
        let mut eof = None;

        for item in self.stack.iter().rev() {
            match item {
                ContextItem::If { .. } => unclosed_tags.push("endif"),
                ContextItem::For { .. } => unclosed_tags.push("endfor"),
                ContextItem::Match { .. } => unclosed_tags.push("endmatch"),
                ContextItem::LevelHolder { span, .. } => { eof = Some(*span); break },
                ContextItem::ZeroPoint { .. } => break,
            }
        }

        let msg = unclosed_tags.join("}, #{");
        return Err(Error::new(eof.unwrap_or(Span::call_site()), format!("expected: #{{{}}}", msg)))
    }
}

pub fn parse_unspanned(token_stream: proc_macro2::TokenStream) -> Result<QTokens> {
    parse(token_stream.into_iter().peekable())
}

pub fn parse_spanned(token_stream: proc_macro2::TokenStream) -> Result<(proc_macro2::TokenStream, QTokens)> {
    let mut token_stream = token_stream.into_iter().peekable();
    let mut requested_span = vec![];

    while let Some(token) = token_stream.next() {
        if detect_end_of_span(&token, &mut token_stream) {
            return Ok((requested_span.into_iter().collect(), parse(token_stream)?))
        }
        requested_span.push(token)
    }

    return Err(Error::new(Span::call_site(), "expected span separated by '=>' from tokens"))
}

fn detect_end_of_span(current_token: &TokenTree, rest_stream: &mut TokenStream) -> bool {
    match current_token {
        TokenTree::Punct(punct) if punct.as_char() == '=' && punct.spacing() == Spacing::Joint
            => (),
        _ => return false,
    }
    match rest_stream.peek() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == '>' && punct.spacing() == Spacing::Alone
            => (),
        _ => return false,
    }
    let _ = rest_stream.next();
    true
}

fn parse(mut token_stream: TokenStream) -> Result<QTokens> {
    let mut context = Context::new();

    loop {
        while let Some(token) = token_stream.next() {
            // Check if it's reserved #{...}
            match token {
                TokenTree::Punct(punct) => {
                    let mut next_is_group = || token_stream.peek()
                        .into_iter()
                        .flat_map(TokenTreeExt::as_group)
                        .filter(|group| group.delimiter() == Delimiter::Brace)
                        .next().is_some();

                    if punct.as_char() == '#' && punct.spacing() == Spacing::Alone && next_is_group() {
                        let group = match token_stream.next() {
                            Some(TokenTree::Group(group)) => group,
                            _ => unreachable!("guaranteed by if")
                        };
                        let mut inner_stream = group.stream().into_iter().peekable();

                        match inner_stream.next() {
                            Some(TokenTree::Group(escaping)) => {
                                if let Some(token) = inner_stream.next() {
                                    let err = Error::new(token.span(), "invalid escaping");
                                    let last_span = inner_stream
                                        .map(|token| token.span())
                                        .last();
                                    return Err(if let Some(span) = last_span {
                                        err.end_span(span)
                                    } else {
                                        err
                                    })
                                }
                                context.put_qtoken(TokenTreeQ::Plain(punct.into()))?;
                                context.put_qtoken(TokenTreeQ::Plain(escaping.into()))?;
                            }
                            Some(TokenTree::Ident(ident)) => {
                                let span = ident.span();
                                match ident.to_string().as_str() {
                                    "if" => context.put_if(span, inner_stream)?,
                                    "elif" => context.put_elif(span, inner_stream)?,
                                    "else" => {
                                        context.put_else(span)?;
                                        expect_empty_stream("#{else}", &mut inner_stream)?
                                    },
                                    "endif" => {
                                        context.put_endif(span)?;
                                        expect_empty_stream("#{endif}", &mut inner_stream)?;
                                    },
                                    "for" => context.put_for(span, inner_stream)?,
                                    "endfor" => {
                                        context.put_endfor(span)?;
                                        expect_empty_stream("#{endfor}", &mut inner_stream)?;
                                    },
                                    "match" => context.put_match(span, inner_stream)?,
                                    "of" => context.put_of(span, inner_stream)?,
                                    "endmatch" => {
                                        context.put_endmatch(span)?;
                                        expect_empty_stream("#{endmatch}", &mut inner_stream)?;
                                    },
                                    _ =>
                                        context.put_qtoken(TokenTreeQ::Insertion(group.span(), group.stream()))?,
                                }
                            }
                            Some(_) =>
                                context.put_qtoken(TokenTreeQ::Insertion(group.span(), group.stream()))?,
                            None =>
                                return Err(Error::new(group.span(), "expected tag or expression, got nothing"))
                        }
                    } else {
                        context.put_qtoken(TokenTreeQ::Plain(punct.into()))?
                    }
                }
                TokenTree::Group(group) => {
                    let span = group.span();
                    let delimiter = group.delimiter();
                    let inner_stream = group.stream();
                    context.put_holder(token_stream, span, delimiter)?;
                    token_stream = inner_stream.into_iter().peekable();
                }
                token => context.put_qtoken(TokenTreeQ::Plain(token))?,
            }
        }

        match context.try_release_holder() {
            Some(Ok(held_tokens)) => {
                token_stream = held_tokens;
                continue;
            }
            Some(Err(err)) => return Err(err),
            None => (),
        }
        break;
    }

    context.pick_result()
}

fn expect_something_in_stream(span: Span, expectation_place: &str, expected: &str, token_stream: &mut TokenStream) -> Result<()> {
    if token_stream.peek().is_none() {
        return Err(Error::new(span, format!("Missing {expected} in {place}", expected=expected, place=expectation_place)))
    }
    Ok(())
}

fn expect_empty_stream(expected: &str, token_stream: &mut TokenStream) -> Result<()> {
    if let Some(token) = token_stream.peek() {
        return Err(Error::new(token.span(), format!("Expected {} without any additional data", expected)))
    }
    Ok(())
}

trait TokenTreeExt {
    fn as_group(&self) -> Option<&proc_macro2::Group>;
}

impl TokenTreeExt for TokenTree {
    fn as_group(&self) -> Option<&proc_macro2::Group> {
        match self {
            &TokenTree::Group(ref group) => Some(group),
            _ => None,
        }
    }
}
