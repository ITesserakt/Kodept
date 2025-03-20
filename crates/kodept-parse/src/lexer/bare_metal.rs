use std::cell::Cell;
use crate::common::{EagerTokensProducer, TokenProducer};
use crate::lexer::bare_metal::Error::{NotANumber, UnclosedChar, UnclosedString, Unknown};
use crate::lexer::PackedToken;
use crate::token_match::PackedTokenMatch;
use kodept_core::code_point::CodePoint;
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

impl<F: FnMut(PackedToken)> Sink<F> {
    fn push(&mut self, token: PackedToken) {
        self.0(token)
    }

    fn parse<'t>(&mut self, input: &'t [u8]) -> Result<&'t [u8], Error> {
        match input {
            [b'\n', rest @ ..] | [b'\r', b'\n', rest @ ..] => {
                self.push(PackedToken::Newline);
                Ok(rest)
            }
            [b'\t' | b' ', rest @ ..] => {
                let not_space = rest.iter().position(|&it| it != b'\t' && it != b' ');
                self.push(PackedToken::Whitespace);
                match not_space {
                    Some(pos) => Ok(&rest[pos..]),
                    None => Ok(&[]),
                }
            }
            [b'f', b'u', b'n', rest @ ..] => {
                self.push(PackedToken::Fun);
                Ok(rest)
            }
            [b'v', b'a', b'l', rest @ ..] => {
                self.push(PackedToken::Val);
                Ok(rest)
            }
            [b'v', b'a', b'r', rest @ ..] => {
                self.push(PackedToken::Var);
                Ok(rest)
            }
            [b'm', b'a', b't', b'c', b'h', rest @ ..] => {
                self.push(PackedToken::Match);
                Ok(rest)
            }
            [b'w', b'h', b'i', b'l', b'e', rest @ ..] => {
                self.push(PackedToken::While);
                Ok(rest)
            }
            [b'm', b'o', b'd', b'u', b'l', b'e', rest @ ..] => {
                self.push(PackedToken::Module);
                Ok(rest)
            }
            [b'e', b'x', b't', b'e', b'n', b'd', rest @ ..] => {
                self.push(PackedToken::Extend);
                Ok(rest)
            }
            [b'r', b'e', b't', b'u', b'r', b'n', rest @ ..] => {
                self.push(PackedToken::Return);
                Ok(rest)
            }
            [b'\\', rest @ ..] => {
                self.push(PackedToken::Lambda);
                Ok(rest)
            }
            [b'i', b'f', rest @ ..] => {
                self.push(PackedToken::If);
                Ok(rest)
            }
            [b'e', b'l', b'i', b'f', rest @ ..] => {
                self.push(PackedToken::Elif);
                Ok(rest)
            }
            [b'e', b'l', b's', b'e', rest @ ..] => {
                self.push(PackedToken::Else);
                Ok(rest)
            }
            [b'a', b'b', b's', b't', b'r', b'a', b'c', b't', rest @ ..] => {
                self.push(PackedToken::Abstract);
                Ok(rest)
            }
            [b't', b'r', b'a', b'i', b't', rest @ ..] => {
                self.push(PackedToken::Trait);
                Ok(rest)
            }
            [b's', b't', b'r', b'u', b'c', b't', rest @ ..] => {
                self.push(PackedToken::Struct);
                Ok(rest)
            }
            [b'c', b'l', b'a', b's', b's', rest @ ..] => {
                self.push(PackedToken::Class);
                Ok(rest)
            }
            [b'e', b'n', b'u', b'm', rest @ ..] => {
                self.push(PackedToken::Enum);
                Ok(rest)
            }
            [b'f', b'o', b'r', b'e', b'i', b'g', b'n', rest @ ..] => {
                self.push(PackedToken::Foreign);
                Ok(rest)
            }
            [b't', b'y', b'p', b'e', rest @ ..] => {
                self.push(PackedToken::TypeAlias);
                Ok(rest)
            }
            [b'w', b'i', b't', b'h', rest @ ..] => {
                self.push(PackedToken::With);
                Ok(rest)
            }
            [b',', rest @ ..] => {
                self.push(PackedToken::Comma);
                Ok(rest)
            }
            [b';', rest @ ..] => {
                self.push(PackedToken::Semicolon);
                Ok(rest)
            }
            [b'{', rest @ ..] => {
                self.push(PackedToken::LBrace);
                Ok(rest)
            }
            [b'}', rest @ ..] => {
                self.push(PackedToken::RBrace);
                Ok(rest)
            }
            [b'[', rest @ ..] => {
                self.push(PackedToken::LBracket);
                Ok(rest)
            }
            [b']', rest @ ..] => {
                self.push(PackedToken::RBracket);
                Ok(rest)
            }
            [b'(', rest @ ..] => {
                self.push(PackedToken::LParen);
                Ok(rest)
            }
            [b')', rest @ ..] => {
                self.push(PackedToken::RParen);
                Ok(rest)
            }
            [b':', b':', rest @ ..] => {
                self.push(PackedToken::DoubleColon);
                Ok(rest)
            }
            [b':', rest @ ..] => {
                self.push(PackedToken::Colon);
                Ok(rest)
            }
            [b'=', b'>', rest @ ..] => {
                self.push(PackedToken::Flow);
                Ok(rest)
            }
            [b'+', rest @ ..] => {
                self.push(PackedToken::Plus);
                Ok(rest)
            }
            [b'-', rest @ ..] => {
                self.push(PackedToken::Sub);
                Ok(rest)
            }
            [b'*', b'*', rest @ ..] => {
                self.push(PackedToken::Pow);
                Ok(rest)
            }
            [b'*', rest @ ..] => {
                self.push(PackedToken::Times);
                Ok(rest)
            }
            [b'/', rest @ ..] => {
                self.push(PackedToken::Div);
                Ok(rest)
            }
            [b'%', rest @ ..] => {
                self.push(PackedToken::Mod);
                Ok(rest)
            }
            [b'<', b'=', b'>', rest @ ..] => {
                self.push(PackedToken::Spaceship);
                Ok(rest)
            }
            [b'=', b'=', rest @ ..] => {
                self.push(PackedToken::Equiv);
                Ok(rest)
            }
            [b'=', rest @ ..] => {
                self.push(PackedToken::Equals);
                Ok(rest)
            }
            [b'!', b'=', rest @ ..] => {
                self.push(PackedToken::NotEquiv);
                Ok(rest)
            }
            [b'>', b'=', rest @ ..] => {
                self.push(PackedToken::GreaterEquals);
                Ok(rest)
            }
            [b'>', rest @ ..] => {
                self.push(PackedToken::Greater);
                Ok(rest)
            }
            [b'<', b'=', rest @ ..] => {
                self.push(PackedToken::LessEquals);
                Ok(rest)
            }
            [b'<', rest @ ..] => {
                self.push(PackedToken::Less);
                Ok(rest)
            }
            [b'|', b'|', rest @ ..] => {
                self.push(PackedToken::OrLogic);
                Ok(rest)
            }
            [b'&', b'&', rest @ ..] => {
                self.push(PackedToken::AndLogic);
                Ok(rest)
            }
            [b'!', rest @ ..] => {
                self.push(PackedToken::NotLogic);
                Ok(rest)
            }
            [b'|', rest @ ..] => {
                self.push(PackedToken::OrBit);
                Ok(rest)
            }
            [b'&', rest @ ..] => {
                self.push(PackedToken::AndBit);
                Ok(rest)
            }
            [b'^', rest @ ..] => {
                self.push(PackedToken::XorBit);
                Ok(rest)
            }
            [b'~', rest @ ..] => {
                self.push(PackedToken::NotBit);
                Ok(rest)
            }
            [b'\'', _, b'\'', rest @ ..] => {
                self.push(PackedToken::Char);
                Ok(rest)
            }
            [b'\'', ..] => Err(UnclosedChar),
            [b'"', rest @ ..] => {
                let closing = rest.iter().position(|&it| it == b'"');
                match closing {
                    Some(pos) => {
                        self.push(PackedToken::String);
                        Ok(&rest[pos + 1..])
                    }
                    None => Err(UnclosedString),
                }
            }
            [b'_', b'A'..=b'Z', rest @ ..] | [b'A'..=b'Z', rest @ ..] => {
                let not_letter = rest.iter().position(|&it| !it.is_ascii_alphanumeric());
                self.push(PackedToken::Type);
                match not_letter {
                    Some(pos) => Ok(&rest[pos..]),
                    None => Ok(&[]),
                }
            }
            [b'_', b'a'..=b'z', rest @ ..] | [b'a'..=b'z', rest @ ..] => {
                let not_letter = rest.iter().position(|&it| !it.is_ascii_alphanumeric());
                self.push(PackedToken::Identifier);
                match not_letter {
                    Some(pos) => Ok(&rest[pos..]),
                    None => Ok(&[]),
                }
            }
            // TODO: support multiple __ in identifiers
            [b'_', rest @ ..] => {
                self.push(PackedToken::TypeGap);
                Ok(rest)
            }
            [b'0', b'b' | b'B', b'0'..b'2', rest @ ..] => {
                self.push(PackedToken::Binary);
                Ok(rest)
            }
            [b'0', b'c' | b'C', b'0'..b'8', rest @ ..] => {
                self.push(PackedToken::Octal);
                Ok(rest)
            }
            [b'0', b'x' | b'X', b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f', rest @ ..] => {
                self.push(PackedToken::Hex);
                Ok(rest)
            }
            [b'0', system @ (b'b' | b'B' | b'c' | b'C' | b'x' | b'X'), first, rest @ ..] => {
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
                        b'b' | b'B' => self.push(PackedToken::Binary),
                        b'c' | b'C' => self.push(PackedToken::Octal),
                        b'x' | b'X' => self.push(PackedToken::Hex),
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
                        self.push(PackedToken::Floating);
                        Ok(&[])
                    }
                    Some(pos) => {
                        let input = &rest[pos..];
                        self.push(PackedToken::Floating);
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
                self.push(PackedToken::Floating);
                match not_digit {
                    None => Ok(&[]),
                    Some(pos) => Ok(&rest[pos..]),
                }
            }
            [b'.', rest @ ..] => {
                self.push(PackedToken::Dot);
                Ok(rest)
            }
            _ => Err(Unknown),
        }
    }
}

fn recover_from_error(input: &[u8], error: Error) -> PackedTokenMatch {
    match error {
        UnclosedChar => PackedTokenMatch {
            token: PackedToken::Char,
            point: CodePoint {
                length: 2,
                offset: 0,
            },
        },
        e @ UnclosedString | e @ NotANumber => {
            let whitespace = input.iter().position(|it| matches!(it, b' ' | b'\t'));
            match whitespace {
                None => PackedTokenMatch {
                    token: PackedToken::Unknown,
                    point: CodePoint {
                        length: input.len() as u32,
                        offset: 0,
                    },
                },
                Some(pos) => PackedTokenMatch {
                    token: if matches!(e, UnclosedString) {
                        PackedToken::String
                    } else {
                        PackedToken::Unknown
                    },
                    point: CodePoint {
                        length: pos as u32,
                        offset: 0,
                    },
                },
            }
        }
        Unknown => PackedTokenMatch {
            token: PackedToken::Unknown,
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
    ) -> Result<PackedTokenMatch, Self::Error<'t>> {
        let input = whole_input[position..].as_bytes();
        let mut token = PackedToken::Unknown;
        let mut sink = Sink(|it| token = it);

        match sink.parse(input) {
            Ok(rest) => {
                let length = rest.as_ptr() as usize - input.as_ptr() as usize;
                Ok(PackedTokenMatch {
                    token,
                    point: CodePoint {
                        length: length as u32,
                        offset: 0,
                    },
                })
            }
            Err(e) => Ok(recover_from_error(input, e))
        }
    }
}

impl EagerTokensProducer for Lexer {
    type Error<'t> = Infallible;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<PackedTokenMatch>, Self::Error<'t>> {
        let mut tokens = vec![];
        let mut offset = 0;
        let current_token = Cell::new(PackedToken::Unknown);
        let mut sink = Sink(|it| current_token.set(it));
        let mut input = input.as_bytes();
        
        while !input.is_empty() {
            match sink.parse(input) {
                Ok(rest) => {
                    let length = rest.as_ptr() as usize - input.as_ptr() as usize;
                    tokens.push(PackedTokenMatch {
                        token: current_token.get(),
                        point: CodePoint {
                            length: length as u32,
                            offset
                        }
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
