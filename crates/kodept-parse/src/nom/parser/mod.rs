use crate::common::RLTProducer;
use crate::nom::error::ExpectedError;
use crate::nom::TokenVerificationError;
use crate::token_stream::PackedTokenStream;
use derive_more::{Constructor, Display};
use kodept_core::structure::rlt::RLT;
use nom::error::{ContextError, ErrorKind, FromExternalError};
use ::nom::IResult;
use nom::{Needed, Parser as ParserOps};
use std::borrow::Cow;
use std::fmt::Formatter;

type PResult<'t, O> = IResult<PackedTokenStream<'t>, O, PError<'t>>;

trait PParser<'t, O = PackedTokenStream<'t>>:
    nom::Parser<PackedTokenStream<'t>, Output = O, Error = PError<'t>>
{
}
impl<'t, O, P: nom::Parser<PackedTokenStream<'t>, Output = O, Error = PError<'t>>> PParser<'t, O>
    for P
{
}

#[derive(Debug, Clone, PartialEq)]
pub enum PError<'t, E = TokenVerificationError> {
    Base {
        location: PackedTokenStream<'t>,
        kind: PErrorKind<E>,
    },
    Stack {
        base: Box<Self>,
        contexts: Vec<(PackedTokenStream<'t>, PErrorContext)>,
    },
    Alt(Vec<Self>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PErrorKind<E> {
    Nom(ErrorKind),
    External(E),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PErrorContext {
    Nom(ErrorKind),
    Context(&'static str),
}

impl Display for PErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PErrorContext::Nom(kind) => write!(f, "while parsing {}", kind.description()),
            PErrorContext::Context(ctx) => write!(f, "in section {}", ctx),
        }
    }
}

impl<'t, E> PError<'t, E> {
    pub fn map<F>(self, f: impl Fn(E) -> F + Copy) -> PError<'t, F> {
        match self {
            PError::Base { location, kind } => PError::Base {
                location,
                kind: match kind {
                    PErrorKind::Nom(x) => PErrorKind::Nom(x),
                    PErrorKind::External(e) => PErrorKind::External(f(e)),
                    PErrorKind::Char(x) => PErrorKind::Char(x),
                },
            },
            PError::Stack { base, contexts } => PError::Stack {
                base: Box::new(base.map(f)),
                contexts,
            },
            PError::Alt(vs) => PError::Alt(vs.into_iter().map(|it| it.map(f)).collect()),
        }
    }
}

impl<'t, E> FromExternalError<PackedTokenStream<'t>, E> for PError<'t, E> {
    fn from_external_error(input: PackedTokenStream<'t>, kind: ErrorKind, e: E) -> Self {
        let base = Self::Base {
            location: input,
            kind: PErrorKind::External(e),
        };
        Self::Stack {
            base: Box::new(base),
            contexts: vec![(input, PErrorContext::Nom(kind))],
        }
    }
}

impl<'t, E> ContextError<PackedTokenStream<'t>> for PError<'t, E> {
    fn add_context(input: PackedTokenStream<'t>, ctx: &'static str, other: Self) -> Self {
        Self::Stack {
            base: Box::new(other),
            contexts: vec![(input, PErrorContext::Context(ctx))],
        }
    }
}

impl<'t, E> nom::error::ParseError<PackedTokenStream<'t>> for PError<'t, E> {
    fn from_error_kind(input: PackedTokenStream<'t>, kind: ErrorKind) -> Self {
        PError::Base {
            location: input,
            kind: PErrorKind::Nom(kind),
        }
    }

    fn append(input: PackedTokenStream<'t>, kind: ErrorKind, other: Self) -> Self {
        Self::Stack {
            base: Box::new(other),
            contexts: vec![(input, PErrorContext::Nom(kind))],
        }
    }

    fn from_char(input: PackedTokenStream<'t>, value: char) -> Self {
        Self::Base {
            location: input,
            kind: PErrorKind::Char(value),
        }
    }

    fn or(self, other: Self) -> Self {
        let errors = match (self, other) {
            (PError::Alt(mut es1), PError::Alt(es2)) => {
                es1.extend(es2);
                es1
            }
            (PError::Alt(mut es), e) | (e, PError::Alt(mut es)) => {
                es.push(e);
                es
            }
            (e1, e2) => vec![e1, e2],
        };
        PError::Alt(errors)
    }
}

mod block_level;
mod code_flow;
mod expression;
mod file;
mod function;
mod literal;
mod operator;
mod parameter;
mod term;
mod top_level;
mod r#type;
mod utils;

#[derive(Constructor, Debug)]
pub struct Parser;

#[derive(Debug)]
pub enum ParserError {
    Token(TokenVerificationError),
    Incomplete(Needed),
}

impl ExpectedError for ParserError {
    fn expected(&self) -> Cow<'static, str> {
        match self {
            ParserError::Token(x) => x.expected(),
            ParserError::Incomplete(Needed::Unknown) => Cow::Borrowed("not EOF"),
            ParserError::Incomplete(Needed::Size(n)) if n.get() == 1 => Cow::Borrowed("1 more token"),
            ParserError::Incomplete(Needed::Size(n)) => Cow::Owned(format!("{} more tokens", n)),
        }
    }
}

impl RLTProducer for Parser {
    type Error<'t> = PError<'t, ParserError>;

    fn parse_stream<'t>(&self, input: &PackedTokenStream<'t>) -> Result<RLT, Self::Error<'t>> {
        match file::grammar().parse_complete(*input) {
            Ok((rest, file)) => match &*rest {
                [] => Ok(RLT(file)),
                _ => Err(PError::Base {
                    location: rest,
                    kind: PErrorKind::External(ParserError::Incomplete(Needed::Unknown)),
                }),
            },
            Err(nom::Err::Incomplete(n)) => Err(PError::Base {
                location: input.sub_stream(input.len()..),
                kind: PErrorKind::External(ParserError::Incomplete(n)),
            }),
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(e.map(ParserError::Token)),
        }
    }
}

mod macros {
    // TODO: Make it const as early as possible
    macro_rules! function {
        () => {{
            const fn f() {}
            let name = std::any::type_name_of_val(&f);
            &name[..name.len() - 3]
        }};
    }

    pub(crate) use function;
}
