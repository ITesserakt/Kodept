use crate::lexer::BitOperator::*;
use crate::lexer::ComparisonOperator::*;
use crate::lexer::Keyword::*;
use crate::lexer::LogicOperator::*;
use crate::lexer::MathOperator::*;
use crate::lexer::Operator::*;
use crate::lexer::Symbol::*;
use crate::lexer::{Identifier, Ignore, Keyword, Literal, Operator, Symbol, Token};

pub trait ToRepresentation {
    fn representation(&self) -> &'static str;
}

impl ToRepresentation for Operator {
    fn representation(&self) -> &'static str {
        match self {
            Dot => ".",
            Flow => "=>",
            Math(x) => match x {
                Plus => "+",
                Sub => "-",
                Div => "/",
                Mod => "%",
                Pow => "**",
                Times => "*",
            },
            Comparison(x) => match x {
                Equals => "=",
                Equiv => "==",
                NotEquiv => "!=",
                Less => "<",
                LessEquals => "<=",
                Greater => ">",
                GreaterEquals => ">=",
                Spaceship => "<=>",
            },
            Logic(x) => match x {
                OrLogic => "||",
                AndLogic => "&&",
                NotLogic => "!",
            },
            Bit(x) => match x {
                OrBit => "|",
                AndBit => "&",
                XorBit => "^",
                NotBit => "~",
            },
        }
    }
}

impl ToRepresentation for Symbol {
    fn representation(&self) -> &'static str {
        match self {
            Comma => ",",
            Semicolon => ";",
            LBrace => "{",
            RBrace => "}",
            LBracket => "[",
            RBracket => "]",
            LParen => "(",
            RParen => ")",
            TypeGap => "_",
            DoubleColon => "::",
            Colon => ":",
        }
    }
}

impl ToRepresentation for Keyword {
    fn representation(&self) -> &'static str {
        match self {
            Fun => "fun",
            Val => "val",
            Var => "var",
            If => "if",
            Elif => "elif",
            Else => "else",
            Match => "match",
            While => "while",
            Module => "module",
            Extend => "extend",
            Lambda => "\\",
            Abstract => "abstract",
            Trait => "trait",
            Struct => "struct",
            Class => "class",
            Enum => "enum",
            Foreign => "foreign",
            TypeAlias => "type",
            With => "with",
            Return => "return",
        }
    }
}

impl<'t> Token<'t> {
    #[inline(always)]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            // KEYWORDS
            "fun" => Some(Token::Keyword(Fun)),
            "val" => Some(Token::Keyword(Val)),
            "var" => Some(Token::Keyword(Var)),
            "if" => Some(Token::Keyword(If)),
            "elif" => Some(Token::Keyword(Elif)),
            "else" => Some(Token::Keyword(Else)),
            "match" => Some(Token::Keyword(Match)),
            "while" => Some(Token::Keyword(While)),
            "module" => Some(Token::Keyword(Module)),
            "extend" => Some(Token::Keyword(Extend)),
            "\\" => Some(Token::Keyword(Lambda)),
            "abstract" => Some(Token::Keyword(Abstract)),
            "trait" => Some(Token::Keyword(Trait)),
            "struct" => Some(Token::Keyword(Struct)),
            "class" => Some(Token::Keyword(Class)),
            "enum" => Some(Token::Keyword(Enum)),
            "foreign" => Some(Token::Keyword(Foreign)),
            "type" => Some(Token::Keyword(TypeAlias)),
            "with" => Some(Token::Keyword(With)),
            "return" => Some(Token::Keyword(Return)),
            // SYMBOLS
            "," => Some(Token::Symbol(Comma)),
            ";" => Some(Token::Symbol(Semicolon)),
            "{" => Some(Token::Symbol(LBrace)),
            "}" => Some(Token::Symbol(RBrace)),
            "[" => Some(Token::Symbol(LBracket)),
            "]" => Some(Token::Symbol(RBracket)),
            "(" => Some(Token::Symbol(LParen)),
            ")" => Some(Token::Symbol(RParen)),
            "_" => Some(Token::Symbol(TypeGap)),
            "::" => Some(Token::Symbol(DoubleColon)),
            ":" => Some(Token::Symbol(Colon)),
            // OPERATORS
            "." => Some(Token::Operator(Dot)),
            "=>" => Some(Token::Operator(Flow)),
            "+" => Some(Token::Operator(Math(Plus))),
            "-" => Some(Token::Operator(Math(Sub))),
            "/" => Some(Token::Operator(Math(Div))),
            "%" => Some(Token::Operator(Math(Mod))),
            "**" => Some(Token::Operator(Math(Pow))),
            "*" => Some(Token::Operator(Math(Times))),

            "=" => Some(Token::Operator(Comparison(Equals))),
            "==" => Some(Token::Operator(Comparison(Equiv))),
            "!=" => Some(Token::Operator(Comparison(NotEquiv))),
            "<" => Some(Token::Operator(Comparison(Less))),
            "<=" => Some(Token::Operator(Comparison(LessEquals))),
            ">" => Some(Token::Operator(Comparison(Greater))),
            ">=" => Some(Token::Operator(Comparison(GreaterEquals))),
            "<=>" => Some(Token::Operator(Comparison(Spaceship))),

            "||" => Some(Token::Operator(Logic(OrLogic))),
            "&&" => Some(Token::Operator(Logic(AndLogic))),
            "!" => Some(Token::Operator(Logic(NotLogic))),

            "|" => Some(Token::Operator(Bit(OrBit))),
            "&" => Some(Token::Operator(Bit(AndBit))),
            "^" => Some(Token::Operator(Bit(XorBit))),
            "~" => Some(Token::Operator(Bit(NotBit))),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn to_name(&self) -> &'static str {
        match self {
            Token::Keyword(x) => x.representation(),
            Token::Symbol(x) => x.representation(),
            Token::Operator(x) => x.representation(),
            Token::Unknown => "<???>",
            
            Token::Ignore(Ignore::Comment(_)) => "<comment>",
            Token::Ignore(Ignore::MultilineComment(_)) => "<comment>",
            Token::Ignore(Ignore::Newline) => "<newline>",
            Token::Ignore(Ignore::Whitespace) => "<ws>",
            
            Token::Literal(Literal::String(_)) => "<string literal>",
            Token::Literal(Literal::Char(_)) => "<char literal>",
            Token::Literal(Literal::Binary(_)) => "<binary literal>",
            Token::Literal(Literal::Octal(_)) => "<octal literal>",
            Token::Literal(Literal::Hex(_)) => "<hex literal>",
            Token::Literal(Literal::Floating(_)) => "<number literal>",
            
            Token::Identifier(Identifier::Identifier(_)) => "<ident>",
            Token::Identifier(Identifier::Type(_)) => "<Ident>"
        }
    }
}

#[cfg(all(test, feature = "enum-iter"))]
mod tests {
    use enum_iterator::{all, Sequence};
    use std::fmt::Debug;

    use crate::lexer::traits::ToRepresentation;
    use crate::lexer::{Keyword, Operator, Symbol, Token};
    use rstest::rstest;

    #[rstest]
    #[case(Keyword::Struct)]
    #[case(Symbol::Comma)]
    #[case(Operator::Dot)]
    fn test_lexers<T>(#[case] _example: T)
    where
        T: Sequence + ToRepresentation + PartialEq + Debug + for<'a> Into<Token<'a>>,
    {
        let values = all::<T>().map(|it| {
            let repr = it.representation();
            (it, Token::from_name(repr).unwrap())
        });

        for (original, gen) in values {
            assert_eq!(gen, original.into());
        }
    }
}
