use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[global_allocator]
#[cfg(feature = "profiler")]
static ALLOC: dhat::Alloc = dhat::Alloc;

static PROFILER: HeapProfiler = HeapProfiler::new();

#[derive(Clone)]
pub struct HeapProfilerLock {
    consumed: Arc<AtomicBool>,
}

pub struct HeapProfiler {
    #[cfg(feature = "profiler")]
    inner: std::sync::Mutex<Option<dhat::Profiler>>,
}

impl HeapProfiler {
    const fn new() -> Self {
        Self {
            #[cfg(feature = "profiler")]
            inner: std::sync::Mutex::new(None),
        }
    }

    pub fn install() -> HeapProfilerLock {
        #[cfg(feature = "profiler")]
        PROFILER
            .inner
            .lock()
            .expect("Cannot install heap profiler")
            .replace(dhat::Profiler::new_heap());
        HeapProfilerLock { consumed: Arc::new(AtomicBool::new(false)) }
    }
}

impl HeapProfilerLock {
    pub fn consume(&mut self) {
        self.consumed.store(true, Ordering::Relaxed);
        #[cfg(feature = "profiler")]
        {
            use tracing::error;

            let Ok(mut profiler) = PROFILER.inner.lock() else {
                error!("Cannot lock mutex");
                return;
            };
            let profiler = profiler.take();
            drop(profiler)
        }
    }

    pub fn consume_on_ctrlc(&mut self) {
        let mut this = self.clone();
        #[cfg(feature = "profiler")]
        {
            ctrlc::set_handler(move || {
                this.consume();
                std::process::exit(0);
            })
            .unwrap()
        }
    }
}

impl Drop for HeapProfilerLock {
    fn drop(&mut self) {
        if !self.consumed.load(Ordering::Relaxed) {
            self.consume();
        }
    }
}
