use bevy_ecs::prelude::Resource;
use bevy_ecs::ptr::Ptr;
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{LazyLock, RwLock};

type DynDebugFn = fn(Ptr, &mut Formatter<'_>) -> std::fmt::Result;

#[derive(Debug, Resource, Clone)]
pub struct DebugRegistry {
    mapping: HashMap<TypeId, DynDebugFn>,
}

#[derive(Copy, Clone)]
pub struct DynDebug<'a> {
    value: Ptr<'a>,
    debug_fn: Option<DynDebugFn>,
}

static GLOBAL_REGISTRY: LazyLock<RwLock<DebugRegistry>> =
    LazyLock::new(|| RwLock::new(DebugRegistry::new()));

impl<'a> Debug for DynDebug<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(debug_fn) = self.debug_fn {
            debug_fn(self.value, f)
        } else {
            self.value.as_ptr().fmt(f)
        }
    }
}

impl<'a> DynDebug<'a> {
    pub fn is_known(&self) -> bool {
        self.debug_fn.is_some()
    }

    pub fn into_inner(self) -> Ptr<'a> {
        self.value
    }
}

impl DebugRegistry {
    pub fn empty() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    pub fn with_global(f: impl FnOnce(&mut DebugRegistry)) {
        let mut lock = GLOBAL_REGISTRY.write().unwrap_or_else(|e| e.into_inner());
        f(&mut lock);
    }

    pub fn new() -> Self {
        let mut this = Self::empty();
        this.register::<()>();
        this.register::<bool>();
        this.register::<char>();
        this.register::<u8>();
        this.register::<u16>();
        this.register::<u32>();
        this.register::<u64>();
        this.register::<u128>();
        this.register::<usize>();
        this.register::<i8>();
        this.register::<i16>();
        this.register::<i32>();
        this.register::<i64>();
        this.register::<i128>();
        this.register::<isize>();
        this.register::<f32>();
        this.register::<f64>();
        this.register::<String>();
        this.register::<Cow<'static, str>>();
        this
    }

    pub fn register_global<T: Any + Debug>() {
        let mut lock = GLOBAL_REGISTRY.write().unwrap_or_else(|e| e.into_inner());
        lock.register::<T>();
    }

    pub fn register<T: Any + Debug>(&mut self) {
        #[allow(unsafe_code)]
        self.mapping.insert(TypeId::of::<T>(), |this, fmt| {
            let this = unsafe { this.deref() };
            T::fmt(this, fmt)
        });
    }

    #[allow(unsafe_code)]
    pub unsafe fn debug<'a, T: Any + Debug>(&self, value: Ptr<'a>) -> DynDebug<'a> {
        DynDebug {
            value,
            debug_fn: self.mapping.get(&TypeId::of::<T>()).cloned(),
        }
    }

    #[allow(unsafe_code)]
    pub unsafe fn debug_global<T: Any + Debug>(value: Ptr) -> DynDebug {
        let lock = GLOBAL_REGISTRY.read().unwrap_or_else(|e| e.into_inner());
        unsafe { lock.debug::<T>(value) }
    }

    #[allow(unsafe_code)]
    pub unsafe fn debug_dynamic<'a>(&self, value: Ptr<'a>, type_id: TypeId) -> DynDebug<'a> {
        DynDebug {
            value,
            debug_fn: self.mapping.get(&type_id).cloned(),
        }
    }

    #[allow(unsafe_code)]
    pub unsafe fn debug_dynamic_global(value: Ptr, type_id: TypeId) -> DynDebug {
        let lock = GLOBAL_REGISTRY.read().unwrap_or_else(|e| e.into_inner());
        unsafe { lock.debug_dynamic(value, type_id) }
    }
}
