pub use grammar::kodept;
use kodept_core::structure::*;
use kodept_core::structure::rlt::new_types::BinaryOperationSymbol;
use kodept_core::structure::rlt::new_types::UnaryOperationSymbol;
use kodept_core::structure::rlt::new_types::Keyword;
use kodept_core::structure::rlt::new_types::Symbol;

use crate::grammar::macros::tok;
use crate::lexer::{Identifier as I, Token};
use crate::lexer::Literal::*;
use crate::lexer::Token::*;
use crate::lexer::Symbol::Semicolon;
use crate::lexer::Ignore::{Newline};
use crate::OptionTExt;
use crate::parser::common::VerboseEnclosed;
use crate::token_match::TokenMatch;
use crate::token_stream::TokenStream;

peg::parser! {grammar grammar<'t>() for TokenStream<'t> {
    /// UTILITIES
    /// --------------------------------------------------------------------------------------------
    #[cache]
    rule _ = quiet! { [tok!(Ignore(_))]* }

    rule comma_separated0<T>(items: rule<T>) -> Vec<T> =
        i:(items() ** (_ "," _)) _ ","? { i }

    rule paren_enclosed<T>(inner: rule<T>) -> VerboseEnclosed<T> =
        lp:$"(" _ i:inner() _ rp:$")" { VerboseEnclosed::from_located(lp, i, rp) }

    rule brace_enclosed<T>(inner: rule<T>) -> VerboseEnclosed<T> =
        lp:$"{" _ i:inner() _ rp:$"}" { VerboseEnclosed::from_located(lp, i, rp) }

    #[cache]
    rule separation() = 
        (quiet!{ [tok!(Ignore(Newline))]+ } / expected!("<newline>")) _ /
        (quiet!{ [tok!(Token::Symbol(Semicolon))] } / expected!(";")) _ 
    
    rule separated<T>(inner: rule<T>) -> Vec<T> =
        inner() ** separation()

    rule ident() -> TokenMatch<'input> =
        quiet!{ [tok!(Identifier(I::Identifier(_)))] } / expected!("<ident>")

    rule type_ident() -> rlt::new_types::TypeName =
        i:(quiet!{ [tok!(Identifier(I::Type(_)))] } / expected!("<Ident>")) {
            rlt::new_types::TypeName::from(i.span)
        }

    /// Type grammar
    /// --------------------------------------------------------------------------------------------

    rule return_type() -> (Symbol, rlt::Type) =
        c:$":" _ ty:type_grammar() { (Symbol::from_located(c), ty) }

    rule tuple() -> rlt::Type =
        i:paren_enclosed(<comma_separated0(<type_grammar()>)>) { rlt::Type::Tuple(i.into()) }

    pub rule type_grammar() -> rlt::Type =
        i:type_ident() { rlt::Type::Reference(i) } /
        tuple()

    /// Parameters grammar
    /// --------------------------------------------------------------------------------------------

    pub rule typed_parameter() -> rlt::TypedParameter =
        i:ident() _ ":" _ t:type_grammar() {
            rlt::TypedParameter {  id: i.span.into(), parameter_type: t}
        }

    pub rule untyped_parameter() -> rlt::UntypedParameter =
        i:ident() _ (":" _ "_")? {
            rlt::UntypedParameter { id: i.span.into() }
        }

    pub rule parameter() -> rlt::Parameter =
        t:typed_parameter() { rlt::Parameter::Typed(t) } /
        u:untyped_parameter() { rlt::Parameter::Untyped(u) }

    /// Literals grammar
    /// --------------------------------------------------------------------------------------------

    rule lit<T>(inner: rule<T>, name: &'static str) -> T =
        quiet!{ inner() } / expected!(name)

    pub rule literal_grammar() -> rlt::Literal =
        i:lit(<[tok!(Literal(Binary(_)))]>,   "<binary literal>") { rlt::Literal::Binary(i.span) }   /
        i:lit(<[tok!(Literal(Octal(_)))]>,    "<octal literal>")  { rlt::Literal::Octal(i.span) }    /
        i:lit(<[tok!(Literal(Hex(_)))]>,      "<hex literal>")    { rlt::Literal::Hex(i.span) }      /
        i:lit(<[tok!(Literal(Floating(_)))]>, "<number literal>") { rlt::Literal::Floating(i.span) } /
        i:lit(<[tok!(Literal(Char(_)))]>,     "<char literal>")   { rlt::Literal::Char(i.span) }     /
        i:lit(<[tok!(Literal(String(_)))]>,   "<string literal>") { rlt::Literal::String(i.span) }

    /// Operators grammar
    /// --------------------------------------------------------------------------------------------

    pub rule operator_grammar() -> rlt::Operation = precedence! {
        a:@ _ op:$"=" _ b:(@) { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Assign(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        a:(@) _ op:$"||" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Logic(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$"&&" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Logic(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        a:(@) _ op:$"|" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Bit(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$"&" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Bit(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$"^" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Bit(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        a:(@) _ op:$"<" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Comparison(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$">" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Comparison(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        a:(@) _ op:$"<=" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$"!=" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$"==" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$">=" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        a:(@) _ op:$"<=>" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::ComplexComparison(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        a:(@) _ op:$"+" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Add(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$"-" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Add(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        a:(@) _ op:$"*" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Mul(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$"/" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Mul(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        a:(@) _ op:$"%" _ b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Mul(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        a:@ _ op:$"**" _ b:(@) { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Pow(Symbol::from_located(op)),
            right: Box::new(b)
        } }
        --
        op:$"-" _ a:@ { rlt::Operation::TopUnary {
            operator: UnaryOperationSymbol::Neg(Symbol::from_located(op)),
            expr: Box::new(a)
        } }
        op:$"!" _ a:@ { rlt::Operation::TopUnary {
            operator: UnaryOperationSymbol::Not(Symbol::from_located(op)),
            expr: Box::new(a)
        } }
        op:$"~" _ a:@ { rlt::Operation::TopUnary {
            operator: UnaryOperationSymbol::Inv(Symbol::from_located(op)),
            expr: Box::new(a)
        } }
        op:$"+" _ a:@ { rlt::Operation::TopUnary {
            operator: UnaryOperationSymbol::Plus(Symbol::from_located(op)),
            expr: Box::new(a)
        } }
        --
        a:(@) _ op:$"." _ b:@ { rlt::Operation::Access {
            left: Box::new(a),
            dot: Symbol::from_located(op),
            right: Box::new(b)
        } }
        --
        i:application() { i }
        i:atom()        { i }
    }

    #[cache]
    rule atom() -> rlt::Operation =
        i:expression_grammar()                                             { rlt::Operation::Expression(i) } /
        i:paren_enclosed(<comma_separated0(<operator_grammar()>)>) {
            if i.inner.len() == 1 {
                let mut vec = i.inner;
                vec.pop().unwrap()
            } else {
                rlt::Operation::Expression(rlt::Expression::Literal(rlt::Literal::Tuple(i.into())))
            }
        } /
        i:block()                                                          { rlt::Operation::Block(i) }

    rule application() -> rlt::Operation =
        a:atom() b:paren_enclosed(<comma_separated0(<operator_grammar()>)>)? { match b {
            None => a,
            Some(params) =>
                rlt::Operation::Application(Box::new(rlt::Application {
                    expr: a,
                    params: Some(params.into())
                }))
            }
        }

    /// Expressions grammar
    /// --------------------------------------------------------------------------------------------

    rule lambda() -> rlt::Expression =
        l:$"\\" _ ps:comma_separated0(<parameter()>) _ f:$"=>" _ expr:operator_grammar() {
        rlt::Expression::Lambda {
            keyword: Keyword::from_located(l),
            binds: ps.into_boxed_slice(),
            flow: Symbol::from_located(f),
            expr: Box::new(expr)
        }
    }

    pub rule expression_grammar() -> rlt::Expression =
        lambda()                                                   /
        i:term_grammar()      { rlt::Expression::Term(i) }         /
        i:literal_grammar()   { rlt::Expression::Literal(i) }      /
        i:code_flow_grammar() { rlt::Expression::If(Box::new(i)) }

    /// References grammar
    /// --------------------------------------------------------------------------------------------

    /// |      | Global   | Local     |
    /// | ---- | -------- | --------- |
    /// | Type | ::{X::}X | X::X{::X} |
    /// | Ref  | ::{X::}x | X::{X::}x |

    rule type_ref() -> rlt::Reference = t:type_ident() { rlt::Reference::Type(t) }

    rule variable_ref() -> rlt::Reference =
        t:ident() { rlt::Reference::Identifier(t.span.into()) }

    rule ref() -> rlt::Reference =
        variable_ref() /
        type_ref()

    rule global_type_ref() -> (rlt::Context, rlt::Reference) =
        g:$"::" ctx:(type_ref() ++ "::") {
            let start = rlt::Context::Global {
                colon: Symbol::from_located(g)
            };
            let mut ctx = ctx;
            let last = ctx.pop().unwrap();
            let context = ctx.into_iter().fold(start, |acc, next| rlt::Context::Inner {
                parent: Box::new(acc),
                needle: next
            });
            (context, last)
        }

    rule global_ref() -> (rlt::Context, rlt::Reference) =
        g:$"::" ctx:(type_ref() ++ (!("::" variable_ref()) "::")) "::" v:variable_ref() {
            let start = rlt::Context::Global {
                colon: Symbol::from_located(g)
            };
            let context = ctx.into_iter().fold(start, |acc, next| rlt::Context::Inner {
                parent: Box::new(acc),
                needle: next
            });
            (context, v)
        }

    rule local_type_ref() -> (rlt::Context, rlt::Reference) =
        ctx:(type_ref() **<2,> "::") {
            let start = rlt::Context::Local;
            let mut ctx = ctx;
            let last = ctx.pop().unwrap();
            let context = ctx.into_iter().fold(start, |acc, next| rlt::Context::Inner {
                parent: Box::new(acc),
                needle: next
            });
            (context, last)
        }

    rule local_ref() -> (rlt::Context, rlt::Reference) =
        ctx:(type_ref() ++ (!("::" variable_ref()) "::")) "::" v:variable_ref() {
            let start = rlt::Context::Local;
            let context = ctx.into_iter().fold(start, |acc, next| rlt::Context::Inner {
                parent: Box::new(acc),
                needle: next
            });
            (context, v)
        }

    rule contextual() -> rlt::ContextualReference = i:(
        global_ref()      /
        global_type_ref() /
        local_ref()       /
        local_type_ref()
    ) { rlt::ContextualReference {
        context: i.0,
        inner: i.1
    } }

    pub rule term_grammar() -> rlt::Term =
        i:contextual() { rlt::Term::Contextual(i) } /
        i:ref()        { rlt::Term::Reference(i) }

    /// Code flow grammar
    /// --------------------------------------------------------------------------------------------

    rule else() -> rlt::ElseExpr =
        k:$"else" _ i:body() {
            rlt::ElseExpr {
                keyword: Keyword::from_located(k),
                body: i
            }
        }

    rule elif() -> rlt::ElifExpr =
        k:$"elif" _ c:operator_grammar() _ i:body() {
            rlt::ElifExpr {
                keyword: Keyword::from_located(k),
                condition: c,
                body: i
            }
        }

    rule if() -> rlt::IfExpr =
        k:$"if" _ c:operator_grammar() _ i:body() _ el:(elif() ** _) _ es:else()? {
            rlt::IfExpr {
                keyword: Keyword::from_located(k),
                condition: c,
                body: i,
                elif: el.into_boxed_slice(),
                el: es
            }
        }

    pub rule code_flow_grammar() -> rlt::IfExpr = if()

    /// Block level grammar
    /// --------------------------------------------------------------------------------------------

    rule block() -> rlt::ExpressionBlock =
        lb:$"{" _ i:separated(<block_level_grammar()>) _ rb:$"}" {
            rlt::ExpressionBlock {
                lbrace: Symbol::from_located(lb),
                expression: i.into_boxed_slice(),
                rbrace: Symbol::from_located(lb)
            }
        }

    rule simple() -> rlt::Body =
        f:$"=>" _ i:(
            i:block()            { rlt::BlockLevelNode::Block(i) }     /
            i:operator_grammar() { rlt::BlockLevelNode::Operation(i) }
        ) { rlt::Body::Simplified {
            flow: Symbol::from_located(f),
            expression: i
        } }

    pub rule body() -> rlt::Body =
        i:block() { rlt::Body::Block(i) } /
        simple()

    rule var_decl() -> rlt::Variable =
        k:$"val" _ id:ident() _ ty:return_type()? { rlt::Variable::Immutable {
            keyword: Keyword::from_located(k),
            id: id.span.into(),
            assigned_type: ty
        } } /
        k:$"var" _ id:ident() _ ty:return_type()? { rlt::Variable::Mutable {
            keyword: Keyword::from_located(k),
            id: id.span.into(),
            assigned_type: ty
        } }

    rule init_var() -> rlt::InitializedVariable =
        v:var_decl() _ e:$"=" _ o:operator_grammar() { rlt::InitializedVariable {
            variable: v,
            expression: o,
            equals: Symbol::from_located(e)
        } }

    pub rule block_level_grammar() -> rlt::BlockLevelNode =
        i:block()            { rlt::BlockLevelNode::Block(i) }     /
        i:init_var()         { rlt::BlockLevelNode::InitVar(i) }   /
        i:bodied()           { rlt::BlockLevelNode::Function(i) }  /
        i:operator_grammar() { rlt::BlockLevelNode::Operation(i) }

    /// Functions grammar
    /// --------------------------------------------------------------------------------------------

    rule bodied() -> rlt::BodiedFunction =
        k:$"fun" _ id:ident() _ ps:paren_enclosed(<comma_separated0(<parameter()>)>)? _
        ty:return_type()? _ b:body() {
            rlt::BodiedFunction {
                keyword: Keyword::from_located(k),
                params: ps.map_into(),
                id: id.span.into(),
                return_type: ty,
                body: Box::new(b)
            }
        }

    /// Top level grammar
    /// --------------------------------------------------------------------------------------------

    rule enum_statement() -> rlt::Enum =
        k:$"enum" _ "struct" _ id:type_ident() _ i:(
            ";"                                                  { None }    /
            i:brace_enclosed(<comma_separated0(<type_ident()>)>) { Some(i) }
        ) {
            rlt::Enum::Stack {
                keyword: Keyword::from_located(k),
                id,
                contents: i.map_into()
            }
        }

    rule struct_statement() -> rlt::Struct =
        k:$"struct" _ id:type_ident() _ ps:paren_enclosed(<comma_separated0(<typed_parameter()>)>)? _
        i:brace_enclosed(<separated(<bodied()>)>)? {
            rlt::Struct {
                keyword: Keyword::from_located(k),
                id,
                parameters: ps.map_into(),
                body: i.map_into()
            }
        }

    pub rule top_level_grammar() -> rlt::TopLevelNode =
        i:enum_statement()   { rlt::TopLevelNode::Enum(i) }           /
        i:struct_statement() { rlt::TopLevelNode::Struct(i) }         /
        i:bodied()           { rlt::TopLevelNode::BodiedFunction(i) }

    /// Modules grammar
    /// --------------------------------------------------------------------------------------------

    rule module() -> rlt::Module =
        k:$"module" _ id:type_ident() _ lb:$"{" _ i:separated(<top_level_grammar()>) _ rb:$"}" {
            rlt::Module::Ordinary {
                keyword: Keyword::from_located(k),
                id,
                lbrace: Symbol::from_located(lb),
                rbrace: Symbol::from_located(rb),
                rest: i.into_boxed_slice()
            }
        }

    rule global_module() -> rlt::Module =
        k:$"module" _ id:type_ident() _ f:$"=>" _ i:separated(<top_level_grammar()>) {
            rlt::Module::Global {
                keyword: Keyword::from_located(k),
                id,
                flow: Symbol::from_located(f),
                rest: i.into_boxed_slice()
            }
        }

    /// Root
    /// --------------------------------------------------------------------------------------------

    rule file_grammar() -> rlt::File =
        i:module() ++ _   { rlt::File::new(i.into_boxed_slice()) } /
        i:global_module() { rlt::File::new(Box::new([i])) }

    rule traced<T>(e: rule<T>) -> T =
        &(input:$([_]*) {
            #[cfg(feature = "trace")]
            println!("[PEG_INPUT_START]\n{}\n[PEG_TRACE_START]", input);
        })
        e:e()? {?
            #[cfg(feature = "trace")]
            println!("[PEG_TRACE_STOP]");
            e.ok_or("")
        }

    pub rule kodept() -> rlt::RLT =
        _ i:traced(<file_grammar()>) _ ![_] { rlt::RLT(i) }
}}
