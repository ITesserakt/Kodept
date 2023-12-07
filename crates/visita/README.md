# visita &emsp; 

- [**crates.io**](https://crates.io/crates/visita)
- [**github**](https://github.com/jvcmarcenes/visita)

Elegant implementation of the [**Visitor Pattern**](https://en.wikipedia.org/wiki/Visitor_pattern) in Rust

---

## Usage:

```rust
use visita::*;

pub enum Operation { Add, Sub, Mul, Div }

// Use the `node_group` macro to annotate your group of nodes
// the `data` field allows you to attach additional data to your nodes
#[node_group(data = ())]
pub enum Expr {

  NumLit(f64),

  Binary {
    op: Operation,
    lhs: Expr,
    rhs: Expr,
  }

}

// use the `visitor` macro to annotate your visitor structs
// the `output` field marks the result of the visit operation
// this macro will require that your Visitor implements Visit for every variant in the enum
#[visitor(Expr, output = f32)]
struct Interpreter;

impl Visit<NumLit> for Interpreter {
  fn visit(&mut self, node: &NumLit, _data: &Data<Self, NumLit>) -> Self::Output {
    node.0
  }
}

impl Visit<Binary> for Interpreter {
  fn visit(&mut self, node: &Binary_data: &Data<Self, Binary>) -> Self::Output {
    match node.op {
      Operation::Add => node.lhs.accept(self) + node.rhs.accept(self),
      Operation::Sub => node.lhs.accept(self) - node.rhs.accept(self),
      Operation::Mul => node.lhs.accept(self) * node.rhs.accept(self),
      Operation::Div => node.lhs.accept(self) / node.rhs.accept(self),
    }
  }
}
```

---

## Explanation:

the implementation of the pattern is split between 4 traits:

```rust
// Marks a type as a family of nodes
// and is responsible for routing the visitor to the appropriate visitor methods
pub trait NodeFamily<V> : Sized where V : Visitor<Self> {
  // The additional data we want to tag with the nodes
  type Data;
  // The method responsible for the routing
  fn accept(&self, v: &mut V) -> V::Output;
}

// Marks a type as being a node belonging to a family
pub trait Node<V> : Sized where V : Visitor<Self::Family> + Visit<Self> {
  type Family : NodeFamily<V>;

  fn accept(&self, v: &mut V, data: &Data<V, Self>) -> V::Output {
    v.visit(self, data)
  }
}

// Marks a type as being a visitor to a family of nodes
pub trait Visitor<F> : Sized where F : NodeFamily<Self> {
  // The output of performing this operation
  type Output;
}

// Implements the actual visiting logic for a specific node
// This is the only trait you'll need to implement manually
pub trait Visit<N> : Visitor<N::Family> where N : Node<Self> {
  fn visit(&mut self, node: &N, data: &Data<Self, N>) -> Self::Output;
}
```

the `node_group` macro will perform the following:
- extract the enum variants into their own structs;
- create a new enum which groups said structs;
- create a new struct which holds the node variant and the additional data;
- implement NodeFamily for said struct;
- implement Node for the struct variants;

```rust
#[node_group(data = ())]
enum Expr {
  NumLit(f32),
  Binary(Expr, Operation, Expr),
}

// Becomes:

struct NumLit(f32);

impl<V> Node<V> for NumLit
where V : Visitor<Expr> + Visit<NumLit> + Visit<Binary> {
  type Family = Expr;
}

impl NumLit {
  pub fn to_node(self, data: ()) -> Expr {
    Expr {
      node: ExprNode::NumLit(self),
      data,
    }
  }
}

struct Binary(Expr, Operation, Expr);

impl<V> Node<V> for Binary
where V : Visitor<Expr> + Visit<NumLit> + Visit<Binary> {
  type Family = Expr;
}

impl Binary {
  pub fn to_node(self, data: ()) -> Expr {
    Expr {
      node: Box::new(ExprNode::NumLit(self)),
      data,
    }
  }
}

enum ExprNode {
  NumLit(NumLit),
  Binary(Binary),
}

struct Expr {
  node: Box<ExprNode>,
  data: (),
}

impl<V> NodeFamily<V> for Expr
where V : Visitor<Expr> + Visit<NumLit> + Visit<Binary> {
  type Data = ();
  fn accept(&self, v: &mut V) -> V::Output {
    match self.node.as_ref() {
      ExprNode::NumLit(node) => v.visit(node, &self.data),
      ExprNode::Binary(node) => v.visit(node, &self.data),
    }
  }
}
```

To construct a node you'd use: `NumLit(23.0).to_node(())`.

The `visitor` macro simply implements the Visitor trait for a type:

```rust
#[visitor(Expr, output = f32)]
struct Interpreter;

// Becomes:

struct Interpreter;

impl Visitor<Expr> for Interpreter {
  type Output = f32;
}
```

Because of the bounds made by the `node_group` macro, marking a type as a `visitor` will also require that it implements `Visit` for every possible node inside that node family.
