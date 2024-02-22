use crate::lexer::{
    BitOperator, ComparisonOperator, Keyword, LogicOperator, MathOperator, Operator, Symbol,
};

pub trait ToRepresentation
    where
        Self: 'static,
{
    fn representation(&self) -> &'static str;
}

impl ToRepresentation for Operator {
    fn representation(&self) -> &'static str {
        match self {
            Operator::Dot => ".",
            Operator::Flow => "=>",
            Operator::Math(x) => match x {
                MathOperator::Plus => "+",
                MathOperator::Sub => "-",
                MathOperator::Div => "/",
                MathOperator::Mod => "%",
                MathOperator::Pow => "**",
                MathOperator::Times => "*",
            },
            Operator::Comparison(x) => match x {
                ComparisonOperator::Equals => "=",
                ComparisonOperator::Equiv => "==",
                ComparisonOperator::NotEquiv => "!=",
                ComparisonOperator::Less => "<",
                ComparisonOperator::LessEquals => "<=",
                ComparisonOperator::Greater => ">",
                ComparisonOperator::GreaterEquals => ">=",
                ComparisonOperator::Spaceship => "<=>",
            },
            Operator::Logic(x) => match x {
                LogicOperator::OrLogic => "||",
                LogicOperator::AndLogic => "&&",
                LogicOperator::NotLogic => "!",
            },
            Operator::Bit(x) => match x {
                BitOperator::OrBit => "|",
                BitOperator::AndBit => "&",
                BitOperator::XorBit => "^",
                BitOperator::NotBit => "~",
            },
        }
    }
}

impl ToRepresentation for Symbol {
    fn representation(&self) -> &'static str {
        match self {
            Symbol::Comma => ",",
            Symbol::Semicolon => ";",
            Symbol::LBrace => "{",
            Symbol::RBrace => "}",
            Symbol::LBracket => "[",
            Symbol::RBracket => "]",
            Symbol::LParen => "(",
            Symbol::RParen => ")",
            Symbol::TypeGap => "_",
            Symbol::DoubleColon => "::",
            Symbol::Colon => ":",
        }
    }
}

impl ToRepresentation for Keyword {
    fn representation(&self) -> &'static str {
        match self {
            Keyword::Fun => "fun",
            Keyword::Val => "val",
            Keyword::Var => "var",
            Keyword::If => "if",
            Keyword::Elif => "elif",
            Keyword::Else => "else",
            Keyword::Match => "match",
            Keyword::While => "while",
            Keyword::Module => "module",
            Keyword::Extend => "extend",
            Keyword::Lambda => "\\",
            Keyword::Abstract => "abstract",
            Keyword::Trait => "trait",
            Keyword::Struct => "struct",
            Keyword::Class => "class",
            Keyword::Enum => "enum",
            Keyword::Foreign => "foreign",
            Keyword::TypeAlias => "type",
            Keyword::With => "with",
            Keyword::Return => "return",
        }
    }
}

#[cfg(all(test, feature = "enum-iter"))]
mod tests {
    use std::fmt::Debug;

    use enum_iterator::{all, Sequence};
    use nom::Parser;
    use nom_supreme::error::ErrorTree;
    use nom_supreme::final_parser::final_parser;
    use rstest::rstest;

    use crate::lexer::traits::ToRepresentation;
    use crate::lexer::*;

    #[rstest]
    #[case(symbol)]
    #[case(operator)]
    #[case(keyword)]
    fn test_lexers<'t, T, P>(#[case] mut parser: P)
        where
            T: Sequence + ToRepresentation + PartialEq + Debug,
            P: Parser<&'t str, T, TokenizationError<'t>>,
    {
        let values = all::<T>().map(|it| {
            let parsed: Result<_, ErrorTree<&str>> =
                final_parser(parser.by_ref())(it.representation());
            (it, parsed)
        });

        for (original, gen) in values {
            match gen {
                Ok(it) => assert_eq!(original, it),
                Err(e) => panic!("For input `{original:?}` {e}"),
            };
        }
    }
}
