use proc_macro::{TokenTree, Spacing};

use crate::language::*;

fn parse<S>(token_stream: S) -> TokenStreamQ
    where S: IntoIterator<Item=TokenTree>
{
    let mut found_qtoken = false;
    let mut token_stream = token_stream.into_iter().peekable();
    let mut output: Vec<TokenTreeQ> = vec![];

    while let Some(token) = token_stream.next() {
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
                            assert!(inner_stream.next().is_none(), "invalid escaping");
                            output.push(TokenTreeQ::Plain(punct.into()));
                            output.push(TokenTreeQ::Plain(group.into()));
                        }
                        Some(TokenTree::Ident(ident)) => {
                            
                        }
                    }
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
