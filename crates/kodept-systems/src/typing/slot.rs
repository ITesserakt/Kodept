use crate::typing::{PartialInfer, PerFunctionScope};
use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::{Entity, Resource};
use bevy_tasks::AsyncComputeTaskPool;
use kodept_inference::process::Infer;
use oneshot::{Receiver, channel};
use std::convert::Infallible;
use std::sync::Mutex;
use tracing::warn;

#[derive(Debug)]
pub(super) struct Cancelled;

impl From<Infallible> for Cancelled {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug)]
pub(super) struct InferHandle {
    // mutex just makes [`oneshot::Receiver`] `Sync`
    receiver: Mutex<Receiver<PartialInfer>>,
}

impl InferHandle {
    pub(super) async fn recv(self) -> Result<PartialInfer, Cancelled> {
        let lock = self.receiver.into_inner();
        // lock will never be poisoned, because mutex is never locked
        let receiver = lock.unwrap_or_else(|e| e.into_inner());
        receiver.await.map_err(|_| Cancelled)
    }

    fn new(receiver: Receiver<PartialInfer>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
        }
    }
}

#[derive(Debug, Resource, Default)]
pub(super) struct InferHandles {
    handles: EntityHashMap<Option<InferHandle>>,
}

impl InferHandles {
    pub(super) fn new() -> Self {
        Self {
            handles: EntityHashMap::new(),
        }
    }

    pub(super) fn contains(&self, entity: Entity) -> bool {
        self.handles.contains_key(&entity)
    }

    pub(super) fn acquire_handle(&mut self, entity: Entity) -> Option<InferHandle> {
        self.handles.get_mut(&entity)?.take()
    }

    pub(super) fn spawn_infer_task<V>(&mut self, entity: Entity, corresponding_task: V)
    where
        PerFunctionScope: Infer<V, Entity, Error: Into<Cancelled>>,
        V: 'static,
    {
        let pool = AsyncComputeTaskPool::get();
        let (tx, rx) = channel();
        let fut = PerFunctionScope::partial_infer(corresponding_task);

        pool.spawn(async move {
            match fut.await {
                Ok(result) => tx.send(dbg!(result)).expect("Receiver dropped"),
                Err(_) => {
                    warn!("Infer task was cancelled");
                }
            };
        })
        .detach();

        self.handles.insert(entity, Some(InferHandle::new(rx)));
    }
}
