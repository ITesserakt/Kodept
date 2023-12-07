use derive_more::From;
use visita::{impl_visitor, node_group, Node};

#[derive(Debug, Clone)]
enum BinOp {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
struct NumLit(f32);

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

node_group! {
    family: Expr,
    nodes: [
        Expr,
        NumLit,
        Bin
    ],
    meta: ()
}

struct Interpreter;

impl_visitor! {
    Interpreter,
    family: Expr,
    output: f32,
    [
        Expr => |visitor, node, data| {
            match node {
                Expr::NumLit(x) => x.accept(visitor, data),
                Expr::Bin(x) => x.accept(visitor, data)
            }
        },
        NumLit => |visitor, node, data| {
            node.0
        },
        Bin => |visitor, node, data| {
            match node.op {
                BinOp::Add => node.lhs.accept(visitor, data) + node.rhs.accept(visitor, data),
                BinOp::Sub => node.lhs.accept(visitor, data) - node.rhs.accept(visitor, data)
            }
        }
    ]
}

struct Printer;

impl_visitor! {
    Printer,
    family: Expr,
    output: String,
    [
        Expr => |visitor, node, data| {
            match node {
                Expr::NumLit(x) => x.accept(visitor, data),
                Expr::Bin(x) => x.accept(visitor, data)
            }
        },
        NumLit => |visitor, node, data| {
            node.0.to_string()
        },
        Bin => |visitor, node, data| {
            format!("({} {} {})", node.lhs.accept(visitor, data), match node.op {
                BinOp::Add => "+",
                BinOp::Sub => "-"
            }, node.rhs.accept(visitor, data))
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

    let interpreter_res = expr.accept(&mut Interpreter, &());
    let printer_res = expr.accept(&mut Printer, &());

    assert_eq!(interpreter_res, 46.0);
    assert_eq!(printer_res, "(23 + (42 - 19))");
}
