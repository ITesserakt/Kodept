use derive_more::{Display, Error};
use std::marker::PhantomData;
use std::rc::Rc;

use slotmap::sparse_secondary::Entry;
use slotmap::{SecondaryMap, SparseSecondaryMap};

use kodept_ast::graph::{GenericASTNode, GenericNodeKey};
use kodept_ast::traits::Identifiable;
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::Language;
use kodept_inference::r#type::PolymorphicType;

type Cache<V: StoreObject> = SecondaryMap<GenericNodeKey, V::Boxed>;

#[derive(Debug, Error, Display)]
#[display(fmt = "Unknown key passed to the cache")]
pub struct UnknownKey;

#[derive(Debug, Default)]
pub struct Store {
    models_cache: Cache<Model>,
    types_cache: Cache<Type>,
    constraints_cache: Cache<Constraint>,
}

pub struct StoreRef<'a, Variant: StoreObject> {
    inner: &'a Store,
    to_add: SparseSecondaryMap<GenericNodeKey, Variant::Boxed>,
    _phantom: PhantomData<Variant>,
}

pub struct Model;
pub struct Type;
pub struct Constraint;

trait StoreObject {
    type Object;
    type Boxed;
}

pub trait StoreOps<V: StoreObject> {
    fn get_or_init(
        &mut self,
        key: GenericNodeKey,
        f: impl FnOnce() -> V::Object,
    ) -> Result<V::Boxed, UnknownKey>;
    
    fn get<N>(&mut self, node: &N, f: impl FnOnce() -> V::Object) -> V::Boxed
    where
        N: Identifiable + Into<GenericASTNode>,
    {
        let id: GenericNodeKey = node.get_id().widen().into();
        self.get_or_init(id, f).unwrap()
    }

    fn save(self, store: &mut Store);

    fn cache(&self) -> &Cache<V>;
}

impl StoreObject for Model {
    type Object = Language;
    type Boxed = Rc<Language>;
}

impl StoreObject for Type {
    type Object = PolymorphicType;
    type Boxed = Rc<PolymorphicType>;
}

impl StoreObject for Constraint {
    type Object = Assumptions;
    type Boxed = Box<Assumptions>;
}

impl<'a, V: StoreObject> StoreRef<'a, V> {
    pub fn new(store: &'a Store) -> Self {
        Self {
            inner: store,
            to_add: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl StoreOps<Model> for StoreRef<'_, Model> {
    fn get_or_init(
        &mut self,
        key: GenericNodeKey,
        f: impl FnOnce() -> Language,
    ) -> Result<Rc<Language>, UnknownKey> {
        return match self.to_add.entry(key).ok_or(UnknownKey)? {
            Entry::Occupied(x) => Ok(x.get().clone()),
            Entry::Vacant(x) => match self.inner.models_cache.get(key) {
                None => Ok(x.insert(Rc::new(f())).clone()),
                Some(it) => Ok(it.clone()),
            },
        };
    }

    fn save(self, store: &mut Store) {
        store.models_cache.extend(self.to_add.into_iter())
    }

    fn cache(&self) -> &Cache<Model> {
        &self.inner.models_cache
    }
}

impl StoreOps<Type> for StoreRef<'_, Type> {
    fn get_or_init(
        &mut self,
        key: GenericNodeKey,
        f: impl FnOnce() -> PolymorphicType,
    ) -> Result<Rc<PolymorphicType>, UnknownKey> {
        return match self.to_add.entry(key).ok_or(UnknownKey)? {
            Entry::Occupied(x) => Ok(x.get().clone()),
            Entry::Vacant(x) => match self.inner.types_cache.get(key) {
                None => Ok(x.insert(Rc::new(f())).clone()),
                Some(it) => Ok(it.clone()),
            },
        };
    }

    fn save(self, store: &mut Store) {
        store.types_cache.extend(self.to_add.into_iter())
    }

    fn cache(&self) -> &Cache<Type> {
        &self.inner.types_cache
    }
}

impl StoreOps<Constraint> for StoreRef<'_, Constraint> {
    fn get_or_init(
        &mut self,
        key: GenericNodeKey,
        f: impl FnOnce() -> Assumptions,
    ) -> Result<Box<Assumptions>, UnknownKey> {
        return match self.to_add.entry(key).ok_or(UnknownKey)? {
            Entry::Occupied(x) => Ok(x.get().clone()),
            Entry::Vacant(x) => match self.inner.constraints_cache.get(key) {
                None => Ok(x.insert(Box::new(f())).clone()),
                Some(it) => Ok(it.clone()),
            },
        };
    }

    fn save(self, store: &mut Store) {
        store.constraints_cache.extend(self.to_add.into_iter())
    }

    fn cache(&self) -> &Cache<Constraint> {
        &self.inner.constraints_cache
    }
}

impl Store {
    pub fn models(&self) -> StoreRef<Model> {
        StoreRef::new(self)
    }

    pub fn types(&self) -> StoreRef<Type> {
        StoreRef::new(self)
    }

    pub fn constraints(&self) -> StoreRef<Constraint> {
        StoreRef::new(self)
    }
}
