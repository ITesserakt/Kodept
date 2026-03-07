use kodept_ecs::bundle::Bundle;
use kodept_ecs::entity::Entity;
use kodept_ecs::exported::bevy_ecs::error::HandleError;
use kodept_ecs::system::{Command, Commands, EntityCommands};
use kodept_ecs::world::{EntityWorldMut, World};

pub trait Buffer {
    type Reborrowed<'a>: Buffer
    where
        Self: 'a;

    fn reborrow(&mut self) -> Self::Reborrowed<'_>;

    #[track_caller]
    fn spawn(self, bundle: impl Bundle) -> (Entity, Self);
    fn queue<T>(self, action: impl Command<T> + HandleError<T>) -> Self;
}

pub trait RefBuffer: Buffer {
    fn borrow(&self) -> Self::Reborrowed<'_>;
}

impl<B: Buffer> Buffer for &mut B {
    type Reborrowed<'a>
        = &'a mut B
    where
        Self: 'a;

    #[inline(always)]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        self
    }

    #[inline]
    fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
        let buffer = B::reborrow(self);
        let (id, _) = buffer.spawn(bundle);
        (id, self)
    }

    #[inline]
    fn queue<T>(self, action: impl Command<T> + HandleError<T>) -> Self {
        let buffer = B::reborrow(self);
        let _ = buffer.queue(action);
        self
    }
}

impl<B: RefBuffer> Buffer for &B {
    type Reborrowed<'a>
        = &'a B
    where
        Self: 'a;

    #[inline(always)]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        self
    }

    #[inline]
    fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
        let buffer = B::borrow(self);
        let (id, _) = buffer.spawn(bundle);
        (id, self)
    }

    #[inline]
    fn queue<T>(self, action: impl Command<T> + HandleError<T>) -> Self {
        let buffer = B::borrow(self);
        let _ = buffer.queue(action);
        self
    }
}

impl<B: RefBuffer> RefBuffer for &B {
    #[inline(always)]
    fn borrow(&self) -> Self::Reborrowed<'_> {
        self
    }
}

impl<'w, 's> Buffer for Commands<'w, 's> {
    type Reborrowed<'a>
        = Commands<'w, 'a>
    where
        Self: 'a;

    #[inline]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        Commands::reborrow(self)
    }

    #[inline]
    fn spawn(mut self, bundle: impl Bundle) -> (Entity, Self) {
        let id = Commands::spawn(&mut self, bundle).id();
        (id, self)
    }

    #[inline]
    fn queue<T>(mut self, action: impl Command<T> + HandleError<T>) -> Self {
        Commands::queue(&mut self, action);
        self
    }
}

pub struct ReborrowedWorld<'w>(&'w mut World);

impl Buffer for World {
    type Reborrowed<'b> = ReborrowedWorld<'b>;

    #[inline]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        ReborrowedWorld(self)
    }

    #[inline]
    fn spawn(mut self, bundle: impl Bundle) -> (Entity, Self) {
        let id = World::spawn(&mut self, bundle).id();
        (id, self)
    }

    #[inline]
    fn queue<T>(mut self, action: impl Command<T> + HandleError<T>) -> Self {
        action.handle_error().apply(&mut self);
        self
    }
}

impl Buffer for ReborrowedWorld<'_> {
    type Reborrowed<'a>
        = ReborrowedWorld<'a>
    where
        Self: 'a;

    #[inline]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        ReborrowedWorld(self.0)
    }

    #[inline]
    fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
        let id = self.0.spawn(bundle).id();
        (id, self)
    }

    #[inline]
    fn queue<T>(self, action: impl Command<T> + HandleError<T>) -> Self {
        action.handle_error().apply(self.0);
        self
    }
}

impl Buffer for EntityCommands<'_> {
    type Reborrowed<'a>
        = EntityCommands<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        EntityCommands::reborrow(self)
    }

    #[inline]
    fn spawn(mut self, bundle: impl Bundle) -> (Entity, Self) {
        let mut buffer = self.commands();
        let id = Commands::spawn(&mut buffer, bundle).id();
        (id, self)
    }

    #[inline]
    fn queue<T>(mut self, action: impl Command<T> + HandleError<T>) -> Self {
        let buffer = self.commands();
        let _ = buffer.queue(action);
        self
    }
}

pub struct ReborrowedEntityWorldMut<'a, 'w>(&'a mut EntityWorldMut<'w>);

impl<'w> Buffer for EntityWorldMut<'w> {
    type Reborrowed<'a>
        = ReborrowedEntityWorldMut<'a, 'w>
    where
        Self: 'a;

    #[inline(always)]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        ReborrowedEntityWorldMut(self)
    }

    #[inline]
    fn spawn(mut self, bundle: impl Bundle) -> (Entity, Self) {
        let id = self.world_scope(|w| w.spawn(bundle).id());
        (id, self)
    }

    #[inline]
    fn queue<T>(mut self, action: impl Command<T> + HandleError<T>) -> Self {
        self.world_scope(|w| {
            w.queue(action);
        });
        self
    }
}

impl<'s, 'w> Buffer for ReborrowedEntityWorldMut<'s, 'w> {
    type Reborrowed<'a>
        = ReborrowedEntityWorldMut<'a, 'w>
    where
        Self: 'a;

    #[inline(always)]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        ReborrowedEntityWorldMut(self.0)
    }

    #[inline]
    fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
        let id = self.0.world_scope(|w| w.spawn(bundle).id());
        (id, self)
    }

    #[inline]
    fn queue<T>(self, action: impl Command<T> + HandleError<T>) -> Self {
        self.0.world_scope(|w| {
            w.queue(action);
        });
        self
    }
}

#[cfg(feature = "parallel")]
mod parallel {
    use crate::syntax_tree::buffer::{Buffer, RefBuffer};
    use kodept_ecs::bundle::Bundle;
    use kodept_ecs::entity::Entity;
    use kodept_ecs::error::HandleError;
    use kodept_ecs::system::{Command, Commands, ParallelCommands};

    pub struct BorrowedParallelCommands<'w, 's>(&'s ParallelCommands<'w, 's>);

    impl<'w, 's> Buffer for ParallelCommands<'w, 's> {
        type Reborrowed<'a>
            = BorrowedParallelCommands<'w, 'a>
        where
            Self: 'a;

        #[inline]
        fn reborrow(&mut self) -> Self::Reborrowed<'_> {
            BorrowedParallelCommands(self)
        }

        #[inline]
        fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
            let id = self.command_scope(|mut c| Commands::spawn(&mut c, bundle).id());
            (id, self)
        }

        #[inline]
        fn queue<T>(self, action: impl Command<T> + HandleError<T>) -> Self {
            self.command_scope(|c| {
                c.queue(action);
            });
            self
        }
    }

    impl<'w, 's> RefBuffer for ParallelCommands<'w, 's> {
        #[inline]
        fn borrow(&self) -> Self::Reborrowed<'_> {
            BorrowedParallelCommands(self)
        }
    }

    impl<'w, 's> Buffer for BorrowedParallelCommands<'w, 's> {
        type Reborrowed<'a>
            = BorrowedParallelCommands<'w, 'a>
        where
            Self: 'a;

        #[inline]
        fn reborrow(&mut self) -> Self::Reborrowed<'_> {
            BorrowedParallelCommands(self.0)
        }

        #[inline]
        fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
            let id = self
                .0
                .command_scope(|mut c| Commands::spawn(&mut c, bundle).id());
            (id, self)
        }

        #[inline]
        fn queue<T>(self, action: impl Command<T> + HandleError<T>) -> Self {
            self.0.command_scope(|mut c| {
                Commands::queue(&mut c, action);
            });
            self
        }
    }

    impl<'w, 's> RefBuffer for BorrowedParallelCommands<'w, 's> {
        #[inline]
        fn borrow(&self) -> Self::Reborrowed<'_> {
            BorrowedParallelCommands(self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax_tree::buffer::Buffer;
    use kodept_ecs::component::Component;
    use kodept_ecs::exported::bevy_ecs;
    use kodept_ecs::system::Command;
    use kodept_ecs::world::World;

    #[derive(Debug, Component)]
    struct Success;

    struct SampleCommand;

    impl Command for SampleCommand {
        fn apply(self, world: &mut World) -> () {
            world.spawn(Success);
        }
    }

    struct AssertCommand(usize);

    impl Command for AssertCommand {
        fn apply(self, world: &mut World) -> () {
            let mut query = world.query::<&Success>();
            assert_eq!(query.iter(world).count(), self.0);
        }
    }

    #[test]
    fn test_buffer() {
        let mut buffer = World::new();

        buffer.reborrow().queue(SampleCommand);
        buffer.queue(AssertCommand(1));
    }

    #[test]
    fn test_buffer_mut() {
        let mut world = World::new();
        {
            let mut commands = world.commands();
            let mut buffer = &mut commands;

            (&mut buffer).queue(SampleCommand);
            (&mut buffer).queue(SampleCommand);
        }
        world.flush();
        AssertCommand(2).apply(&mut world);
    }

    #[test]
    #[cfg(feature = "parallel")]
    fn test_buffer_ref() {
        use kodept_ecs::system::{ParallelCommands, SystemState};

        let mut world = World::new();
        let mut state = SystemState::<ParallelCommands>::new(&mut world);
        {
            let buffer = state.get_mut(&mut world);

            for _ in 0..10 {
                (&buffer).queue(SampleCommand);
            }
        }
        state.apply(&mut world);
        AssertCommand(10).apply(&mut world);
    }
}
