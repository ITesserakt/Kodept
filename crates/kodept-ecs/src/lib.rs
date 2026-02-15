//! This crate exposes a subset of bevy_ecs features to narrow Public API
//! and support future rewrite using custom primitives

pub mod archetype {
    pub use bevy_ecs::archetype::{Archetype, Archetypes};
}
pub mod bundle {
    pub use bevy_ecs::bundle::{
        Bundle, BundleFromComponents, BundleId, BundleInfo, DynamicBundle, InsertMode,
        NoBundleEffect,
    };
}
pub mod component {
    pub use bevy_ecs::component::{
        Component, ComponentId, ComponentIdFor, ComponentInfo, ComponentMutability, Components,
        Immutable, Mutable,
    };
}
pub mod change_detection {
    pub use bevy_ecs::change_detection::{MaybeLocation, Tick};
}
pub mod entity {
    pub use bevy_ecs::entity::{
        ContainsEntity, Entity, EntityEquivalent, EntityMapper, MapEntities,
    };
}
pub mod error {
    pub use bevy_ecs::error::{CommandWithEntity, ErrorContext, HandleError};
}
pub mod event {
    pub use bevy_ecs::event::{EntityEvent, Event, Trigger};
}
pub mod hierarchy {
    pub use bevy_ecs::hierarchy::{ChildOf, Children};
}
pub mod lifecycle {
    pub use bevy_ecs::lifecycle::{Add, Despawn, HookContext, Insert, Remove, Replace};
}
pub mod ptr {
    pub use bevy_ecs::ptr::{MovingPtr, Ptr, PtrMut};
}
pub mod query {
    pub use bevy_ecs::query::{
        Access, AnyOf, ArchetypeQueryData, EcsAccessType, FilteredAccess, Has, Or, QueryData,
        QueryEntityError, QueryFilter, QueryItem, QueryManyIter, ROQueryItem, ReadOnlyQueryData,
        ReleaseStateQueryData, With, Without, WorldQuery,
    };
}
pub mod relationship {
    pub use bevy_ecs::relationship::{
        OrderedRelationshipSourceCollection, Relationship, RelationshipSourceCollection,
        RelationshipTarget,
    };
}
pub mod resource {
    pub use bevy_ecs::resource::Resource;
}
pub mod schedule {
    pub use bevy_ecs::schedule::{
        ExecutorKind, IntoScheduleConfigs, IntoSystemSet, Schedule, ScheduleConfigs, ScheduleLabel,
        Schedules, SystemSet,
    };
}
pub mod storage {
    pub use bevy_ecs::storage::{SparseSet, Storages, Table, TableRow};
}
pub mod system {
    pub use bevy_ecs::observer::{Observer, On};
    pub use bevy_ecs::system::entity_command::{EntityCommand, EntityCommandError};
    pub use bevy_ecs::system::{
        Adapt, Command, Commands, Deferred, EntityCommands, If, In, InMut, IntoObserverSystem,
        IntoSystem, Local, NonSend, NonSendMut, Populated, Query, QueryLens, ReadOnlySystem,
        ReadOnlySystemParam, Res, ResMut, ScheduleSystem, Single, StaticSystemParam, SystemBuffer,
        SystemInput, SystemMeta, SystemParam, SystemParamItem, SystemState,
    };

    #[cfg(feature = "parallel")]
    pub use bevy_ecs::system::ParallelCommands;
}
pub mod world {
    pub use bevy_ecs::world::error::EntityMutableFetchError;
    pub use bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell;
    pub use bevy_ecs::world::{
        DeferredWorld, EntityMut, EntityRef, EntityWorldMut, FromWorld, Mut, Ref, World,
    };
}

pub mod tasks {
    pub use bevy_tasks::{
        AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, TaskPool, TaskPoolBuilder,
    };
}
pub mod utils {
    pub use bevy_utils::prelude::{DebugName, ShortName};
    pub use bevy_utils::{PreHashMap, PreHashMapExt, TypeIdMap, TypeIdMapExt};

    #[cfg(feature = "parallel")]
    pub use bevy_utils::Parallel;
}

/// Use this module *only* to bring `bevy_ecs` crate into scope _for macros_
#[doc(hidden)]
pub mod exported {
    pub use bevy_ecs;
}
