pub enum HeapProfilerGuard {
    Empty,
    #[cfg(feature = "profiler")]
    Wrapper(implementation::HeapProfiler)
}

impl HeapProfilerGuard {
    pub fn install() -> Self {
        #[cfg(feature = "profiler")]
        return Self::Wrapper(implementation::HeapProfiler::new());
        #[cfg(not(feature = "profiler"))]
        return Self::Empty;
    }
}

#[cfg(feature = "profiler")]
mod implementation {
    use dhat::*;
    use kodept_frontend::external::Resource;

    #[global_allocator]
    static ALLOC: Alloc = Alloc;

    #[derive(Resource)]
    pub struct HeapProfiler {
        _inner: Profiler,
    }

    impl HeapProfiler {
        pub fn new() -> Self {
            Self {
                _inner: Profiler::new_heap(),
            }
        }
    }
}
