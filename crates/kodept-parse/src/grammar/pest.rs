use crate::lexer::{
    BitOperator, ComparisonOperator, Identifier, Keyword, Literal, LogicOperator, MathOperator,
    Operator, Symbol, Token,
};
use crate::token_match::TokenMatch;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::Span;
use pest_typed::{ParsableTypedNode, Spanned};
use pest_typed_derive::{match_choices, TypedParser};

#[derive(TypedParser)]
#[grammar = "grammar/kodept.pest"]
pub struct Tokenizer<'t> {
    buffer: &'t str,
    tokens: Vec<TokenMatch<'t>>,
    index: usize
}

impl<'t> Tokenizer<'t> {
    pub fn new(input: &'t str) -> Self {
        let mut this = Self {
            buffer: input,
            tokens: vec![],
            index: 0
        };
        this.parse_tokens();
        this.remove_quotes();
        this
    }
    
    fn remove_quotes(&mut self) {
        self.tokens.iter_mut().for_each(|TokenMatch { token, .. }| {
            if let Token::Literal(Literal::Char(s) | Literal::String(s)) = token {
                *s = s.trim_matches(['"', '\'']);
            }
        });
    }
    
    fn parse_tokens(&mut self) {
        let tokens = match pairs::tokens::try_parse(self.buffer) {
            Ok(t) => t.content.content.1.matched,
            Err(_) => return
        };
        
        self.tokens = tokens.into_iter_matched()
            .map(Self::parse_one_token)
            .map(|(token, span)| {
                let matched_length = span.as_str().len();
                TokenMatch::new(
                    token,
                    Span::new(CodePoint::new(matched_length, span.start())),
                )
            }).collect();
    }

    fn parse_one_token(token: rules::token) -> (Token, pest_typed::Span) {
        match_choices!(token.content.as_ref() {
            keyword => (Token::Keyword(match_choices!(keyword.content.as_ref() {
                _fun => Keyword::Fun,
                _val => Keyword::Val,
                _var => Keyword::Var,
                _match_ => Keyword::Match,
                _while_ => Keyword::While,
                _module => Keyword::Module,
                _extend => Keyword::Extend,
                _return_ => Keyword::Return,
                _back => Keyword::Lambda,
                _if_ => Keyword::If,
                _elif => Keyword::Elif,
                _else_ => Keyword::Else,
                _abstract_ => Keyword::Abstract,
                _trait_ => Keyword::Trait,
                _struct_ => Keyword::Struct,
                _class => Keyword::Class,
                _enum_ => Keyword::Enum,
                _foreign => Keyword::Foreign,
                _type_ => Keyword::TypeAlias,
                _with => Keyword::With
            })), keyword.span()),
            symbol => (Token::Symbol(match_choices!(symbol.content.as_ref() {
                _comma => Symbol::Comma,
                _semicolon => Symbol::Semicolon,
                _lbrace => Symbol::LBrace,
                _rbrace => Symbol::RBrace,
                _lbracket => Symbol::LBracket,
                _rbracket => Symbol::RBracket,
                _lparen => Symbol::LParen,
                _rparen => Symbol::RParen,
                _under => Symbol::TypeGap,
                _double => Symbol::DoubleColon,
                _colon => Symbol::Colon
            })), symbol.span()),
            identifier => {
                let (_, start, _) = identifier.as_ref();
                let input = identifier.span().as_str();
                let ty = match_choices!(start {
                    _lower => Identifier::Identifier(input),
                    _upper => Identifier::Type(input)
                });
                (Token::Identifier(ty), identifier.span())
            },
            literal => {
                let input = literal.span().as_str();
                match_choices!(literal.content.as_ref() {
                    _binary => (Token::Literal(Literal::Binary(input)), literal.span()),
                    _octal => (Token::Literal(Literal::Octal(input)), literal.span()),
                    _hex => (Token::Literal(Literal::Hex(input)), literal.span()),
                    _floating => (Token::Literal(Literal::Floating(input)), literal.span()),
                    _char => {
                        let span = literal.span();
                        (Token::Literal(Literal::Char(span.as_str())), span)
                    },
                    _string => {
                        let span = literal.span();
                        (Token::Literal(Literal::String(span.as_str())), span)
                    }
                })
            },
            operator => (Token::Operator(match_choices!(operator.content.as_ref() {
                _dot => Operator::Dot,
                _flow => Operator::Flow,
                _plus => Operator::Math(MathOperator::Plus),
                _minus => Operator::Math(MathOperator::Sub),
                _pow => Operator::Math(MathOperator::Pow),
                _times => Operator::Math(MathOperator::Times),
                _div => Operator::Math(MathOperator::Div),
                _mod_ => Operator::Math(MathOperator::Mod),
                _spaceship => Operator::Comparison(ComparisonOperator::Spaceship),
                _equiv => Operator::Comparison(ComparisonOperator::Equiv),
                _equals => Operator::Comparison(ComparisonOperator::Equals),
                _not_equiv => Operator::Comparison(ComparisonOperator::NotEquiv),
                _greater_eq => Operator::Comparison(ComparisonOperator::GreaterEquals),
                _greater => Operator::Comparison(ComparisonOperator::Greater),
                _less_eq => Operator::Comparison(ComparisonOperator::LessEquals),
                _less => Operator::Comparison(ComparisonOperator::Less),
                _or => Operator::Logic(LogicOperator::OrLogic),
                _and => Operator::Logic(LogicOperator::AndLogic),
                _not => Operator::Logic(LogicOperator::NotLogic),
                _disj => Operator::Bit(BitOperator::OrBit),
                _conj => Operator::Bit(BitOperator::AndBit),
                _xor => Operator::Bit(BitOperator::XorBit),
                _nor => Operator::Bit(BitOperator::NotBit)
            })), operator.span()),
            unknown => (Token::Unknown, unknown.span()),
        })
    }

    pub fn into_vec(mut self) -> Vec<TokenMatch<'t>> {
        self.tokens.shrink_to_fit();
        self.tokens
    }
}

impl<'t> Iterator for Tokenizer<'t> {
    type Item = TokenMatch<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tokens.len() {
            None
        } else {
            self.index += 1;
            Some(self.tokens[self.index - 1])
        }
    }
}
