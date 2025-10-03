#![allow(unsafe_code)]

use crate::traversal::sealed::Sealed;
use kodept_core::Freeze;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SyntaxNodeKind {
    File,
    Module,
    Keyword,
    Symbol,
    TypeName,
    Identifier,
    TopLevel,
    Enum,
    Struct,
    Function,
    Body,
    ExpressionBlock,
    BlockLevel,
    InitializedVariable,
    Variable,
    Operation,
    Expression,
    Term,
    Literal,
    Lambda,
    If,
    Elif,
    Else,
    Application,
    Parameter,
    UntypedParameter,
    TypedParameter,
    Type,
    Tuple,
    Binary,
    Unary,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ErasedNodeBorrow<'a>(ErasedNodePtr, PhantomData<&'a ()>);

#[allow(private_bounds)]
pub trait SyntaxNode: SpanBounds + Located + Send + Sync + 'static + Sealed {
    #[allow(private_interfaces)]
    const KIND: SyntaxNodeKind;
}

trait DynSyntaxNode: SpanBounds + Located {
    fn kind(&self) -> SyntaxNodeKind;
}
impl<T: SyntaxNode> DynSyntaxNode for T {
    fn kind(&self) -> SyntaxNodeKind {
        T::KIND
    }
}

#[derive(Copy, Clone, Hash, Eq)]
pub struct ErasedNodePtr {
    ptr: Freeze<NonNull<dyn DynSyntaxNode>>,
}

impl ErasedNodePtr {
    #[inline(always)]
    fn new_with_parent<T: SyntaxNode>(value: &T) -> Self {
        Self::new(value)
    }

    #[inline(always)]
    pub fn new<T>(value: &T) -> Self
    where
        T: SyntaxNode,
    {
        Self {
            ptr: Freeze::new(NonNull::from_ref(value)),
        }
    }

    #[inline(always)]
    /// SAFETY: lifetime 'a should not be larger than lifetime of an actual data
    pub unsafe fn borrow<'a>(&self) -> ErasedNodeBorrow<'a> {
        ErasedNodeBorrow(*self, PhantomData)
    }
}

impl<'a> ErasedNodeBorrow<'a> {
    #[inline]
    fn as_ref(&self) -> &'a dyn DynSyntaxNode {
        // SAFETY: ptr is immutable and constructed using `NonNull::from_ref`
        unsafe { self.0.ptr.as_ref() }
    }

    #[inline]
    #[allow(private_bounds)]
    pub fn try_cast<T: SyntaxNode>(&self) -> Option<&'a T> {
        if self.as_ref().kind() == T::KIND {
            // SAFETY: Kind should be uniquely tied to node type, therefore if kinds are equal, then
            // types are equal too
            Some(unsafe { self.0.ptr.cast().as_ref() })
        } else {
            None
        }
    }
}

impl SpanBounds for ErasedNodeBorrow<'_> {
    #[inline]
    fn bounds(&self) -> Span {
        self.as_ref().bounds()
    }
}

impl Located for ErasedNodeBorrow<'_> {
    #[inline]
    fn location(&self) -> CodePoint {
        self.as_ref().location()
    }
}

impl PartialEq for ErasedNodePtr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ptr.as_ptr(), other.ptr.as_ptr())
    }
}

impl Debug for ErasedNodePtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasedNodePtr")
            .field("ptr", &*self.ptr)
            .finish_non_exhaustive()
    }
}

unsafe impl Send for ErasedNodePtr {}
unsafe impl Sync for ErasedNodePtr {}

pub unsafe trait Traversal {
    fn traverse(&self, callback: &mut impl FnMut(ErasedNodePtr));
}

mod sealed {
    pub(super) trait Sealed {}
}

mod impls {
    use super::*;
    use crate::{new_types, prelude};

    macro_rules! impls {
        (for $t:ty {
            traverse($this:ident, $callback:ident, $parent:ident) => $traverse:block,
            kind => $kind:expr
        }) => {
            unsafe impl Traversal for $t {
                #[inline]
                fn traverse(&$this, $callback: &mut impl FnMut(ErasedNodePtr)) {
                    $traverse
                }
            }
            impl SyntaxNode for $t {
                #[allow(private_interfaces)]
                const KIND: SyntaxNodeKind = $kind;
            }
            impl Sealed for $t {}
        };
    }

    impls!(for prelude::File {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);
            self.0.traverse(callback);
        },
        kind => SyntaxNodeKind::File
    });

    impls!(for prelude::Module {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Module::Global {
                    keyword,
                    id,
                    flow,
                    rest,
                } => {
                    keyword.traverse(callback);
                    id.traverse(callback);
                    flow.traverse(callback);
                    rest.traverse(callback);
                }
                prelude::Module::Ordinary {
                    keyword,
                    id,
                    lbrace,
                    rest,
                    rbrace,
                } => {
                    keyword.traverse(callback);
                    id.traverse(callback);
                    lbrace.traverse(callback);
                    rest.traverse(callback);
                    rbrace.traverse(callback);
                }
            }
        },
        kind => SyntaxNodeKind::Module
    });

    impls!(for new_types::Keyword {
        traverse(self, callback, parent) => {
            callback(ErasedNodePtr::new_with_parent(self));
        },
        kind => SyntaxNodeKind::Keyword
    });

    impls!(for new_types::TypeName {
        traverse(self, callback, parent) => {
            callback(ErasedNodePtr::new_with_parent(self));
        },
        kind => SyntaxNodeKind::TypeName
    });

    impls!(for new_types::Symbol {
        traverse(self, callback, parent) => {
            callback(ErasedNodePtr::new_with_parent(self))
        },
        kind => SyntaxNodeKind::Symbol
    });

    impls!(for new_types::Identifier {
        traverse(self, callback, parent) => {
            callback(ErasedNodePtr::new_with_parent(self))
        },
        kind => SyntaxNodeKind::Identifier
    });

    impls!(for prelude::TopLevelNode {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::TopLevelNode::Enum(x) => x.traverse(callback),
                prelude::TopLevelNode::Struct(x) => x.traverse(callback),
                prelude::TopLevelNode::BodiedFunction(x) => x.traverse(callback),
            }
        },
        kind => SyntaxNodeKind::TopLevel
    });

    impls!(for prelude::Enum {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Enum::Stack {
                    keyword,
                    id,
                    contents,
                } => {
                    keyword.traverse(callback);
                    id.traverse(callback);
                    contents.traverse(callback);
                }
                prelude::Enum::Heap {
                    keyword,
                    id,
                    contents,
                } => {
                    keyword.traverse(callback);
                    id.traverse(callback);
                    contents.traverse(callback);
                }
            }
        },
        kind => SyntaxNodeKind::Enum
    });

    impls!(for prelude::Struct {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.keyword.traverse(callback);
            self.id.traverse(callback);
            self.body.traverse(callback);
            self.parameters.traverse(callback);
        },
        kind => SyntaxNodeKind::Struct
    });

    impls!(for prelude::BodiedFunction {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.keyword.traverse(callback);
            self.id.traverse(callback);
            self.params.traverse(callback);
            self.return_type.traverse(callback);
            self.body.traverse(callback);
        },
        kind => SyntaxNodeKind::Function
    });

    impls!(for prelude::Body {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Body::Block(x) => x.traverse(callback),
                prelude::Body::Simplified { flow, expression } => {
                    flow.traverse(callback);
                    expression.traverse(callback);
                }
            }
        },
        kind => SyntaxNodeKind::Body
    });

    impls!(for prelude::ExpressionBlock {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.lbrace.traverse(callback);
            self.expression.traverse(callback);
            self.rbrace.traverse(callback);
        },
        kind => SyntaxNodeKind::ExpressionBlock
    });

    impls!(for prelude::BlockLevelNode {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::BlockLevelNode::Block(x) => x.traverse(callback),
                prelude::BlockLevelNode::Function(x) => x.traverse(callback),
                prelude::BlockLevelNode::InitVar(x) => x.traverse(callback),
                prelude::BlockLevelNode::Operation(x) => x.traverse(callback),
            }
        },
        kind => SyntaxNodeKind::BlockLevel
    });

    impls!(for prelude::InitializedVariable {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.variable.traverse(callback);
            self.equals.traverse(callback);
            self.expression.traverse(callback);
        },
        kind => SyntaxNodeKind::InitializedVariable
    });

    impls!(for prelude::Variable {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Variable::Mutable {
                    keyword,
                    id,
                    assigned_type,
                } => {
                    keyword.traverse(callback);
                    id.traverse(callback);
                    assigned_type.traverse(callback);
                }
                prelude::Variable::Immutable {
                    keyword,
                    id,
                    assigned_type,
                } => {
                    keyword.traverse(callback);
                    id.traverse(callback);
                    assigned_type.traverse(callback);
                }
            }
        },
        kind => SyntaxNodeKind::Variable
    });

    impls!(for prelude::Operation {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Operation::Block(x) => x.traverse(callback),
                prelude::Operation::Access { left, dot, right } => {
                    left.traverse(callback);
                    dot.traverse(callback);
                    right.traverse(callback);
                }
                prelude::Operation::Binary {
                    left,
                    operation,
                    right,
                } => {
                    left.traverse(callback);
                    operation.traverse(callback);
                    right.traverse(callback);
                }
                prelude::Operation::Unary { operator, expr } => {
                    operator.traverse(callback);
                    expr.traverse(callback);
                }
                prelude::Operation::Application(x) => x.traverse(callback),
                prelude::Operation::Expression(x) => x.traverse(callback),
            }
        },
        kind => SyntaxNodeKind::Operation
    });

    impls!(for prelude::Expression {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Expression::If(x) => x.traverse(callback),
                prelude::Expression::Lambda(x) => x.traverse(callback),
                prelude::Expression::Literal(x) => x.traverse(callback),
                prelude::Expression::Term(x) => x.traverse(callback),
            }
        },
        kind => SyntaxNodeKind::Expression
    });

    impls!(for prelude::Term {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Term::ContextualReference(x) => x.traverse(callback),
                prelude::Term::Constant(x) => x.traverse(callback),
                prelude::Term::Reference(x) => x.traverse(callback),
                prelude::Term::ContextualConstant(x) => x.traverse(callback),
            }
        },
        kind => SyntaxNodeKind::Term
    });

    unsafe impl<T: Traversal> Traversal for prelude::Contextual<T> {
        fn traverse(
            &self,
            callback: &mut impl FnMut(ErasedNodePtr),
        ) {
            self.inner.traverse(callback);
        }
    }

    impls!(for prelude::Literal {
        traverse(self, callback, parent) => {
            if let prelude::Literal::Tuple(x) = self {
                let this = ErasedNodePtr::new_with_parent(self);
                callback(this);
                x.traverse(callback);
            } else {
                callback(ErasedNodePtr::new_with_parent(self));
            }
        },
        kind => SyntaxNodeKind::Literal
    });

    impls!(for prelude::Lambda {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.binds.traverse(callback);
            self.flow.traverse(callback);
            self.expr.traverse(callback);
        },
        kind => SyntaxNodeKind::Lambda
    });

    impls!(for prelude::IfExpr {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.keyword.traverse(callback);
            self.condition.traverse(callback);
            self.body.traverse(callback);
            self.elif.traverse(callback);
            self.el.traverse(callback);
        },
        kind => SyntaxNodeKind::If
    });

    impls!(for prelude::ElifExpr {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.keyword.traverse(callback);
            self.condition.traverse(callback);
            self.body.traverse(callback);
        },
        kind => SyntaxNodeKind::Elif
    });

    impls!(for prelude::ElseExpr {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.keyword.traverse(callback);
            self.body.traverse(callback);
        },
        kind => SyntaxNodeKind::Else
    });

    impls!(for prelude::Application {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.expr.traverse(callback);
            self.params.traverse(callback);
        },
        kind => SyntaxNodeKind::Application
    });

    impls!(for prelude::Parameter {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Parameter::Typed(x) => x.traverse(callback),
                prelude::Parameter::Untyped(x) => x.traverse(callback),
            }
        },
        kind => SyntaxNodeKind::Parameter
    });

    impls!(for prelude::UntypedParameter {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.id.traverse(callback);
        },
        kind => SyntaxNodeKind::UntypedParameter
    });

    impls!(for prelude::TypedParameter {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            self.id.traverse(callback);
            self.parameter_type.traverse(callback);
        },
        kind => SyntaxNodeKind::TypedParameter
    });

    impls!(for prelude::Type {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);

            match self {
                prelude::Type::ContextualReference(_, x) => x.traverse(callback),
                prelude::Type::Reference(x) => x.traverse(callback),
                prelude::Type::Tuple(x) => x.traverse(callback),
            }
        },
        kind => SyntaxNodeKind::Type
    });

    impls!(for prelude::Tuple {
        traverse(self, callback, parent) => {
            let this = ErasedNodePtr::new_with_parent(self);
            callback(this);
            self.0.traverse(callback);
        },
        kind => SyntaxNodeKind::Tuple
    });

    impls!(for new_types::BinaryOperationSymbol {
        traverse(self, callback, parent) => {
            callback(ErasedNodePtr::new_with_parent(self));
        },
        kind => SyntaxNodeKind::Binary
    });

    impls!(for new_types::UnaryOperationSymbol {
        traverse(self, callback, parent) => {
            callback(ErasedNodePtr::new_with_parent(self));
        },
        kind => SyntaxNodeKind::Unary
    });

    unsafe impl<T: Traversal> Traversal for new_types::Enclosed<T> {
        #[inline]
        fn traverse(
            &self,
            callback: &mut impl FnMut(ErasedNodePtr),
        ) {
            // do not call callback because Enclosed<T> is not a node

            self.left.traverse(callback);
            self.inner.traverse(callback);
            self.right.traverse(callback);
        }
    }

    unsafe impl<T: Traversal + ?Sized> Traversal for Box<T> {
        #[inline]
        fn traverse(
            &self,
            callback: &mut impl FnMut(ErasedNodePtr),
        ) {
            // do not call callback because Box<T> is not a node
            self.as_ref().traverse(callback);
        }
    }

    unsafe impl<T: Traversal> Traversal for [T] {
        #[inline]
        fn traverse(
            &self,
            callback: &mut impl FnMut(ErasedNodePtr),
        ) {
            // do not call callback because Box<T> is not a node
            for item in self {
                item.traverse(callback);
            }
        }
    }

    unsafe impl<T: Traversal> Traversal for Option<T> {
        #[inline]
        fn traverse(
            &self,
            callback: &mut impl FnMut(ErasedNodePtr),
        ) {
            // do not call callback because Option<T> is not a node
            if let Some(item) = self {
                item.traverse(callback);
            }
        }
    }

    unsafe impl<T1: Traversal, T2: Traversal> Traversal for (T1, T2) {
        #[inline]
        fn traverse(
            &self,
            callback: &mut impl FnMut(ErasedNodePtr),
        ) {
            // do not call callback because (T1, T2) is not a node
            self.0.traverse(callback);
            self.1.traverse(callback);
        }
    }
}
