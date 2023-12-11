use crate::graph::generic_node::GenericASTNode;
use crate::graph::Graph;
use crate::visitor::visit_side::VisitSide;
use crate::AST;
use petgraph::prelude::NodeIndex;
use petgraph::Direction;

pub struct ASTVisitor<'a, F, E>
where
    F: FnMut(&mut GenericASTNode, VisitSide) -> Result<(), E>,
{
    tree: &'a mut Graph,
    handler: F,
}

impl<'a, F, E> ASTVisitor<'a, F, E>
where
    F: FnMut(&mut GenericASTNode, VisitSide) -> Result<(), E>,
{
    pub fn new(ast: &'a mut AST, handler: F) -> Self {
        Self {
            tree: &mut ast.0.inner,
            handler,
        }
    }

    pub fn apply(mut self) -> Result<(), E> {
        let mut roots = self.tree.externals(Direction::Incoming);
        let Some(root_id) = roots.next() else {
            panic!("AST should always has only one root")
        };
        let mut order = vec![];
        dfs(&*self.tree, root_id, &mut |x, y| order.push((x, y)));
        for (id, side) in order {
            let node = &mut self.tree[id];
            (self.handler)(node, side)?;
        }
        Ok(())
    }
}

fn dfs<F>(graph: &Graph, current: NodeIndex<usize>, callback: &mut F)
where
    F: FnMut(NodeIndex<usize>, VisitSide),
{
    let children: Vec<_> = graph
        .neighbors_directed(current, Direction::Outgoing)
        .collect();
    if children.is_empty() {
        callback(current, VisitSide::Leaf)
    } else {
        callback(current, VisitSide::Entering);
        for child in children.into_iter().rev() {
            dfs(graph, child, callback);
        }
        callback(current, VisitSide::Exiting);
    }
}
