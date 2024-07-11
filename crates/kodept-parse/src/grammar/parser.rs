use crate::grammar::macros::tok;
use crate::lexer::Symbol::*;
use crate::lexer::Token::*;
use crate::lexer::Literal::*;
use crate::lexer::Identifier as I;
use crate::lexer::Operator::*;
use crate::lexer::LogicOperator::*;
use crate::lexer::BitOperator::*;
use crate::lexer::ComparisonOperator::*;
use crate::lexer::MathOperator::*;
use crate::lexer::Keyword::*;
use crate::parser::nom::VerboseEnclosed;
use crate::token_stream::TokenStream;
use kodept_core::structure::*;
use kodept_core::structure::rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use crate::OptionTExt;

peg::parser! {grammar grammar<'t>() for TokenStream<'t> {
    /// UTILITIES
    /// --------------------------------------------------------------------------------------------
    
    rule comma_separated0<T>(items: rule<T>) -> Vec<T> =
        i:(items() ** ",") ","? { i }
    
    rule comma_separated1<T>(items: rule<T>) -> Vec<T> =
        i:(items() ++ ",") ","? { i }

    rule paren_enclosed<T>(inner: rule<T>) -> VerboseEnclosed<'input, T> =
        lp:[tok!(Symbol(LParen))] i:inner() rp:[tok!(Symbol(RParen))] { VerboseEnclosed::from((lp, i, rp)) }
    
    rule brace_enclosed<T>(inner: rule<T>) -> VerboseEnclosed<'input, T> =
        lp:[tok!(Symbol(LBrace))] i:inner() rp:[tok!(Symbol(RBrace))] { VerboseEnclosed::from((lp, i, rp)) }
    
    rule separated<T>(inner: rule<T>) -> Vec<T> =
        inner() ** (("\n" / "\t" / ";" / "\r\n")+)
    
    /// Type grammar
    /// --------------------------------------------------------------------------------------------

    rule reference() -> rlt::new_types::TypeName =
        i:[tok!(Identifier(I::Type(_)))] { rlt::new_types::TypeName::from(i.span) }

    rule tuple() -> rlt::Type =
        i:paren_enclosed(<comma_separated0(<type_grammar()>)>) { rlt::Type::Tuple(i.into()) }

    pub rule type_grammar() -> rlt::Type =
        i:reference() { rlt::Type::Reference(i) } /
        tuple()
    
    /// Parameters grammar
    /// --------------------------------------------------------------------------------------------

    pub rule typed_parameter() -> rlt::TypedParameter =
        i:[tok!(Identifier(I::Identifier(_)))] [tok!(Symbol(Colon))] t:type_grammar() { 
            rlt::TypedParameter {  id: i.span.into(), parameter_type: t} 
        }
    
    pub rule untyped_parameter() -> rlt::UntypedParameter =
        i:[tok!(Identifier(I::Identifier(_)))] ([tok!(Symbol(Colon))] [tok!(Symbol(TypeGap))])? { 
            rlt::UntypedParameter { id: i.span.into() } 
        }
    
    pub rule parameter() -> rlt::Parameter =
        t:typed_parameter() { rlt::Parameter::Typed(t) } /
        u:untyped_parameter() { rlt::Parameter::Untyped(u) }
    
    /// Literals grammar
    /// --------------------------------------------------------------------------------------------
    
    rule tuple_literal() -> rlt::Literal =
        i:paren_enclosed(<comma_separated0(<operator_grammar()>)>) { 
            rlt::Literal::Tuple(i.into())
        }
    
    pub rule literal_grammar() -> rlt::Literal =
        i:[tok!(Literal(Binary(_)))] { rlt::Literal::Binary(i.span) } /
        i:[tok!(Literal(Octal(_)))] { rlt::Literal::Octal(i.span) } /
        i:[tok!(Literal(Hex(_)))] { rlt::Literal::Hex(i.span) } /
        i:[tok!(Literal(Floating(_)))] { rlt::Literal::Floating(i.span) } /
        i:[tok!(Literal(Char(_)))] { rlt::Literal::Char(i.span) } /
        i:[tok!(Literal(String(_)))] { rlt::Literal::String(i.span) }
    
    /// Operators grammar
    /// --------------------------------------------------------------------------------------------
    
    pub rule operator_grammar() -> rlt::Operation = precedence! {
        a:(@) op:[tok!(Operator(Logic(OrLogic)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Logic(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Logic(AndLogic)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Logic(op.span.into()),
            right: Box::new(b)
        } }
        --
        a:(@) op:[tok!(Operator(Bit(OrBit)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Bit(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Bit(AndBit)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Bit(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Bit(XorBit)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Bit(op.span.into()),
            right: Box::new(b)
        } }
        --
        a:(@) op:[tok!(Operator(Comparison(Less)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Comparison(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Comparison(Greater)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Comparison(op.span.into()),
            right: Box::new(b)
        } }
        --
        a:(@) op:[tok!(Operator(Comparison(LessEquals)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Comparison(NotEquiv)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Comparison(Equiv)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Comparison(GreaterEquals)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(op.span.into()),
            right: Box::new(b)
        } }
        --
        a:(@) op:[tok!(Operator(Comparison(Spaceship)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::ComplexComparison(op.span.into()),
            right: Box::new(b)
        } }
        --
        a:(@) op:[tok!(Operator(Math(Plus)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Add(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Math(Sub)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Add(op.span.into()),
            right: Box::new(b)
        } }
        --
        a:(@) op:[tok!(Operator(Math(Times)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Mul(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Math(Div)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Mul(op.span.into()),
            right: Box::new(b)
        } }
        a:(@) op:[tok!(Operator(Math(Mod)))] b:@ { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Mul(op.span.into()),
            right: Box::new(b)
        } }
        --
        a:@ op:[tok!(Operator(Math(Pow)))] b:(@) { rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Pow(op.span.into()),
            right: Box::new(b)
        } }
        --
        op:[tok!(Operator(Math(Sub)))] a:@ { rlt::Operation::TopUnary {
            operator: UnaryOperationSymbol::Neg(op.span.into()),
            expr: Box::new(a)
        } }
        op:[tok!(Operator(Logic(NotLogic)))] a:@ { rlt::Operation::TopUnary {
            operator: UnaryOperationSymbol::Not(op.span.into()),
            expr: Box::new(a)
        } }
        op:[tok!(Operator(Bit(NotBit)))] a:@ { rlt::Operation::TopUnary {
            operator: UnaryOperationSymbol::Inv(op.span.into()),
            expr: Box::new(a)
        } }
        op:[tok!(Operator(Math(Plus)))] a:@ { rlt::Operation::TopUnary {
            operator: UnaryOperationSymbol::Plus(op.span.into()),
            expr: Box::new(a)
        } }
        --
        expr:@ "" param:(@) { match param {
            rlt::Operation::Expression(rlt::Expression::Literal(rlt::Literal::Tuple(ps))) => rlt::Operation::Application(Box::new(rlt::Application { 
                expr,
                params: Some(ps) 
            })),
            _ => expr,
        } }
        --
        a:@ op:[tok!(Operator(Dot))] b:(@) { rlt::Operation::Access { 
            left: Box::new(a),
            dot: op.span.into(),
            right: Box::new(b)
        } }
        --
        i:expression_grammar()                                             { rlt::Operation::Expression(i) }
        [tok!(Symbol(LParen))] i:operator_grammar() [tok!(Symbol(RParen))] { i }
    }
    
    /// Expressions grammar
    /// --------------------------------------------------------------------------------------------
    
    rule lambda() -> rlt::Expression =
        l:[tok!(Keyword(Lambda))] ps:comma_separated0(<parameter()>) f:[tok!(Operator(Flow))] expr:operator_grammar() {
        rlt::Expression::Lambda {
            keyword: l.span.into(),
            binds: ps.into_boxed_slice(),
            flow: f.span.into(),
            expr: Box::new(expr)
        }
    }
    
    pub rule expression_grammar() -> rlt::Expression = 
        lambda()                                            /
        i:term_grammar()    { rlt::Expression::Term(i) }    /
        i:literal_grammar() { rlt::Expression::Literal(i) }
    
    /// References grammar
    /// --------------------------------------------------------------------------------------------
    
    /// |      | Global   | Local     |
    /// | ---- | -------- | --------- |
    /// | Type | ::{X::}X | X::X{::X} |
    /// | Ref  | ::{X::}x | X::{X::}x |
    
    rule type_ref() -> rlt::Reference =
        t:[tok!(Identifier(I::Type(_)))] { rlt::Reference::Type(t.span.into()) }
    
    rule variable_ref() -> rlt::Reference =
        t:[tok!(Identifier(I::Identifier(_)))] { rlt::Reference::Identifier(t.span.into()) }
    
    rule ref() -> rlt::Reference =
        variable_ref() / 
        type_ref()
    
    rule global_type_ref() -> (rlt::Context, rlt::Reference) =
        g:[tok!(Symbol(DoubleColon))] ctx:(type_ref() ** "::") "::"? t:type_ref() {
            let start = rlt::Context::Global {
                colon: g.span.into()   
            };
            let context = ctx.into_iter().fold(start, |acc, next| rlt::Context::Inner {
                parent: Box::new(acc),
                needle: next
            });
            (context, t)
        }
    
    rule global_ref() -> (rlt::Context, rlt::Reference) =
        g:[tok!(Symbol(DoubleColon))] ctx:(type_ref() ** "::") "::"? v:variable_ref() {
            let start = rlt::Context::Global {
                colon: g.span.into()   
            };
            let context = ctx.into_iter().fold(start, |acc, next| rlt::Context::Inner {
                parent: Box::new(acc),
                needle: next
            });
            (context, v)
        }
    
    rule local_type_ref() -> (rlt::Context, rlt::Reference) =
        ctx:(type_ref() ++ "::") "::" t:type_ref() {
            let start = rlt::Context::Local;
            let context = ctx.into_iter().fold(start, |acc, next| rlt::Context::Inner {
                parent: Box::new(acc),
                needle: next
            });
            (context, t)
        }
    
    rule local_ref() -> (rlt::Context, rlt::Reference) =
        ctx:(type_ref() ++ "::") "::" v:variable_ref() {
            let start = rlt::Context::Local;
            let context = ctx.into_iter().fold(start, |acc, next| rlt::Context::Inner {
                parent: Box::new(acc),
                needle: next
            });
            (context, v)
        }
    
    rule contextual() -> rlt::ContextualReference = i:(
        global_type_ref() / 
        global_ref()      /
        local_type_ref()  /
        local_ref()
    ) { rlt::ContextualReference {
        context: i.0,
        inner: i.1
    } }
    
    pub rule term_grammar() -> rlt::Term = 
        i:contextual() { rlt::Term::Contextual(i) } /
        i:ref()        { rlt::Term::Reference(i) }
    
    /// Block level grammar
    /// --------------------------------------------------------------------------------------------
    
    rule block() -> rlt::ExpressionBlock =
        lb:[tok!(Symbol(LBrace))] i:separated(<block_level_grammar()>) rb:[tok!(Symbol(RBrace))] {
            rlt::ExpressionBlock { 
                lbrace: lb.span.into(),
                expression: i.into_boxed_slice(),
                rbrace: rb.span.into()
            }
        }
    
    rule simple() -> rlt::Body =
        f:[tok!(Operator(Flow))] i:block_level_grammar() { rlt::Body::Simplified {
            flow: f.span.into(),
            expression: i
        } }
    
    pub rule body() -> rlt::Body =
        i:block() { rlt::Body::Block(i) } /
        simple()
    
    rule var_decl() -> rlt::Variable =
        k:[tok!(Keyword(Val))] id:[tok!(Identifier(I::Identifier(_)))] ty:(c:[tok!(Symbol(Colon))] t:type_grammar() { 
            (rlt::new_types::Symbol(c.span), t) }
        )?
        {
            rlt::Variable::Immutable {
                keyword: k.span.into(),
                id: id.span.into(),
                assigned_type: ty
            }    
        } /
        k:[tok!(Keyword(Var))] id:[tok!(Identifier(I::Identifier(_)))] ty:(c:[tok!(Symbol(Colon))] t:type_grammar() { 
            (rlt::new_types::Symbol(c.span), t) }
        )?
        {
            rlt::Variable::Mutable {
                keyword: k.span.into(),
                id: id.span.into(),
                assigned_type: ty
            }    
        }
    
    rule init_var() -> rlt::InitializedVariable =
        v:var_decl() e:[tok!(Operator(Comparison(Equals)))] o:operator_grammar() { rlt::InitializedVariable {
            variable: v,
            expression: o,
            equals: e.span.into()
        } }
    
    pub rule block_level_grammar() -> rlt::BlockLevelNode =
        i:block()            { rlt::BlockLevelNode::Block(i) }     /
        i:init_var()         { rlt::BlockLevelNode::InitVar(i) }   /
        i:operator_grammar() { rlt::BlockLevelNode::Operation(i) }
    
    /// Functions grammar
    /// --------------------------------------------------------------------------------------------
    
    rule bodied() -> rlt::BodiedFunction =
        k:[tok!(Keyword(Fun))] 
        id:[tok!(Identifier(I::Identifier(_)))] 
        ps:paren_enclosed(<comma_separated0(<parameter()>)>)? 
        ty:(c:[tok!(Symbol(Colon))] ty:type_grammar() { (rlt::new_types::Symbol(c.span), ty) })?
        b:body() {
            rlt::BodiedFunction {
                keyword: k.span.into(),
                params: ps.map_into(),
                id: id.span.into(),
                return_type: ty,
                body: Box::new(b) 
            }
        }
    
    /// Top level grammar
    /// --------------------------------------------------------------------------------------------
    
    rule enum_statement() -> rlt::Enum =
        k:[tok!(Keyword(Enum))] [tok!(Keyword(Struct))] id:reference() i:(
            [tok!(Symbol(Semicolon))] { None } /
            i:brace_enclosed(<comma_separated0(<reference()>)>) { Some(i) }
        ) {
            rlt::Enum::Stack { 
                keyword: k.span.into(),
                id,
                contents: i.map_into()
            }
        }
    
    rule struct_statement() -> rlt::Struct = 
        k:[tok!(Keyword(Struct))] id:reference() 
        ps:paren_enclosed(<comma_separated0(<typed_parameter()>)>)?
        i:brace_enclosed(<separated(<bodied()>)>)? {
            rlt::Struct {
                keyword: k.span.into(),
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
        k:[tok!(Keyword(Module))]
        id:[tok!(Identifier(I::Type(_)))]
        lb:[tok!(Symbol(LBrace))]
        i:separated(<top_level_grammar()>)
        rb:[tok!(Symbol(RBrace))] {
            rlt::Module::Ordinary { 
                keyword: k.span.into(),
                id: id.span.into(),
                lbrace: lb.span.into(),
                rbrace: rb.span.into(),
                rest: i.into_boxed_slice()
            }
        }
    
    rule global_module() -> rlt::Module =
        k:[tok!(Keyword(Module))]
        id:[tok!(Identifier(I::Type(_)))]
        f:[tok!(Operator(Flow))]
        i:separated(<top_level_grammar()>) {
            rlt::Module::Global { 
                keyword: k.span.into(),
                id: id.span.into(),
                flow: f.span.into(),
                rest: i.into_boxed_slice()
            }
        }
    
    /// Root
    /// --------------------------------------------------------------------------------------------
    
    rule file_grammar() -> rlt::File =
        i:module()+       { rlt::File::new(i.into_boxed_slice()) } /
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
        i:traced(<file_grammar()>) ![_] { rlt::RLT(i) }
}}

pub use grammar::kodept;
