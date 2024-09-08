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
    use enum_iterator::{all, Sequence};
    use std::fmt::Debug;

    use crate::common::TokenProducer;
    use crate::lexer::traits::ToRepresentation;
    use crate::lexer::{Keyword, Operator, PestLexer, Symbol, Token};
    use rstest::rstest;

    #[rstest]
    #[case(Keyword::Struct)]
    #[case(Symbol::Comma)]
    #[case(Operator::Dot)]
    fn test_lexers<T>(#[case] _example: T)
    where
        T: Sequence + ToRepresentation + PartialEq + Debug + for<'a> Into<Token<'a>>
    {
        let values = all::<T>().map(|it| {
            let parsed = PestLexer::new().parse_string(it.representation(), 0);
            (it, parsed)
        });

        for (original, gen) in values {
            match gen {
                Ok(it) => assert_eq!(original.into(), it.token),
                Err(e) => panic!("For input `{original:?}` {e}"),
            };
        }
    }
}
