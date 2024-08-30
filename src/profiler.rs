use dhat::Profiler;
use std::sync::Mutex;
use tracing::error;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

static PROFILER: HeapProfiler = HeapProfiler::new();

pub struct HeapProfiler {
    inner: Mutex<Option<Profiler>>,
}

impl HeapProfiler {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn install() {
        PROFILER
            .inner
            .lock()
            .expect("Cannot install heap profiler")
            .replace(Profiler::new_heap());
    }

    pub fn consume() {
        let Ok(mut profiler) = PROFILER.inner.lock() else {
            error!("Cannot lock mutex");
            return;
        };
        let profiler = profiler.take();
        drop(profiler)
    }
}
