use crate::lexer::BitOperator::*;
use crate::lexer::ComparisonOperator::*;
use crate::lexer::Keyword::*;
use crate::lexer::LogicOperator::*;
use crate::lexer::MathOperator::*;
use crate::lexer::Operator::*;
use crate::lexer::Symbol::*;
use crate::lexer::{Identifier, Ignore, Keyword, Literal, Operator, PackedToken, Symbol, Token};

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

impl ToRepresentation for PackedToken {
    #[inline(always)]
    fn representation(&self) -> &'static str {
        match self {
            PackedToken::MultilineComment | PackedToken::Comment => "<comment>",
            PackedToken::Newline => "<newline>",
            PackedToken::Whitespace => "<ws>",
            PackedToken::Fun => "fun",
            PackedToken::Val => "val",
            PackedToken::Var => "var",
            PackedToken::If => "if",
            PackedToken::Elif => "elif",
            PackedToken::Else => "else",
            PackedToken::Match => "match",
            PackedToken::While => "while",
            PackedToken::Module => "module",
            PackedToken::Extend => "extend",
            PackedToken::Lambda => "\\",
            PackedToken::Abstract => "abstract",
            PackedToken::Trait => "trait",
            PackedToken::Struct => "struct",
            PackedToken::Class => "class",
            PackedToken::Enum => "enum",
            PackedToken::Foreign => "foreign",
            PackedToken::TypeAlias => "type",
            PackedToken::With => "with",
            PackedToken::Return => "return",
            PackedToken::Comma => ",",
            PackedToken::Semicolon => ";",
            PackedToken::LBrace => "{",
            PackedToken::RBrace => "}",
            PackedToken::LBracket => "[",
            PackedToken::RBracket => "]",
            PackedToken::LParen => "(",
            PackedToken::RParen => ")",
            PackedToken::TypeGap => "_",
            PackedToken::DoubleColon => "::",
            PackedToken::Colon => ":",
            PackedToken::Identifier => "<ident>",
            PackedToken::Type => "<Ident>",
            PackedToken::Binary => "<binary literal>",
            PackedToken::Octal => "<octal literal>",
            PackedToken::Hex => "<hex literal>",
            PackedToken::Floating => "<number literal>",
            PackedToken::Char => "<char literal>",
            PackedToken::String => "<string literal>",
            PackedToken::Dot => ".",
            PackedToken::Flow => "=>",
            PackedToken::Plus => "+",
            PackedToken::Sub => "-",
            PackedToken::Div => "/",
            PackedToken::Mod => "%",
            PackedToken::Pow => "**",
            PackedToken::Times => "*",
            PackedToken::Equals => "=",
            PackedToken::Equiv => "==",
            PackedToken::NotEquiv => "!=",
            PackedToken::Less => "<",
            PackedToken::LessEquals => "<=",
            PackedToken::Greater => ">",
            PackedToken::GreaterEquals => ">=",
            PackedToken::Spaceship => "<=>",
            PackedToken::OrLogic => "||",
            PackedToken::AndLogic => "&&",
            PackedToken::NotLogic => "!",
            PackedToken::OrBit => "|",
            PackedToken::AndBit => "&",
            PackedToken::XorBit => "^",
            PackedToken::NotBit => "~",
            PackedToken::Unknown => "<???>"
        }
    }
}

impl PackedToken {
    #[inline(always)]
    pub fn from_name(name: &str) -> Option<PackedToken> {
        match name {
            // KEYWORDS
            "fun" => Some(PackedToken::Fun),
            "val" => Some(PackedToken::Val),
            "var" => Some(PackedToken::Var),
            "if" => Some(PackedToken::If),
            "elif" => Some(PackedToken::Elif),
            "else" => Some(PackedToken::Else),
            "match" => Some(PackedToken::Match),
            "while" => Some(PackedToken::While),
            "module" => Some(PackedToken::Module),
            "extend" => Some(PackedToken::Extend),
            "\\" => Some(PackedToken::Lambda),
            "abstract" => Some(PackedToken::Abstract),
            "trait" => Some(PackedToken::Trait),
            "struct" => Some(PackedToken::Struct),
            "class" => Some(PackedToken::Class),
            "enum" => Some(PackedToken::Enum),
            "foreign" => Some(PackedToken::Foreign),
            "type" => Some(PackedToken::TypeAlias),
            "with" => Some(PackedToken::With),
            "return" => Some(PackedToken::Return),
            // SYMBOLS
            "," => Some(PackedToken::Comma),
            ";" => Some(PackedToken::Semicolon),
            "{" => Some(PackedToken::LBrace),
            "}" => Some(PackedToken::RBrace),
            "[" => Some(PackedToken::LBracket),
            "]" => Some(PackedToken::RBracket),
            "(" => Some(PackedToken::LParen),
            ")" => Some(PackedToken::RParen),
            "_" => Some(PackedToken::TypeGap),
            "::" => Some(PackedToken::DoubleColon),
            ":" => Some(PackedToken::Colon),
            // OPERATORS
            "." => Some(PackedToken::Dot),
            "=>" => Some(PackedToken::Flow),
            
            "+" => Some(PackedToken::Plus),
            "-" => Some(PackedToken::Sub),
            "/" => Some(PackedToken::Div),
            "%" => Some(PackedToken::Mod),
            "**" => Some(PackedToken::Pow),
            "*" => Some(PackedToken::Times),

            "=" => Some(PackedToken::Equals),
            "==" => Some(PackedToken::Equiv),
            "!=" => Some(PackedToken::NotEquiv),
            "<" => Some(PackedToken::Less),
            "<=" => Some(PackedToken::LessEquals),
            ">" => Some(PackedToken::Greater),
            ">=" => Some(PackedToken::GreaterEquals),
            "<=>" => Some(PackedToken::Spaceship),

            "||" => Some(PackedToken::OrLogic),
            "&&" => Some(PackedToken::AndLogic),
            "!" => Some(PackedToken::NotLogic),

            "|" => Some(PackedToken::OrBit),
            "&" => Some(PackedToken::AndBit),
            "^" => Some(PackedToken::XorBit),
            "~" => Some(PackedToken::NotBit),
            _ => None,
        }
    }
}

#[cfg(all(test, feature = "enum-iter"))]
mod tests {
    use enum_iterator::{all, Sequence};
    use std::fmt::Debug;

    use crate::lexer::traits::ToRepresentation;
    use crate::lexer::{Keyword, Operator, PackedToken, Symbol, Token};
    use rstest::rstest;

    #[rstest]
    fn test_tokens_have_proper_to_from_repr() {
        for token in all::<PackedToken>() {
            let repr = token.representation();
            let old_token = Token::from_name(repr);
            
            if let Some(old_token) = old_token {
                assert_eq!(PackedToken::from(old_token), token);
            }
            if let Some(Token::Keyword(keyword)) = old_token {
                assert_eq!(keyword.representation(), repr)
            }
            if let Some(Token::Symbol(symbol)) = old_token {
                assert_eq!(symbol.representation(), repr)
            }
            if let Some(Token::Operator(operator)) = old_token {
                assert_eq!(operator.representation(), repr)
            }
        }
    }
}
