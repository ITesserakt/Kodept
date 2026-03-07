use crate::common::{EagerTokensProducer, TokenProducer};
use crate::lexer::Token;
use crate::lexer::bare_metal::Error::{NotANumber, UnclosedChar, UnclosedString, Unknown};
use crate::token_match::TokenMatch;
use kodept_core::code_point::CodePoint;
use std::cell::Cell;
use std::convert::Infallible;

struct Sink<F>(F);

#[derive(Debug, Copy, Clone)]
pub struct Lexer;

enum Error {
    UnclosedChar,
    UnclosedString,
    NotANumber,
    Unknown,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer
    }
}

impl<F: FnMut(Token)> Sink<F> {
    #[inline(always)]
    fn push(&mut self, token: Token) {
        self.0(token)
    }

    fn parse<'t>(&mut self, input: &'t [u8]) -> Result<&'t [u8], Error> {
        #[inline]
        fn boundary(rest: &[u8]) -> bool {
            match rest {
                [b'_', ..] => false,
                [c, ..] if c.is_ascii_alphanumeric() => false,
                [_, ..] => true,
                [] => true,
            }
        }

        match input {
            [b'\n', rest @ ..] | [b'\r', b'\n', rest @ ..] => {
                self.push(Token::Newline);
                Ok(rest)
            }
            [b'\t' | b' ', rest @ ..] => {
                let not_space = rest.iter().position(|&it| it != b'\t' && it != b' ');
                self.push(Token::Whitespace);
                match not_space {
                    Some(pos) => Ok(&rest[pos..]),
                    None => Ok(&[]),
                }
            }
            [b'/', b'/', rest @ ..] => {
                // TODO: support \r\n endings
                let separator = rest.iter().position(|it| matches!(it, b'\n'));
                self.push(Token::Comment);
                if let Some(pos) = separator {
                    Ok(&rest[pos..])
                } else {
                    Ok(&[])
                }
            }
            [b'f', b'u', b'n', rest @ ..] if boundary(rest) => {
                self.push(Token::Fun);
                Ok(rest)
            }
            [b'v', b'a', b'l', rest @ ..] if boundary(rest) => {
                self.push(Token::Val);
                Ok(rest)
            }
            [b'v', b'a', b'r', rest @ ..] if boundary(rest) => {
                self.push(Token::Var);
                Ok(rest)
            }
            [b'm', b'a', b't', b'c', b'h', rest @ ..] if boundary(rest) => {
                self.push(Token::Match);
                Ok(rest)
            }
            [b'w', b'h', b'i', b'l', b'e', rest @ ..] if boundary(rest) => {
                self.push(Token::While);
                Ok(rest)
            }
            [b'm', b'o', b'd', b'u', b'l', b'e', rest @ ..] if boundary(rest) => {
                self.push(Token::Module);
                Ok(rest)
            }
            [b'e', b'x', b't', b'e', b'n', b'd', rest @ ..] if boundary(rest) => {
                self.push(Token::Extend);
                Ok(rest)
            }
            [b'r', b'e', b't', b'u', b'r', b'n', rest @ ..] if boundary(rest) => {
                self.push(Token::Return);
                Ok(rest)
            }
            [b'i', b'f', rest @ ..] if boundary(rest) => {
                self.push(Token::If);
                Ok(rest)
            }
            [b'e', b'l', b'i', b'f', rest @ ..] if boundary(rest) => {
                self.push(Token::Elif);
                Ok(rest)
            }
            [b'e', b'l', b's', b'e', rest @ ..] if boundary(rest) => {
                self.push(Token::Else);
                Ok(rest)
            }
            [b'a', b'b', b's', b't', b'r', b'a', b'c', b't', rest @ ..] if boundary(rest) => {
                self.push(Token::Abstract);
                Ok(rest)
            }
            [b't', b'r', b'a', b'i', b't', rest @ ..] if boundary(rest) => {
                self.push(Token::Trait);
                Ok(rest)
            }
            [b's', b't', b'r', b'u', b'c', b't', rest @ ..] if boundary(rest) => {
                self.push(Token::Struct);
                Ok(rest)
            }
            [b'c', b'l', b'a', b's', b's', rest @ ..] if boundary(rest) => {
                self.push(Token::Class);
                Ok(rest)
            }
            [b'e', b'n', b'u', b'm', rest @ ..] if boundary(rest) => {
                self.push(Token::Enum);
                Ok(rest)
            }
            [b'f', b'o', b'r', b'e', b'i', b'g', b'n', rest @ ..] if boundary(rest) => {
                self.push(Token::Foreign);
                Ok(rest)
            }
            [b't', b'y', b'p', b'e', rest @ ..] if boundary(rest) => {
                self.push(Token::TypeAlias);
                Ok(rest)
            }
            [b'w', b'i', b't', b'h', rest @ ..] if boundary(rest) => {
                self.push(Token::With);
                Ok(rest)
            }
            [b',', rest @ ..] => {
                self.push(Token::Comma);
                Ok(rest)
            }
            [b';', rest @ ..] => {
                self.push(Token::Semicolon);
                Ok(rest)
            }
            [b'{', rest @ ..] => {
                self.push(Token::LBrace);
                Ok(rest)
            }
            [b'}', rest @ ..] => {
                self.push(Token::RBrace);
                Ok(rest)
            }
            [b'[', rest @ ..] => {
                self.push(Token::LBracket);
                Ok(rest)
            }
            [b']', rest @ ..] => {
                self.push(Token::RBracket);
                Ok(rest)
            }
            [b'(', rest @ ..] => {
                self.push(Token::LParen);
                Ok(rest)
            }
            [b')', rest @ ..] => {
                self.push(Token::RParen);
                Ok(rest)
            }
            [b':', b':', rest @ ..] => {
                self.push(Token::DoubleColon);
                Ok(rest)
            }
            [b':', rest @ ..] => {
                self.push(Token::Colon);
                Ok(rest)
            }
            [b'=', b'>', rest @ ..] => {
                self.push(Token::Flow);
                Ok(rest)
            }
            [b'+', rest @ ..] => {
                self.push(Token::Plus);
                Ok(rest)
            }
            [b'-', rest @ ..] => {
                self.push(Token::Sub);
                Ok(rest)
            }
            [b'*', b'*', rest @ ..] => {
                self.push(Token::Pow);
                Ok(rest)
            }
            [b'*', rest @ ..] => {
                self.push(Token::Times);
                Ok(rest)
            }
            [b'/', rest @ ..] => {
                self.push(Token::Div);
                Ok(rest)
            }
            [b'%', rest @ ..] => {
                self.push(Token::Mod);
                Ok(rest)
            }
            [b'<', b'=', b'>', rest @ ..] => {
                self.push(Token::Spaceship);
                Ok(rest)
            }
            [b'=', b'=', rest @ ..] => {
                self.push(Token::Equiv);
                Ok(rest)
            }
            [b'=', rest @ ..] => {
                self.push(Token::Equals);
                Ok(rest)
            }
            [b'!', b'=', rest @ ..] => {
                self.push(Token::NotEquiv);
                Ok(rest)
            }
            [b'>', b'=', rest @ ..] => {
                self.push(Token::GreaterEquals);
                Ok(rest)
            }
            [b'>', rest @ ..] => {
                self.push(Token::Greater);
                Ok(rest)
            }
            [b'<', b'=', rest @ ..] => {
                self.push(Token::LessEquals);
                Ok(rest)
            }
            [b'<', rest @ ..] => {
                self.push(Token::Less);
                Ok(rest)
            }
            [b'|', b'|', rest @ ..] => {
                self.push(Token::OrLogic);
                Ok(rest)
            }
            [b'&', b'&', rest @ ..] => {
                self.push(Token::AndLogic);
                Ok(rest)
            }
            [b'!', rest @ ..] => {
                self.push(Token::NotLogic);
                Ok(rest)
            }
            [b'|', rest @ ..] => {
                self.push(Token::OrBit);
                Ok(rest)
            }
            [b'&', rest @ ..] => {
                self.push(Token::AndBit);
                Ok(rest)
            }
            [b'^', rest @ ..] => {
                self.push(Token::XorBit);
                Ok(rest)
            }
            [b'~', rest @ ..] => {
                self.push(Token::NotBit);
                Ok(rest)
            }
            [b'\'', _, b'\'', rest @ ..] => {
                self.push(Token::Char);
                Ok(rest)
            }
            [b'\'', ..] => Err(UnclosedChar),
            [b'"', rest @ ..] => {
                let closing = rest.iter().position(|&it| it == b'"');
                match closing {
                    Some(pos) => {
                        self.push(Token::String);
                        Ok(&rest[pos + 1..])
                    }
                    None => Err(UnclosedString),
                }
            }
            [b'_', b'A'..=b'Z', rest @ ..] | [b'A'..=b'Z', rest @ ..] => {
                let not_letter = rest
                    .iter()
                    .position(|&it| !it.is_ascii_alphanumeric() && it != b'_');
                self.push(Token::Type);
                match not_letter {
                    Some(pos) => Ok(&rest[pos..]),
                    None => Ok(&[]),
                }
            }
            [b'_', b'a'..=b'z', rest @ ..] | [b'a'..=b'z', rest @ ..] => {
                let not_letter = rest
                    .iter()
                    .position(|&it| !it.is_ascii_alphanumeric() && it != b'_');
                self.push(Token::Identifier);
                match not_letter {
                    Some(pos) => Ok(&rest[pos..]),
                    None => Ok(&[]),
                }
            }
            // TODO: support multiple __ in identifiers
            [b'_', rest @ ..] => {
                self.push(Token::TypeGap);
                Ok(rest)
            }
            [
                b'0',
                system @ (b'b' | b'B' | b'c' | b'C' | b'x' | b'X'),
                first,
                rest @ ..,
            ] => {
                #[inline(always)]
                fn digit_matches(system: &u8, digit: u8) -> bool {
                    match system {
                        b'b' | b'B' => matches!(digit, b'0'..=b'1'),
                        b'c' | b'C' => matches!(digit, b'0'..=b'7'),
                        b'x' | b'X' => matches!(digit, b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f'),
                        _ => false,
                    }
                }

                if !digit_matches(system, *first) {
                    Err(NotANumber)
                } else {
                    let not_number = rest
                        .iter()
                        .position(|&it| it != b'_' && !digit_matches(system, it));
                    match system {
                        b'b' | b'B' => self.push(Token::Binary),
                        b'c' | b'C' => self.push(Token::Octal),
                        b'x' | b'X' => self.push(Token::Hex),
                        _ => unreachable!(),
                    }
                    match not_number {
                        Some(pos) => Ok(&rest[pos..]),
                        None => Ok(&[]),
                    }
                }
            }
            // TODO: add sign and e-notation support
            [b'0'..=b'9', rest @ ..] => {
                let not_digit = rest.iter().position(|it| !matches!(it, b'0'..=b'9'));
                match not_digit {
                    None => {
                        self.push(Token::Floating);
                        Ok(&[])
                    }
                    Some(pos) => {
                        let input = &rest[pos..];
                        self.push(Token::Floating);
                        if !matches!(input, [b'.', ..]) {
                            return Ok(input);
                        }
                        let input = &input[1..];
                        let not_digit = input.iter().position(|it| !matches!(it, b'0'..=b'9'));
                        match not_digit {
                            Some(pos) => Ok(&input[pos..]),
                            None => Ok(&[]),
                        }
                    }
                }
            }
            [b'.', b'0'..=b'9', rest @ ..] => {
                let not_digit = rest.iter().position(|it| !matches!(it, b'0'..=b'9'));
                self.push(Token::Floating);
                match not_digit {
                    None => Ok(&[]),
                    Some(pos) => Ok(&rest[pos..]),
                }
            }
            [b'.', rest @ ..] => {
                self.push(Token::Dot);
                Ok(rest)
            }
            _ => Err(Unknown),
        }
    }
}

fn recover_from_error(input: &[u8], error: Error) -> TokenMatch {
    match error {
        UnclosedChar => TokenMatch {
            token: Token::Char,
            point: CodePoint {
                length: 2,
                offset: 0,
            },
        },
        e @ UnclosedString | e @ NotANumber => {
            let whitespace = input.iter().position(|it| matches!(it, b' ' | b'\t'));
            match whitespace {
                None => TokenMatch {
                    token: Token::Unknown,
                    point: CodePoint {
                        length: input.len() as u32,
                        offset: 0,
                    },
                },
                Some(pos) => TokenMatch {
                    token: if matches!(e, UnclosedString) {
                        Token::String
                    } else {
                        Token::Unknown
                    },
                    point: CodePoint {
                        length: pos as u32,
                        offset: 0,
                    },
                },
            }
        }
        Unknown => TokenMatch {
            token: Token::Unknown,
            point: CodePoint::single_point(0),
        },
    }
}

impl TokenProducer for Lexer {
    type Error<'t> = Infallible;

    fn parse_string<'t>(
        &self,
        whole_input: &'t str,
        position: usize,
    ) -> Result<TokenMatch, Self::Error<'t>> {
        let input = whole_input[position..].as_bytes();
        let mut token = Token::Unknown;
        let mut sink = Sink(|it| token = it);

        match sink.parse(input) {
            Ok(rest) => {
                let length = input.len() - rest.len();
                Ok(TokenMatch {
                    token,
                    point: CodePoint {
                        length: length as u32,
                        offset: 0,
                    },
                })
            }
            Err(e) => Ok(recover_from_error(input, e)),
        }
    }
}

impl EagerTokensProducer for Lexer {
    type Error<'t> = Infallible;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<TokenMatch>, Self::Error<'t>> {
        let mut tokens = vec![];
        let mut offset = 0;
        let current_token = Cell::new(Token::Unknown);
        let mut sink = Sink(|it| current_token.set(it));
        let mut input = input.as_bytes();

        while !input.is_empty() {
            match sink.parse(input) {
                Ok(rest) => {
                    let length = input.len() - rest.len();
                    tokens.push(TokenMatch {
                        token: current_token.get(),
                        point: CodePoint {
                            length: length as u32,
                            offset,
                        },
                    });
                    offset += length as u32;
                    input = rest;
                }
                Err(e) => {
                    let mut token_match = recover_from_error(input, e);
                    token_match.point.offset = offset;
                    tokens.push(token_match);
                }
            }
        }
        Ok(tokens)
    }
}
