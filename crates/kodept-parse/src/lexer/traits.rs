use crate::lexer::PackedToken;

pub trait ToRepresentation {
    fn representation(&self) -> &'static str;
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
            PackedToken::Unknown => "<???>",
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
    use enum_iterator::all;

    use crate::lexer::traits::ToRepresentation;
    use crate::lexer::PackedToken;
    use rstest::rstest;

    #[rstest]
    fn test_tokens_have_proper_to_from_repr() {
        for token in all::<PackedToken>() {
            let repr = token.representation();
            let old_token = PackedToken::from_name(repr);

            if let Some(old_token) = old_token {
                assert_eq!(old_token, token);
            }
        }
    }
}
