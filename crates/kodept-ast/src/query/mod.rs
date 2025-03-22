use crate::arity::{Optional, Plural, Singular};
use crate::prelude::{AnyNodeRefItem, NodeId};
use crate::properties::{Node, Root};
use crate::relationship::{ArityValue, Contains, NodeRelationships, RelationshipMetadata};
use bevy_ecs::component::Components;
use bevy_ecs::prelude::{Entity, EntityRef, Query, Single};
use bevy_ecs::ptr::Ptr;
use bevy_ecs::query::{QueryEntityError, With};
use bevy_ecs::relationship::RelationshipSourceCollection;
use bevy_ecs::system::SystemParam;
use std::iter::repeat;

#[derive(SystemParam)]
pub struct AnyNodeQuery<'w, 's> {
    query: Query<'w, 's, EntityRef<'static>, With<Node>>,
    root: Single<'w, Entity, With<Root>>,
    components: &'w Components,
}

pub(crate) struct TopDownIteratorWithMetadata<'w, 's> {
    stack: Vec<(Option<(Entity, RelationshipMetadata)>, Entity)>,
    nodes: Query<'w, 's, EntityRef<'static>, With<Node>>,
    components: &'w Components,
}

pub struct TopDownIterator<'w, 's> {
    stack: Vec<(Option<Entity>, Entity)>,
    nodes: Query<'w, 's, EntityRef<'static>, With<Node>>,
    components: &'w Components,
}

impl<'w, 's> AnyNodeQuery<'w, 's> {
    pub fn iter_top_down(&self) -> TopDownIterator<'_, 's> {
        TopDownIterator {
            stack: Vec::from([(None, *self.root)]),
            nodes: self.query.as_readonly(),
            components: self.components,
        }
    }
    
    pub fn iter_top_down_from(&self, id: NodeId) -> TopDownIterator<'_, 's> { 
        TopDownIterator {
            stack: vec![(None, id.entity())],
            nodes: self.query.as_readonly(),
            components: &self.components,
        }
    }
    
    pub(crate) fn iter_top_down_with_metadata(&self) -> TopDownIteratorWithMetadata<'_, 's> {
        TopDownIteratorWithMetadata {
            stack: vec![(None, *self.root)],
            nodes: self.query.as_readonly(),
            components: self.components,
        }
    } 

    pub fn get(&self, id: NodeId) -> Result<AnyNodeRefItem<'_, 'w>, QueryEntityError> {
        let reference = self.query.get(id.entity())?;
        let item = AnyNodeRefItem::from_inner(reference);
        Ok(item)
    }

    pub fn root(&self) -> AnyNodeRefItem<'_, 'w> {
        let reference = self.query.get(*self.root).unwrap();
        AnyNodeRefItem::from_inner(reference)
    }
}

impl<'w, 's> TopDownIteratorWithMetadata<'w, 's> {
    #[allow(unsafe_code)]
    fn extract_siblings(ptr: Ptr, arity: ArityValue) -> impl Iterator<Item=Entity> + '_ {
        enum Helper<'a> {
            A(<Entity as RelationshipSourceCollection>::SourceIter<'a>),
            B(<crate::arity::Option as RelationshipSourceCollection>::SourceIter<'a>),
            C(<Vec<Entity> as RelationshipSourceCollection>::SourceIter<'a>),
        }

        impl<'a> Iterator for Helper<'a> {
            type Item = Entity;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Helper::A(x) => x.next(),
                    Helper::B(x) => x.next(),
                    Helper::C(x) => x.next()
                }
            }
        }

        // SAFETY: There are only 3 arities, so it's safe to cast ptr to one of them.
        //         Also, tag does not matter because it's phantom type.
        match arity {
            ArityValue::Singular => Helper::A(unsafe { ptr.deref::<Contains<(), Singular>>() }.into_iter()),
            ArityValue::Optional => Helper::B(unsafe { ptr.deref::<Contains<(), Optional>>() }.into_iter()),
            ArityValue::Plural => Helper::C(unsafe { ptr.deref::<Contains<(), Plural>>() }.into_iter()),
        }
    }
}

impl<'w, 's> Iterator for TopDownIteratorWithMetadata<'w, 's> {
    /// (Parent, Node)
    type Item = (Option<(NodeId, RelationshipMetadata)>, NodeId);

    fn next(&mut self) -> Option<Self::Item> {
        let (parent, current) = self.stack.pop()?;
        let reference = self.nodes.get(current).unwrap();

        let iter = NodeRelationships
            .into_iter()
            .filter_map(|meta| {
                let component_id = meta.forward_component_id(self.components);
                let ptr = reference.get_by_id(component_id).ok()?;
                Some((ptr, meta))
            })
            .flat_map(|(ptr, meta)| Self::extract_siblings(ptr, meta.arity()).zip(repeat(meta)));

        for (sibling, meta) in iter {
            self.stack.push((Some((current, meta)), sibling));
        }

        Some((parent.map(|it| (it.0.into(), it.1)), current.into()))
    }
}

impl<'w, 's> Iterator for TopDownIterator<'w, 's> {
    type Item = (Option<NodeId>, NodeId);

    fn next(&mut self) -> Option<Self::Item> {
        let (parent, current) = self.stack.pop()?;
        let reference = self.nodes.get(current).unwrap();

        let iter = NodeRelationships
            .into_iter()
            .map(|meta| {
                let component_id = meta.forward_component_id(self.components);
                let ptr = reference.get_by_id(component_id).unwrap();
                (ptr, meta)
            })
            .flat_map(|(ptr, meta)| TopDownIteratorWithMetadata::extract_siblings(ptr, meta.arity()));

        for sibling in iter {
            self.stack.push((Some(current), sibling));
        }

        Some((parent.map(|it| it.into()), current.into()))
    }
}
