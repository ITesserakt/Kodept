use crate::lexer::Token;

pub trait ToRepresentation {
    fn representation(&self) -> &'static str;
}

impl ToRepresentation for Token {
    #[inline(always)]
    fn representation(&self) -> &'static str {
        match self {
            Token::MultilineComment | Token::Comment => "<comment>",
            Token::Newline => "<newline>",
            Token::Whitespace => "<ws>",
            Token::Fun => "fun",
            Token::Val => "val",
            Token::Var => "var",
            Token::If => "if",
            Token::Elif => "elif",
            Token::Else => "else",
            Token::Match => "match",
            Token::While => "while",
            Token::Module => "module",
            Token::Extend => "extend",
            Token::Abstract => "abstract",
            Token::Trait => "trait",
            Token::Struct => "struct",
            Token::Class => "class",
            Token::Enum => "enum",
            Token::Foreign => "foreign",
            Token::TypeAlias => "type",
            Token::With => "with",
            Token::Return => "return",
            Token::Comma => ",",
            Token::Semicolon => ";",
            Token::LBrace => "{",
            Token::RBrace => "}",
            Token::LBracket => "[",
            Token::RBracket => "]",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::TypeGap => "_",
            Token::DoubleColon => "::",
            Token::Colon => ":",
            Token::Identifier => "<ident>",
            Token::Type => "<Ident>",
            Token::Binary => "<binary literal>",
            Token::Octal => "<octal literal>",
            Token::Hex => "<hex literal>",
            Token::Floating => "<number literal>",
            Token::Char => "<char literal>",
            Token::String => "<string literal>",
            Token::Dot => ".",
            Token::Flow => "=>",
            Token::Plus => "+",
            Token::Sub => "-",
            Token::Div => "/",
            Token::Mod => "%",
            Token::Pow => "**",
            Token::Times => "*",
            Token::Equals => "=",
            Token::Equiv => "==",
            Token::NotEquiv => "!=",
            Token::Less => "<",
            Token::LessEquals => "<=",
            Token::Greater => ">",
            Token::GreaterEquals => ">=",
            Token::Spaceship => "<=>",
            Token::OrLogic => "||",
            Token::AndLogic => "&&",
            Token::NotLogic => "!",
            Token::OrBit => "|",
            Token::AndBit => "&",
            Token::XorBit => "^",
            Token::NotBit => "~",
            Token::Unknown => "<???>",
        }
    }
}

impl Token {
    #[inline(always)]
    pub fn from_name(name: &str) -> Option<Token> {
        match name {
            // KEYWORDS
            "fun" => Some(Token::Fun),
            "val" => Some(Token::Val),
            "var" => Some(Token::Var),
            "if" => Some(Token::If),
            "elif" => Some(Token::Elif),
            "else" => Some(Token::Else),
            "match" => Some(Token::Match),
            "while" => Some(Token::While),
            "module" => Some(Token::Module),
            "extend" => Some(Token::Extend),
            "abstract" => Some(Token::Abstract),
            "trait" => Some(Token::Trait),
            "struct" => Some(Token::Struct),
            "class" => Some(Token::Class),
            "enum" => Some(Token::Enum),
            "foreign" => Some(Token::Foreign),
            "type" => Some(Token::TypeAlias),
            "with" => Some(Token::With),
            "return" => Some(Token::Return),
            // SYMBOLS
            "," => Some(Token::Comma),
            ";" => Some(Token::Semicolon),
            "{" => Some(Token::LBrace),
            "}" => Some(Token::RBrace),
            "[" => Some(Token::LBracket),
            "]" => Some(Token::RBracket),
            "(" => Some(Token::LParen),
            ")" => Some(Token::RParen),
            "_" => Some(Token::TypeGap),
            "::" => Some(Token::DoubleColon),
            ":" => Some(Token::Colon),
            // OPERATORS
            "." => Some(Token::Dot),
            "=>" => Some(Token::Flow),

            "+" => Some(Token::Plus),
            "-" => Some(Token::Sub),
            "/" => Some(Token::Div),
            "%" => Some(Token::Mod),
            "**" => Some(Token::Pow),
            "*" => Some(Token::Times),

            "=" => Some(Token::Equals),
            "==" => Some(Token::Equiv),
            "!=" => Some(Token::NotEquiv),
            "<" => Some(Token::Less),
            "<=" => Some(Token::LessEquals),
            ">" => Some(Token::Greater),
            ">=" => Some(Token::GreaterEquals),
            "<=>" => Some(Token::Spaceship),

            "||" => Some(Token::OrLogic),
            "&&" => Some(Token::AndLogic),
            "!" => Some(Token::NotLogic),

            "|" => Some(Token::OrBit),
            "&" => Some(Token::AndBit),
            "^" => Some(Token::XorBit),
            "~" => Some(Token::NotBit),
            _ => None,
        }
    }
}

#[cfg(all(test, feature = "enum-iter"))]
mod tests {
    use enum_iterator::all;

    use crate::lexer::Token;
    use crate::lexer::traits::ToRepresentation;
    use rstest::rstest;

    #[rstest]
    fn test_tokens_have_proper_to_from_repr() {
        for token in all::<Token>() {
            let repr = token.representation();
            let old_token = Token::from_name(repr);

            if let Some(old_token) = old_token {
                assert_eq!(old_token, token);
            }
        }
    }
}
