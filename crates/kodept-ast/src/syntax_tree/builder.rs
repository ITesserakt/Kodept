use crate::prelude::{ASTNode, Choose, CodeHolder, FromSyntax, NodeId};
use crate::properties::tags::Tagged;
use crate::properties::{Node, NodeProperty};
use crate::resource::rlt::{SyntaxResolver, SyntaxVariant};
use crate::syntax_tree::children::HasChild;
use crate::utils::{HasLength, IntoCommonIter};
use bevy_ecs::entity::Entities;
use bevy_ecs::prelude::{Entity, World};
use bevy_ecs::world::CommandQueue;
use bevy_hierarchy::BuildChildren;
use std::cell::OnceCell;
use std::marker::PhantomData;
use std::sync::LazyLock;

static SWITCH_TO_PARALLEL_THRESHOLD: LazyLock<usize> =
    LazyLock::new(|| match std::thread::available_parallelism() {
        Ok(x) => x.get(),
        Err(_) => 4,
    });

pub struct Pool<'e> {
    syntax: &'e SyntaxResolver,
    lazy_entities: &'e Entities,
}

impl<'e> Pool<'e> {
    pub fn new(syntax: &'e SyntaxResolver, lazy_entities: &'e Entities) -> Self {
        Self {
            syntax,
            lazy_entities,
        }
    }
}

impl Pool<'_> {
    pub(crate) fn allocate(&self) -> Entity {
        self.lazy_entities.reserve_entity()
    }

    pub(crate) fn syntax_root(&self) -> &kodept_rlt::prelude::File {
        self.syntax.root()
    }

    pub(crate) fn link_syntax<'r>(&'r self, id: NodeId, rlt_node: impl Into<SyntaxVariant<'r>>) {
        self.syntax.insert(id, rlt_node)
    }
}

pub struct ASTBuilder<Root> {
    queue: CommandQueue,
    root: Entity,
    _phantom: PhantomData<Root>,
}

pub struct ChildrenScope<'p, 'e, Root, Source> {
    children_buffer: Vec<Entity>,
    queue: OnceCell<CommandQueue>,
    root: Entity,
    source: Source,
    pool: &'p Pool<'e>,
    _phantom: PhantomData<Root>,
}

impl<Root> ASTBuilder<Root> {
    #[must_use]
    pub fn new(pool: &Pool, root: Root) -> Self
    where
        Root: ASTNode,
    {
        let root_id = pool.allocate();
        let mut queue = CommandQueue::default();
        queue.push(move |w: &mut World| {
            let mut entity = w.entity_mut(root_id);
            entity.insert((root, Node));
        });
        Self {
            queue,
            root: root_id,
            _phantom: Default::default(),
        }
    }

    #[must_use]
    pub fn with_property(mut self, property: impl NodeProperty) -> Self {
        let root = self.root;
        self.queue.push(move |w: &mut World| {
            w.entity_mut(root).insert(property);
        });
        self
    }

    #[must_use]
    #[inline(always)]
    pub fn with_children<'p, 'e, S>(
        mut self,
        source: S,
        pool: &'p Pool<'e>,
        f: impl FnOnce(&mut ChildrenScope<'p, 'e, Root, S>),
    ) -> Self
    where
        S: CodeHolder,
    {
        let mut scope = ChildrenScope {
            source,
            pool,
            children_buffer: Default::default(),
            queue: OnceCell::new(),
            root: self.root,
            _phantom: Default::default(),
        };
        f(&mut scope);
        let root = self.root;
        if let Some(queue) = scope.queue.get_mut() {
            self.queue.append(queue);
        }
        self.queue.push(move |w: &mut World| {
            w.entity_mut(root).add_children(&scope.children_buffer);
        });
        self
    }

    pub(crate) fn consume(mut self, world: &mut World) {
        self.queue.apply(world)
    }
    pub(crate) fn into_inner(self) -> CommandQueue {
        self.queue
    }
    pub(crate) fn erase(self) -> ASTBuilder<()> {
        ASTBuilder {
            queue: self.queue,
            root: self.root,
            _phantom: Default::default(),
        }
    }
    pub(crate) fn id(&self) -> NodeId {
        NodeId::from_inner(self.root)
    }
}

impl<'p, 'e, Root, Source> ChildrenScope<'p, 'e, Root, Source>
where
    Source: CodeHolder,
{
    #[inline(always)]
    fn insert<Tag>(&mut self, mut part: ASTBuilder<()>, tag: Tag)
    where
        Tag: Tagged,
    {
        let child_id = part.root;
        let root_id = self.root;
        part.queue.push(move |w: &mut World| {
            w.entity_mut(child_id).insert(tag).insert_if_new(Node);
            w.entity_mut(root_id).add_child(child_id);
        });
        match self.queue.take() {
            None => self.queue.set(part.queue).unwrap(),
            Some(mut old) => {
                old.append(&mut part.queue);
                self.queue.set(old).unwrap()
            }
        }
    }

    #[inline(always)]
    pub fn many<'t, U, Tag>(&mut self, iter: impl IntoCommonIter<Item = &'t U::Syntax> + HasLength)
    where
        Root: HasChild<U, Tag>,
        U: ASTNode + FromSyntax,
        Tag: Tagged,
        &'t U::Syntax: Into<SyntaxVariant<'p>>,
    {
        if cfg!(not(feature = "parallel")) || iter.len() < *SWITCH_TO_PARALLEL_THRESHOLD {
            for item in iter.into_iter() {
                let part = U::from_syntax(item, self.source, self.pool);
                self.children_buffer.push(part.root);
                self.pool.link_syntax(NodeId::from_inner(part.root), item);
                self.insert::<Tag>(part.erase(), Tag::default());
            }
            return;
        }

        #[cfg(not(feature = "parallel"))]
        unreachable!();

        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;

            let (sx, rx) = std::sync::mpsc::channel();
            let iter = iter.into_par_iter();
            let source = self.source;
            let pool = self.pool;

            rayon::join(
                move || {
                    iter.for_each_with(sx, |sender, it| {
                        let part = U::from_syntax(it, source, pool);
                        pool.link_syntax(NodeId::from_inner(part.root), it);
                        sender.send(part).unwrap()
                    })
                },
                || {
                    for part in rx {
                        self.children_buffer.push(part.root);
                        self.insert(part.erase(), Tag::default());
                    }
                },
            );
        }
    }

    #[inline(always)]
    pub fn maybe_many<'t, U, Tag>(
        &mut self,
        option: Option<impl IntoCommonIter<Item = &'t U::Syntax> + HasLength>,
    ) where
        Root: HasChild<U, Tag>,
        U: ASTNode + FromSyntax,
        Tag: Tagged,
        &'t U::Syntax: Into<SyntaxVariant<'p>>,
    {
        if let Some(iter) = option {
            self.many(iter)
        }
    }

    #[inline(always)]
    pub fn choose<'a, T, Chooser, Tag>(
        &mut self,
        _chooser: Chooser,
        iter: impl IntoCommonIter<Item = &'a T> + HasLength,
    ) where
        Root: Send,
        Chooser: Choose<T, Root, Tag>,
        Tag: Tagged,
        T: 'a,
        'a: 'p,
    {
        if cfg!(not(feature = "parallel")) || iter.len() < *SWITCH_TO_PARALLEL_THRESHOLD {
            for item in iter.into_iter() {
                let disjoint = Chooser::branch(item);
                let part = (disjoint.conversion)(disjoint.inner, self.source, self.pool);
                self.pool
                    .link_syntax(NodeId::from_inner(part.root), disjoint.inner);
                self.children_buffer.push(part.root);
                self.insert(part.erase(), Tag::default());
            }
            return;
        }

        #[cfg(not(feature = "parallel"))]
        unreachable!();

        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;

            let (sx, rx) = std::sync::mpsc::channel();
            let iter = iter.into_par_iter();
            let source = self.source;
            let pool = self.pool;

            rayon::join(
                move || {
                    iter.for_each_with(sx, |sender, it| {
                        let disjoint = Chooser::branch(it);
                        let part = (disjoint.conversion)(disjoint.inner, source, pool);
                        pool.link_syntax(NodeId::from_inner(part.root), disjoint.inner);
                        sender.send(part).unwrap()
                    })
                },
                move || {
                    for part in rx {
                        self.children_buffer.push(part.root);
                        self.insert(part.erase(), Tag::default());
                    }
                },
            );
        }
    }

    #[inline(always)]
    pub fn maybe_choose<'a, T, Chooser, Tag>(
        &mut self,
        chooser: Chooser,
        option: Option<impl IntoCommonIter<Item = &'a T> + HasLength>,
    ) where
        Root: Send,
        Chooser: Choose<T, Root, Tag>,
        Tag: Tagged,
        T: 'a,
        'a: 'p,
    {
        if let Some(iter) = option {
            self.choose(chooser, iter)
        }
    }

    #[allow(clippy::wrong_self_convention)]
    #[inline(always)]
    pub fn from_builder<'a, T, U, Tag>(&mut self, node: &'a T, builder: ASTBuilder<U>)
    where
        Root: HasChild<U, Tag>,
        U: ASTNode,
        Tag: Tagged,
        &'a T: Into<SyntaxVariant<'p>>,
    {
        self.children_buffer.push(builder.root);
        self.pool
            .link_syntax(NodeId::from_inner(builder.root), node);
        self.insert(builder.erase(), Tag::default());
    }

    #[inline(always)]
    pub fn pool(&self) -> &'p Pool<'e> {
        self.pool
    }

    #[inline(always)]
    pub fn source(&self) -> Source {
        self.source
    }
}
