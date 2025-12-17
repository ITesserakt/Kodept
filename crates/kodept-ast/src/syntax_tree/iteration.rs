use crate::arity::{Optional, Plural, Singular};
use crate::properties::{Node, Root};
use crate::relationship::{ArityValue, Contains, NodeRelationships, RelationshipMetadata};
use bevy_ecs::prelude::{Entity, EntityRef, Query, Res, Single, With};
use bevy_ecs::system::SystemParam;
use std::fmt::{Debug, Formatter};

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum NodeSlot<'a> {
    Root(EntityRef<'a>),
    Inner {
        parent_id: Entity,
        edge_metadata: RelationshipMetadata,
        node_ref: EntityRef<'a>,
    },
}

#[derive(SystemParam)]
pub struct AllNodesQuery<'w, 's> {
    root: Single<'w, 's, Entity, With<Root>>,
    nodes: Query<'w, 's, EntityRef<'static>, With<Node>>,
    relationships: Res<'w, NodeRelationships>,
}

pub struct AllNodesQueryIter<'a, 'w, 's> {
    stack: Vec<(Option<(Entity, RelationshipMetadata)>, Entity)>,
    nodes: &'a Query<'w, 's, EntityRef<'static>, With<Node>>,
    relationships: &'a NodeRelationships,
}

impl<'w, 's> AllNodesQuery<'w, 's> {
    pub fn iter<'ss>(&'ss self) -> AllNodesQueryIter<'ss, 'w, 's> {
        AllNodesQueryIter {
            stack: vec![(None, *self.root)],
            nodes: &self.nodes,
            relationships: self.relationships.as_ref(),
        }
    }
}

impl<'a, 'w, 's> Iterator for AllNodesQueryIter<'a, 'w, 's>
where
    'a: 'w,
    's: 'w,
{
    type Item = NodeSlot<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        let (edge, current) = self.stack.pop()?;
        let current_ref = self.nodes.get(current).ok()?;

        for meta in self.relationships.into_iter() {
            let relationship_component_id = meta.forward_component_id();
            let Ok(relationship_component) = current_ref.get_by_id(relationship_component_id)
            else {
                continue;
            };

            type Rel<A> = Contains<(), A>;
            #[allow(unsafe_code)]
            match meta.arity() {
                ArityValue::Singular => {
                    for child in unsafe { relationship_component.deref::<Rel<Singular>>() }
                        .into_iter()
                        .rev()
                    {
                        self.stack.push((Some((current, meta)), child));
                    }
                }
                ArityValue::Optional => {
                    for child in unsafe { relationship_component.deref::<Rel<Optional>>() }
                        .into_iter()
                        .rev()
                    {
                        self.stack.push((Some((current, meta)), child));
                    }
                }
                ArityValue::Plural => {
                    for child in unsafe { relationship_component.deref::<Rel<Plural>>() }
                        .into_iter()
                        .rev()
                    {
                        self.stack.push((Some((current, meta)), child));
                    }
                }
            };
        }

        match edge {
            None => Some(NodeSlot::Root(current_ref)),
            Some((parent_id, meta)) => Some(NodeSlot::Inner {
                node_ref: current_ref,
                edge_metadata: meta,
                parent_id,
            }),
        }
    }
}

impl<'a> Debug for NodeSlot<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeSlot::Root(_) => f.debug_tuple("NodeSlot::Root").finish_non_exhaustive(),
            NodeSlot::Inner {
                parent_id,
                edge_metadata,
                ..
            } => f
                .debug_struct("NodeSlot::Inner")
                .field("parent_id", parent_id)
                .field("edge_metadata", edge_metadata)
                .finish_non_exhaustive(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::arity::{Plural, Singular};
    use crate::properties::{Node, Root};
    use crate::relationship::Contains;
    use crate::syntax_tree::iteration::{AllNodesQuery, NodeSlot};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use kodept_core::file_name::{FileDescriptor, FileId, FileName};

    #[derive(Debug, Component, PartialEq)]
    #[require(Node { kind: std::any::type_name::<Self>() })]
    struct A(usize);

    #[test]
    fn test_nodes_iteration() {
        let mut world = World::new();

        world.spawn((
            A(1),
            Root {
                associated_file: FileDescriptor::new(FileName::Anon, FileId::generate()),
            },
            related!(
                Contains < (),
                Plural > [A(2), (A(3), related!(Contains < bool, Singular > [A(4)]))]
            ),
        ));

        world.run_system_once(system).unwrap();
    }

    fn system(all_nodes: AllNodesQuery) {
        let nodes = all_nodes
            .iter()
            .map(|it| match it {
                NodeSlot::Root(x) => (None, None, x.components::<&A>(), x.id()),
                NodeSlot::Inner {
                    parent_id,
                    edge_metadata,
                    node_ref,
                } => (
                    Some(parent_id),
                    Some(edge_metadata),
                    node_ref.components::<&A>(),
                    node_ref.id(),
                ),
            })
            .collect::<Vec<_>>();

        assert_eq!(nodes.len(), 4);

        assert!(matches!(nodes[0], (None, None, A(1), _)), "{:?}", &nodes[0]);
        assert!(
            matches!(nodes[1], (Some(_), Some(_), A(2), _)),
            "{:?}",
            &nodes[1]
        );
        assert!(
            matches!(nodes[2], (Some(_), Some(_), A(3), _)),
            "{:?}",
            &nodes[2]
        );
        assert!(
            matches!(nodes[3], (Some(_), Some(_), A(4), _)),
            "{:?}",
            &nodes[3]
        );

        assert_eq!(nodes[1].0, Some(nodes[0].3));
        assert_eq!(nodes[2].0, Some(nodes[0].3));
        assert_eq!(nodes[3].0, Some(nodes[2].3));
    }
}
