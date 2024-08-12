use dhat::Profiler;
use std::sync::{Mutex, OnceLock};
use tracing::error;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

static PROFILER: HeapProfiler = HeapProfiler::new();

pub struct HeapProfiler {
    inner: OnceLock<Mutex<Option<Profiler>>>,
}

impl HeapProfiler {
    const fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    pub fn install() {
        PROFILER
            .inner
            .set(Mutex::new(Some(Profiler::new_heap())))
            .expect("Cannot install heap profiler")
    }

    pub fn consume() {
        let Some(mutex) = PROFILER.inner.get() else {
            error!("Heap profiler is not installed");
            return;
        };
        let Ok(mut profiler) = mutex.lock() else {
            error!("Cannot lock mutex");
            return;
        };
        let profiler = profiler.take();
        drop(profiler)
    }
}
