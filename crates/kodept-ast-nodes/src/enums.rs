use crate::block_level::{InitVar, VarDecl};
use crate::code_flow::{ElifExpr, ElseExpr, IfExpr};
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::file::{FileDecl, ModDecl};
use crate::function::Func;
use crate::literal::{Literal, Tuple};
use crate::term::Ref;
use crate::top_level::{EnumConst, EnumDecl, StructDecl};
use crate::types::{NonTyParam, ProdTy, Ty, TyParam};
use kodept_ast::define_union;

define_union!(pub enum NodeUnion[NodeUnionItem] {
    FileDecl 
    | ModDecl 
    | StructDecl 
    | EnumDecl 
    | EnumConst 
    | Func 
    | VarDecl 
    | InitVar 
    | IfExpr 
    | ElifExpr 
    | ElseExpr 
    | Exprs 
    | App 
    | Lambda 
    | BinExpr 
    | UnExpr 
    | Literal 
    | Tuple 
    | Ref 
    | Ty 
    | ProdTy 
    | TyParam 
    | NonTyParam
});

define_union!(pub enum TopLevelUnion[TopLevelUnionItem, TopLevelUnionFilter] {
    StructDecl
    | EnumDecl
    | Func
});
