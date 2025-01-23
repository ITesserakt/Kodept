use crate::function::Func;
use crate::properties::TopLevel;
use crate::top_level::{EnumDecl, StructDecl};
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::children::ChildrenDisjoint;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::{derive_node, Str};
use kodept_ast::properties::{RequireProperty, Root};
use kodept_rlt::prelude::{File, Module, TopLevelNode};

#[derive(Debug, PartialEq)]
pub enum ModKind {
    Global,
    Ordinary,
}

#[derive(Debug, PartialEq, Component)]
pub struct FileDecl;

#[derive(Debug, PartialEq, Component)]
pub struct ModDecl {
    pub kind: ModKind,
    pub name: Str,
}

derive_node!(FileDecl {
    relations = [children ModDecl,],
    properties = []
});
derive_node!(ModDecl {
    relations = [
        children EnumDecl where tag = TopLevel,
        children StructDecl where tag = TopLevel,
        children Func where tag = TopLevel,
    ],
    properties = []
});

impl FromSyntax for FileDecl {
    type Syntax = File;

    fn from_syntax(node: &File, source_code: impl CodeHolder, builder: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(builder, FileDecl)
            .with_property(Root)
            .with_children(source_code, builder, |scope| scope.many(node.0.as_ref()))
    }
}

impl FromSyntax for ModDecl {
    type Syntax = Module;

    fn from_syntax(node: &Module, source_code: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let (kind, id, rest) = match node {
            Module::Global { id, rest, .. } => (ModKind::Global, id, rest.as_ref()),
            Module::Ordinary { id, rest, .. } => (ModKind::Ordinary, id, rest.as_ref()),
        };
        let name = source_code.get_chunk_located(id);
        ASTBuilder::new(pool, ModDecl { kind, name })
            .with_children(source_code, pool, |scope| scope.choose(Unit, rest))
    }
}

impl Choose<TopLevelNode, ModDecl, TopLevel> for Unit {
    #[inline(always)]
    fn branch<S: CodeHolder>(node: &TopLevelNode) -> ChildrenDisjoint<ModDecl, S, TopLevel> {
        match node {
            TopLevelNode::Enum(x) => ChildrenDisjoint::new::<EnumDecl>(x),
            TopLevelNode::Struct(x) => ChildrenDisjoint::new::<StructDecl>(x),
            TopLevelNode::BodiedFunction(x) => ChildrenDisjoint::new::<Func>(x),
        }
    }
}

impl RequireProperty<Root> for FileDecl {}
