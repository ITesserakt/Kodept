use crate::dag::container::SlotMapContainer;
use crate::dag::{Children, Node, NodeKey};
use crate::key::CommonKey;
use std::collections::VecDeque;

pub struct DagDetachIter<'a, T, C, E = ()>
where
    C: SlotMapContainer<Key=CommonKey, Data=Node<T, E>>
{
    arena: &'a mut C,
    stacks: VecDeque<StacksState<T, E>>,
}

struct StacksState<T, E> {
    parent: NodeKey,
    current_child: Node<T, E>,
    current_child_id: CommonKey,
}

impl<'a, T, C, E> DagDetachIter<'a, T, C, E>
where
    C: SlotMapContainer<Key=CommonKey, Data=Node<T, E>>
{
    pub fn new(arena: &'a mut C, head_id: NodeKey, current_children: Option<Children>) -> Self {
        let mut stacks = VecDeque::new();

        if let Some(children) = current_children {
            stacks.push_front(StacksState {
                parent: head_id,
                current_child: arena.remove(children.first).unwrap(),
                current_child_id: children.first,
            });
        }
        Self { arena, stacks }
    }
}

#[derive(Debug)]
pub struct DetachedNode<T, E> {
    pub parent_id: NodeKey,
    pub id: NodeKey,
    pub value: T,
    pub edge: E
}

impl<'a, T, C, E> Iterator for DagDetachIter<'a, T, C, E>
where
    C: SlotMapContainer<Key=CommonKey, Data=Node<T, E>>
{
    type Item = DetachedNode<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.stacks.pop_front()?;

        if let Some(next_sibling) = frame.current_child.next_sibling {
            self.stacks.push_front(StacksState {
                parent: frame.parent,
                current_child: self.arena.remove(next_sibling)?,
                current_child_id: next_sibling,
            });
        }

        if let Some(children) = frame.current_child.children {
            self.stacks.push_front(StacksState {
                parent: NodeKey::Child(frame.current_child_id),
                current_child: self.arena.remove(children.first)?,
                current_child_id: children.first,
            });
        }

        Some(DetachedNode {
            parent_id: frame.parent,
            id: NodeKey::Child(frame.current_child_id),
            value: frame.current_child.value,
            edge: frame.current_child.edge_data
        })
    }
}

impl<'a, T, C, E> Drop for DagDetachIter<'a, T, C, E>
where
    C: SlotMapContainer<Key=CommonKey, Data=Node<T, E>>
{
    fn drop(&mut self) {
        for _ in self {}
    }
}
