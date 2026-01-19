use crate::substitution::Substitutions;
use crate::traits::{FreeTypeVars, Substitutable};
use crate::utils::JoinedDisplay;
use derive_more::From;
use kodept_interning::{GlobalInterner, InternInto, Interned, Interner};
use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{BuildHasher, Hash, RandomState};
use std::num::NonZeroU8;
use std::ops::BitAnd;

impl<'a> InternInto<MonomorphicType> for &'a MonomorphicType {
    fn intern_into(self) -> Interned<MonomorphicType> {
        self.intern()
    }
}

#[allow(dead_code)]
fn expand_to_string(id: usize, alphabet: &'static str) -> String {
    if id == 0 {
        return alphabet
            .chars()
            .next()
            .expect("Alphabet should contain at least one letter")
            .to_string();
    }

    let alphabet: Vec<_> = alphabet.chars().collect();
    let mut current = id;
    let mut result = String::new();
    while current > 0 {
        result.push(alphabet[current % alphabet.len()]);
        current /= alphabet.len();
    }
    result
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    /// Singleton type
    Unit,
    /// Type with two possibilities: true and false
    Boolean,
    // To keep future compatibility with hvm, builtin types should have 24 bytes size at max
    /// Parameter represents the number of bytes for number
    I(NonZeroU8),
    /// Parameter represents the number of bytes for number
    U(NonZeroU8),
    F24,
    /// String with some value that is known at compile time
    StaticString(u64),
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct TVar {
    #[cfg(not(test))]
    index: usize,
    #[cfg(test)]
    pub index: usize,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct TConstant(usize);

#[derive(PartialEq, Eq, Hash, Clone, From)]
pub enum MonomorphicType {
    #[from(ignore)]
    Primitive(Interned<PrimitiveType>),
    Var(TVar),
    #[from(ignore)]
    Fn(Interned<MonomorphicType>, Interned<MonomorphicType>),
    #[from(ignore)]
    Tuple(Box<[Interned<MonomorphicType>]>),
    #[from(ignore)]
    Pointer(Interned<MonomorphicType>),
    #[from(ignore)]
    /// Array of some type which size is known at compile time
    StaticArray(u64, Interned<MonomorphicType>),
    Constant(TConstant),
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct PolymorphicType {
    pub bindings: Box<[TVar]>,
    pub binding_type: Interned<MonomorphicType>,
}

mod interning {
    use super::*;
    use kodept_interning::{GlobalInterner, Internable};
    use std::hash::Hasher;

    fn boxy_leak<T: Clone>(value: &T) -> &'static T {
        Box::leak(Box::new(value.clone()))
    }

    static GLOBAL_PRIMITIVE_TYPES_POOL: Interner<PrimitiveType> = Interner::new();
    static GLOBAL_MONOMORPHIC_TYPES_POOL: Interner<MonomorphicType> = Interner::new();

    struct PrimitiveTypes {
        signed: [PrimitiveType; u8::MAX as usize - 1],
        unsigned: [PrimitiveType; u8::MAX as usize - 1],
    }

    static PRIMITIVES: PrimitiveTypes = {
        let mut signed = [PrimitiveType::Boolean; u8::MAX as usize - 1];
        let mut unsigned = [PrimitiveType::Boolean; u8::MAX as usize - 1];
        let mut i = NonZeroU8::MIN;
        while i.get() < u8::MAX {
            signed[i.get() as usize - 1] = PrimitiveType::I(i);
            i = i.saturating_add(1)
        }
        i = NonZeroU8::MIN;
        while i.get() < u8::MAX {
            unsigned[i.get() as usize - 1] = PrimitiveType::U(i);
            i = i.saturating_add(1)
        }

        PrimitiveTypes { signed, unsigned }
    };

    impl Internable for PrimitiveType {
        fn leak(&self) -> &'static Self {
            match self {
                PrimitiveType::Unit => &PrimitiveType::Unit,
                PrimitiveType::Boolean => &PrimitiveType::Boolean,
                PrimitiveType::I(n) => &PRIMITIVES.signed[n.get() as usize - 1],
                PrimitiveType::U(n) => &PRIMITIVES.unsigned[n.get() as usize - 1],
                PrimitiveType::F24 => &PrimitiveType::F24,
                PrimitiveType::StaticString(_) => boxy_leak(self),
            }
        }

        fn ref_eq(&self, other: &Self) -> bool {
            std::ptr::eq(self, other)
        }

        fn ref_hash<H: Hasher>(&self, state: &mut H) {
            std::ptr::hash(self, state)
        }
    }

    impl GlobalInterner for PrimitiveType {
        #[inline(always)]
        fn interner() -> &'static Interner<Self> {
            &GLOBAL_PRIMITIVE_TYPES_POOL
        }
    }

    impl Internable for MonomorphicType {
        fn leak(&self) -> &'static Self {
            boxy_leak(self)
        }

        fn leak_owned(self) -> &'static Self
        where
            Self: Sized,
        {
            Box::leak(Box::new(self))
        }

        fn ref_eq(&self, other: &Self) -> bool {
            std::ptr::eq(self, other)
        }

        fn ref_hash<H: Hasher>(&self, state: &mut H) {
            std::ptr::hash(self, state)
        }
    }

    impl GlobalInterner for MonomorphicType {
        #[inline(always)]
        fn interner() -> &'static Interner<Self> {
            &GLOBAL_MONOMORPHIC_TYPES_POOL
        }
    }
}

mod ctors {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    impl PrimitiveType {
        pub const fn bool() -> Self {
            Self::Boolean
        }

        pub const fn i8() -> Self {
            Self::I(NonZeroU8::new(8).unwrap())
        }

        pub const fn u8() -> Self {
            Self::U(NonZeroU8::new(8).unwrap())
        }

        pub const fn custom(sign: bool, bits: NonZeroU8) -> Self {
            match sign {
                true => Self::I(bits),
                false => Self::U(bits),
            }
        }

        pub const fn f24() -> Self {
            Self::F24
        }

        pub const fn string(size: u64) -> Self {
            Self::StaticString(size)
        }
    }

    impl MonomorphicType {
        pub fn fun1(
            input: impl InternInto<MonomorphicType>,
            output: impl InternInto<MonomorphicType>,
        ) -> MonomorphicType {
            MonomorphicType::Fn(input.intern_into(), output.intern_into())
        }

        pub fn fun<T: InternInto<MonomorphicType>>(
            head: impl InternInto<MonomorphicType>,
            tail: impl IntoIterator<Item = T, IntoIter: DoubleEndedIterator<Item = T>>,
            output: MonomorphicType,
        ) -> MonomorphicType {
            std::iter::once(head.intern_into())
                .chain(tail.into_iter().map(|it| it.intern_into()))
                .rfold(output, |acc, next| {
                    MonomorphicType::Fn(next.intern(), acc.intern_owned())
                })
        }

        pub fn tuple(
            items: impl IntoIterator<Item: InternInto<MonomorphicType>>,
        ) -> MonomorphicType {
            MonomorphicType::Tuple(items.into_iter().map(|it| it.intern_into()).collect())
        }

        pub fn var() -> Self {
            MonomorphicType::Var(TVar::new())
        }

        pub fn constant() -> Self {
            MonomorphicType::Constant(TConstant::new())
        }

        pub fn primitive(value: PrimitiveType) -> Self {
            Self::Primitive(value.intern_owned())
        }

        pub const UNIT: Self = Self::Primitive(Interned(&PrimitiveType::Unit));

        pub fn array(count: u64, inner_type: impl InternInto<MonomorphicType>) -> Self {
            Self::StaticArray(count, inner_type.intern_into())
        }

        pub fn pointer(inner_type: impl InternInto<MonomorphicType>) -> Self {
            Self::Pointer(inner_type.intern_into())
        }
    }

    impl From<PrimitiveType> for MonomorphicType {
        fn from(value: PrimitiveType) -> Self {
            Self::Primitive(value.intern_owned())
        }
    }

    static GENERATOR: AtomicUsize = AtomicUsize::new(0);
    impl TVar {
        #[inline]
        pub fn new() -> TVar {
            let id = GENERATOR.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            TVar { index: id }
        }

        #[inline]
        pub fn new_many<const N: usize>() -> [TVar; N] {
            [0; N].map(|_| TVar::new())
        }
    }
    impl TConstant {
        #[inline]
        pub fn new() -> TConstant {
            let id = GENERATOR.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            TConstant(id)
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Unit => write!(f, "()"),
            PrimitiveType::Boolean => write!(f, "boolean"),
            PrimitiveType::I(size) => write!(f, "i{size}"),
            PrimitiveType::U(size) => write!(f, "u{size}"),
            PrimitiveType::F24 => write!(f, "f24"),
            PrimitiveType::StaticString(size) => write!(f, "s{size}"),
        }
    }
}

impl Debug for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl MonomorphicType {
    fn rename(&mut self, old: TVar, new: TVar) {
        match self {
            Self::Var(id) if id == &old => *id = new,
            Self::Var(_) => {}
            Self::Fn(input, output) => {
                let mut new_input = input.0.clone();
                let mut new_output = output.0.clone();
                new_input.rename(old, new);
                new_output.rename(old, new);
                *input = new_input.intern_owned();
                *output = new_output.intern_owned();
            }
            Self::Pointer(x) => {
                let mut new_ptr = x.0.clone();
                new_ptr.rename(old, new);
                *x = new_ptr.intern_owned();
            }
            Self::Tuple(vec) if !vec.is_empty() => {
                for item in vec {
                    let mut new_item = item.0.clone();
                    new_item.rename(old, new);
                    *item = new_item.intern_owned();
                }
            }
            Self::Tuple(_) => {}
            Self::StaticArray(_, x) => {
                let mut new_x = x.0.clone();
                new_x.rename(old, new);
                *x = new_x.intern_owned();
            }
            Self::Constant(_) => {}
            Self::Primitive(_) => {}
        }
    }

    fn extract_vars<E>(&self, buf: &mut E)
    where
        E: Extend<TVar>,
    {
        let mut stack: Vec<&MonomorphicType> = vec![self];

        while let Some(current) = stack.pop() {
            match current {
                Self::Primitive(_) => {}
                Self::Var(x) => buf.extend(Some(*x)),
                Self::Fn(input, output) => {
                    stack.push(input);
                    stack.push(output);
                }
                Self::Tuple(vec) => stack.extend(vec.iter().map(|it| it.0)),
                Self::Pointer(x) => stack.push(x),
                Self::Constant(_) => {}
                Self::StaticArray(_, inner) => stack.push(inner),
            }
        }
    }

    pub fn generalize(&self, free: &HashSet<TVar>) -> PolymorphicType {
        let diff: Box<_> = self.free_types().difference(free).copied().collect();
        PolymorphicType {
            bindings: diff,
            binding_type: self.intern(),
        }
    }
}

impl PolymorphicType {
    pub fn instantiate(&self) -> Interned<MonomorphicType> {
        let subst = self
            .bindings
            .iter()
            .map(|it| (*it, MonomorphicType::var()))
            .collect();
        self.binding_type & &subst
    }
}

impl BitAnd<Substitutions> for PolymorphicType {
    type Output = PolymorphicType;

    fn bitand(self, rhs: Substitutions) -> Self::Output {
        self.substitute(&rhs)
    }
}

impl BitAnd<&Substitutions> for PolymorphicType {
    type Output = PolymorphicType;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl BitAnd<Substitutions> for &PolymorphicType {
    type Output = PolymorphicType;

    fn bitand(self, rhs: Substitutions) -> Self::Output {
        self.substitute(&rhs)
    }
}

impl BitAnd<&Substitutions> for &PolymorphicType {
    type Output = PolymorphicType;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl BitAnd<Substitutions> for MonomorphicType {
    type Output = Interned<MonomorphicType>;

    fn bitand(self, rhs: Substitutions) -> Self::Output {
        self.substitute(&rhs)
    }
}

impl BitAnd<&Substitutions> for MonomorphicType {
    type Output = Interned<MonomorphicType>;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl BitAnd<Substitutions> for &MonomorphicType {
    type Output = Interned<MonomorphicType>;

    fn bitand(self, rhs: Substitutions) -> Self::Output {
        self.substitute(&rhs)
    }
}

impl BitAnd<&Substitutions> for &MonomorphicType {
    type Output = Interned<MonomorphicType>;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl BitAnd<&Substitutions> for Interned<MonomorphicType> {
    type Output = Self;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl Display for TVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "τ{}", self.index)
    }
}

impl Display for TConstant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "T{}", self.0)
    }
}

impl Display for MonomorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(p) => write!(f, "{p}"),
            Self::Var(v) => write!(f, "{v}"),
            Self::Fn(input, output) => match input.0 {
                Self::Fn(_, _) => write!(f, "({input}) -> {output}"),
                _ => write!(f, "{input} -> {output}"),
            },
            Self::Tuple(vec) => write!(f, "({})", JoinedDisplay::enumerate(vec)),
            Self::Pointer(t) => write!(f, "*{t}"),
            Self::Constant(id) => write!(f, "{id}"),
            Self::StaticArray(count, ty) => write!(f, "{{{ty}}}[{count}]"),
        }
    }
}

struct Bound<T>(T);

impl Display for Bound<TVar> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            expand_to_string(self.0.index, crate::LOWER_ALPHABET)
        )
    }
}

impl Display for Bound<&MonomorphicType> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            MonomorphicType::Var(x) => write!(f, "{}", Bound(*x)),
            MonomorphicType::Fn(input, output) => match input.0 {
                MonomorphicType::Fn(_, _) => {
                    write!(f, "({}) -> {}", Bound(input.0), Bound(output.0))
                }
                _ => write!(f, "{} -> {}", Bound(input.0), Bound(output.0)),
            },
            MonomorphicType::Tuple(vec) => write!(
                f,
                "({})",
                JoinedDisplay::enumerate(vec.iter().map(|it| Bound(it.0))).join()
            ),
            MonomorphicType::Pointer(t) => write!(f, "*{}", Bound(t.0)),
            MonomorphicType::StaticArray(n, t) => write!(f, "{{{}}}[{}]", Bound(t.0), n),
            MonomorphicType::Constant(_) | MonomorphicType::Primitive(_) => write!(f, "{}", self.0),
        }
    }
}

impl Display for PolymorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        struct LinkedHashSet<T, S = RandomState>(LinkedHashMap<T, (), S>);

        impl<T, S> Extend<T> for LinkedHashSet<T, S>
        where
            T: Hash + Eq,
            S: BuildHasher,
        {
            fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
                self.0.extend(iter.into_iter().map(|it| (it, ())))
            }
        }

        impl<T, S> IntoIterator for LinkedHashSet<T, S>
        where
            T: Hash + Eq,
            S: BuildHasher,
        {
            type Item = T;
            type IntoIter = std::iter::Map<linked_hash_map::IntoIter<T, ()>, fn((T, ())) -> T>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter().map(|it| it.0)
            }
        }

        if self.bindings.is_empty() {
            return write!(f, "{}", self.binding_type);
        }
        // quickly normalize type
        let mut binding_type = self.binding_type.0.clone();
        let mut set = LinkedHashSet(LinkedHashMap::new());
        binding_type.extract_vars(&mut set);
        let vars_count = set.0.len();
        set.into_iter()
            .rev()
            .zip(0usize..)
            .for_each(|(old, new)| binding_type.rename(old, TVar { index: new }));

        // replace all TVars with it's bound counterpart
        write!(
            f,
            "∀{} => {}",
            JoinedDisplay::enumerate((0..vars_count).map(|it| Bound(TVar { index: it }))).join(),
            Bound(&binding_type)
        )
    }
}

impl Debug for TVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for TConstant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for MonomorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for PolymorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(test)]
pub mod tests {
    use crate::r#type::{MonomorphicType, TVar};

    //     use crate::r#type::{MonomorphicType, PolymorphicType, PrimitiveType, TVar, fun1, var};
    //     use proptest::arbitrary::StrategyFor;
    //     use proptest::prelude::{Arbitrary, BoxedStrategy, Just, Strategy, any, prop};
    //     use proptest::prop_oneof;
    //     use proptest::strategy::{Map, Recursive};
    //     use std::num::NonZeroU8;
    //     use std::sync::Arc;
    //
    //     impl Arbitrary for PrimitiveType {
    //         type Parameters = ();
    //
    //         fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
    //             let leaf = prop_oneof![
    //                 Just(PrimitiveType::Boolean),
    //                 Just(PrimitiveType::F24),
    //                 (1..=24u8).prop_map(|it| PrimitiveType::I(NonZeroU8::new(it).unwrap())),
    //                 (1..=24u8).prop_map(|it| PrimitiveType::U(NonZeroU8::new(it).unwrap())),
    //                 any::<u64>().prop_map(PrimitiveType::StaticString)
    //             ];
    //
    //             leaf.prop_recursive(2, 3, 1, |inner| {
    //                 (inner, any::<u64>())
    //                     .prop_map(|it| PrimitiveType::StaticArray(it.1, Arc::new(it.0)))
    //                     .boxed()
    //             })
    //         }
    //
    //         type Strategy = Recursive<
    //             PrimitiveType,
    //             fn(BoxedStrategy<PrimitiveType>) -> BoxedStrategy<PrimitiveType>,
    //         >;
    //     }
    //
    //     impl Arbitrary for MonomorphicType {
    //         type Parameters = ();
    //
    //         fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
    //             let leaf = prop_oneof![
    //                 any::<PrimitiveType>().prop_map(MonomorphicType::Primitive),
    //                 any::<TVar>().prop_map(MonomorphicType::Var),
    //                 "_{0,2}[A-Z]([A-Za-z0-9_]){0,5}"
    //                     .prop_map(|it| MonomorphicType::Constant(it.into()))
    //             ];
    //
    //             leaf.prop_recursive(10, 100, 10, |inner| {
    //                 prop_oneof![
    //                     prop::collection::vec(inner.clone(), 0..5)
    //                         .prop_map(|it| MonomorphicType::Tuple(Arc::from(it))),
    //                     inner
    //                         .clone()
    //                         .prop_map(|it| MonomorphicType::Pointer(Arc::new(it))),
    //                     (inner.clone(), inner).prop_map(|it| fun1(it.0, it.1))
    //                 ]
    //                 .boxed()
    //             })
    //         }
    //
    //         type Strategy = Recursive<
    //             MonomorphicType,
    //             fn(BoxedStrategy<MonomorphicType>) -> BoxedStrategy<MonomorphicType>,
    //         >;
    //     }
    //
    //     impl Arbitrary for PolymorphicType {
    //         type Parameters = ();
    //
    //         fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
    //             any::<MonomorphicType>().prop_map(|it| it.normalize())
    //         }
    //
    //         type Strategy = Map<StrategyFor<MonomorphicType>, fn(MonomorphicType) -> PolymorphicType>;
    //     }
    //
    #[test]
    fn test_rename() {
        let [t1, t2, t3] = [1, 2, 3].map(|_| TVar::new());
        let mut ty = MonomorphicType::fun1(t1, t2);
        ty.rename(t1, t3);
        assert_eq!(ty, MonomorphicType::fun1(t3, t2));

        let mut ty = ty.clone();
        ty.rename(t3, t1);
        assert_eq!(ty, MonomorphicType::fun1(t1, t2))
    }
}
