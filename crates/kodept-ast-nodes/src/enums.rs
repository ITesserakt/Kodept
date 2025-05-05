use crate::block_level::{InitVar, VarDecl};
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::file::{FileDecl, ModDecl};
use crate::function::{FuncBody, FuncDecl, FuncSignature};
use crate::literal::{Literal, Tuple};
use crate::top_level::{EnumConst, EnumDecl, StructDecl};
use crate::types::{NonTyParam, ProdTy, Ty, TyParam};
use kodept_ast::define_union;
use crate::code_flow::{ElifExpr, ElseExpr, IfExpr};
use crate::term::Ref;

define_union!(pub enum NodeUnion[NodeUnionItem] {
    FileDecl
    | ModDecl
    | StructDecl
    | EnumDecl
    | EnumConst
    | FuncDecl
    | FuncBody
    | FuncSignature
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
