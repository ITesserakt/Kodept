use derive_more::From;
use visita::{impl_visitor, node_group, Node, NodeFamily};

#[derive(Debug, Clone)]
enum BinOp {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
struct NumLit(f32);

#[derive(Debug, Clone)]
struct Tuple(Vec<Literal>);

#[derive(Debug, Clone)]
struct Bin {
    op: BinOp,
    lhs: Box<Expr>,
    rhs: Box<Expr>,
}

#[derive(Debug, Clone, From)]
enum Expr {
    NumLit(NumLit),
    Bin(Bin),
}

#[derive(Debug, Clone, From)]
enum Literal {
    Num(NumLit),
    Tuple(Tuple),
}

node_group! {
    family: Expr,
    nodes: [
        Expr,
        NumLit,
        Bin
    ],
    meta: ()
}

node_group! {
    family: Literal,
    nodes: [
        Literal,
        NumLit, // <- NumLit is in both groups
        Tuple
    ],
    meta: ()
}

struct InPlaceFlatten(Vec<f32>);

impl_visitor! {
    InPlaceFlatten,
    family: Expr,
    output: (),
    [
        Expr => |visitor, node, data| {
            match node {
                Expr::NumLit(x) => Expr::accept_node(visitor, x, data),
                Expr::Bin(x) => Expr::accept_node(visitor, x, data)
            }
        },
        NumLit => |visitor, node, data| {
            visitor.0.push(node.0);
        },
        Bin => |visitor, node, data| {
            node.lhs.accept(visitor, data);
            node.rhs.accept(visitor, data);
        }
    ]
}

impl_visitor! {
    InPlaceFlatten,
    family: Literal,
    output: (),
    [
        Literal => |this, node, data| {
            match node {
                Literal::Num(x) => Literal::accept_node(this, x, data),
                Literal::Tuple(x) => Literal::accept_node(this, x, data)
            }
        },
        NumLit => |this, node, data| {
            this.0.push(node.0);
        },
        Tuple => |this, node, data| {
            for item in &mut node.0 {
                item.accept(this, data);
            }
        }
    ]
}

fn main() {
    let mut expr: Expr = Bin {
        op: BinOp::Add,
        lhs: Box::new(NumLit(23.0).into()),
        rhs: Box::new(
            Bin {
                op: BinOp::Sub,
                lhs: Box::new(NumLit(42.0).into()),
                rhs: Box::new(NumLit(19.0).into()),
            }
            .into(),
        ),
    }
    .into();

    let mut literal: Literal = Tuple(vec![
        Tuple(vec![NumLit(1.0).into(), NumLit(1.5).into()]).into(),
        NumLit(2.0).into(),
    ])
    .into();
    let mut executor_expr = InPlaceFlatten(vec![]);
    let mut executor_lit = InPlaceFlatten(vec![]);

    expr.accept(&mut executor_expr, &());
    literal.accept(&mut executor_lit, &());

    assert_eq!(executor_expr.0, vec![23.0, 42.0, 19.0]);
    assert_eq!(executor_lit.0, vec![1.0, 1.5, 2.0]);
}
