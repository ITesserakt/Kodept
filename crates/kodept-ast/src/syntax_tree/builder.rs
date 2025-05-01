use crate::prelude::{ASTNode, Choose, CodeHolder, FromSyntax, NodeId};
use crate::properties::{Node, NodeProperty};
use crate::relationship::ContainedBy;
use crate::resource::rlt::SyntaxVariant;
use crate::syntax_tree::builder::queue::{BorrowedQueue, OwnedQueue, Queue};
use crate::syntax_tree::children::HasChild;
use crate::utils::{HasLength, IntoCommonIter};
use bevy_ecs::prelude::{Commands, Entity, World};
use bevy_ecs::relationship::Relationship;
pub use pool::*;
use std::marker::PhantomData;
use std::sync::LazyLock;

static SWITCH_TO_PARALLEL_THRESHOLD: LazyLock<usize> = LazyLock::new(|| 10);

mod pool {
    use crate::node_id::Erase;
    use crate::resource::rlt::{SyntaxResolver, SyntaxVariant};
    use bevy_ecs::entity::{Entities, Entity};

    #[derive(Debug, Copy, Clone)]
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

    impl<'w> Pool<'w> {
        pub(crate) fn entities(&self) -> &'w Entities {
            self.lazy_entities
        }

        pub(crate) fn syntax_root(&self) -> &'w kodept_rlt::prelude::File {
            self.syntax.root()
        }

        /// SAFETY: inherit
        #[allow(unsafe_code)]
        pub(crate) unsafe fn link_syntax(
            &self,
            id: impl Erase<Entity>,
            rlt_node: impl Into<SyntaxVariant<'w>>,
        ) {
            self.syntax.insert(id.erase(), rlt_node)
        }
    }
}

mod queue {
    use crate::syntax_tree::builder::Pool;
    use bevy_ecs::prelude::{Commands, World};
    use bevy_ecs::world::CommandQueue;

    pub trait Queue: Sized {
        fn push(&mut self, item: impl FnOnce(&mut World) + Send + 'static);
    }

    #[derive(Debug, Default)]
    pub struct OwnedQueue(pub(super) CommandQueue);
    pub struct BorrowedQueue<'w, 's, Source>(
        pub(super) Commands<'w, 's>,
        pub(super) Pool<'w>,
        pub(super) Source,
    );

    impl Queue for OwnedQueue {
        fn push(&mut self, item: impl FnOnce(&mut World) + Send + 'static) {
            self.0.push(item);
        }
    }

    impl<'a, 'b, Source> Queue for BorrowedQueue<'a, 'b, Source> {
        fn push(&mut self, item: impl FnOnce(&mut World) + Send + 'static) {
            self.0.queue(item);
        }
    }
}

#[derive(Debug)]
pub struct ASTBuilder<Root, Queue = OwnedQueue> {
    queue: Queue,
    root: Entity,
    _phantom: PhantomData<Root>,
}

pub struct ChildrenScope<'s, 'w, Root, Source> {
    root: Entity,
    commands: Commands<'w, 's>,
    pool: Pool<'w>,
    source: Source,
    _phantom: PhantomData<Root>,
}

impl<Root, Q> ASTBuilder<Root, Q>
where
    Q: Queue,
{
    #[must_use]
    pub fn with_property(mut self, property: impl NodeProperty) -> Self {
        let root = self.root;
        self.queue.push(move |w: &mut World| {
            w.entity_mut(root).insert(property);
        });
        self
    }

    pub(crate) fn id(&self) -> NodeId<Root> {
        self.root.into()
    }
    pub(crate) fn erase(self) -> ASTBuilder<(), Q> {
        ASTBuilder {
            queue: self.queue,
            root: self.root,
            _phantom: Default::default(),
        }
    }
}

impl<Root> ASTBuilder<Root> {
    #[must_use]
    pub fn new(pool: Pool, root: Root) -> Self
    where
        Root: ASTNode,
    {
        let mut queue = OwnedQueue::default();
        let mut commands = Commands::new_from_entities(&mut queue.0, pool.entities());

        let entity_commands = commands.spawn((
            root,
            Node {
                kind: std::any::type_name::<Root>(),
            },
        ));

        let root = entity_commands.id();
        Self {
            queue,
            root,
            _phantom: Default::default(),
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn with_children<'w, S>(
        mut self,
        source: S,
        pool: Pool<'w>,
        f: impl for<'scope> FnOnce(&'scope mut ChildrenScope<'scope, 'w, Root, S>),
    ) -> Self
    where
        S: CodeHolder,
    {
        let mut scope = ChildrenScope {
            source,
            commands: Commands::new_from_entities(&mut self.queue.0, pool.entities()),
            root: self.root,
            pool,
            _phantom: Default::default(),
        };
        f(&mut scope);
        self
    }

    pub(crate) fn consume(mut self, world: &mut World) {
        self.queue.0.apply(world)
    }
}

impl<'w, 's, Root, Source> ASTBuilder<Root, BorrowedQueue<'w, 's, Source>> {
    pub fn from_queue(mut queue: BorrowedQueue<'w, 's, Source>, root: Root) -> Self
    where
        Root: ASTNode,
    {
        let entity_commands = queue.0.spawn((
            root,
            Node {
                kind: std::any::type_name::<Root>(),
            },
        ));
        let root = entity_commands.id();
        Self {
            queue,
            root,
            _phantom: Default::default(),
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn with_children(
        mut self,
        f: impl for<'scope> FnOnce(&'scope mut ChildrenScope<'scope, 'w, Root, Source>),
    ) -> Self
    where
        Source: CodeHolder,
    {
        let mut scope = ChildrenScope {
            source: self.queue.2,
            commands: self.queue.0.reborrow(),
            pool: self.queue.1,
            root: self.root,
            _phantom: Default::default(),
        };
        f(&mut scope);
        self
    }
}

impl<'s, 'w, Root, Source> ChildrenScope<'s, 'w, Root, Source>
where
    Source: CodeHolder,
{
    #[inline(always)]
    fn insert<R>(&mut self, mut part: ASTBuilder<()>)
    where
        R: Relationship,
    {
        let child_id = part.root;
        let root_id = self.root;

        self.commands.entity(root_id).add_one_related::<R>(child_id);
        self.commands.append(&mut part.queue.0);
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    pub fn many<T, U, Tag>(&mut self, iter: impl IntoCommonIter<Item = &'w T> + HasLength)
    where
        Root: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        Tag: Send + Sync + 'static,
        &'w T: Into<SyntaxVariant<'w>>,
        T: 'w
    {
        Root::register();

        for item in iter.into_iter() {
            let part = U::from_syntax(item, self.source);
            // unsafe { self.pool.link_syntax(part.root, item) };
            // self.insert::<Root::Relationship>(part.erase());
        }
    }

    #[inline(always)]
    pub fn maybe_many<T, U, Tag>(
        &mut self,
        option: Option<impl IntoCommonIter<Item = &'w T> + HasLength>,
    ) where
        Root: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        Tag: Send + Sync + 'static,
        &'w T: Into<SyntaxVariant<'w>>,
        T: 'w
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
        Tag: Send + Sync + 'static,
        T: 'a,
        'a: 's,
    {
        if cfg!(not(feature = "parallel")) || iter.len() < *SWITCH_TO_PARALLEL_THRESHOLD {
            for item in iter.into_iter() {
                let disjoint = Chooser::branch(item);
                let part = disjoint.call(self.source, self.pool);
                self.insert::<ContainedBy<Tag, Chooser::Arity>>(part);
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
                        let part = disjoint.call(source, pool);
                        sender.send(part).unwrap()
                    })
                },
                move || {
                    for item in rx.into_iter() {
                        self.insert::<ContainedBy<Tag, Chooser::Arity>>(item);
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
        Tag: Send + Sync + 'static,
        T: 'a,
        'a: 's,
    {
        if let Some(iter) = option {
            self.choose(chooser, iter)
        }
    }

    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn with_builder<T, F, U, Tag>(&mut self, node: &'w T, callback: F)
    where
        &'w T: Into<SyntaxVariant<'w>>,
        F: for<'t> FnOnce(
            BorrowedQueue<'w, 't, Source>,
        ) -> ASTBuilder<U, BorrowedQueue<'w, 't, Source>>,
        Tag: Send + Sync + 'static,
        Root: HasChild<U, Tag>,
        U: ASTNode,
    {
        Root::register();
        let mut builder = callback(BorrowedQueue(
            self.commands.reborrow(),
            self.pool,
            self.source,
        ));
        unsafe { self.pool.link_syntax(builder.root, node) };
        builder
            .queue
            .0
            .entity(self.root)
            .add_one_related::<Root::Relationship>(builder.root);
    }

    #[inline(always)]
    pub fn pool(&self) -> Pool<'w> {
        self.pool
    }

    #[inline(always)]
    pub fn source(&self) -> Source {
        self.source
    }
}
