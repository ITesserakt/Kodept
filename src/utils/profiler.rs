use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::Plugin;

pub struct HeapProfilerPlugin;

impl Plugin for HeapProfilerPlugin {
    #[cfg(feature = "profiler")]
    fn build(self, world: &mut Frontend) {
        // Profiler will drop eventually, so there is no need for explicit exit observer
        world.insert_resource(implementation::HeapProfiler::new());
    }

    #[cfg(not(feature = "profiler"))]
    fn build(self, _: &mut Frontend) {}
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
