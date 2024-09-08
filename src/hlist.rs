pub struct HNil;
pub struct HCons<Head, Tail> {
    pub head: Head,
    pub tail: Tail,
}

pub trait HList {
    const LEN: usize;
}

pub trait FromHList<H: HList> {
    fn from_hlist(value: H) -> Self;
}

pub trait IntoHList<H: HList> {
    fn into_hlist(self) -> H;
}

pub trait Homogenous<U>: HList {
    fn into_vec(self) -> Vec<U>;
}

impl HList for HNil {
    const LEN: usize = 0;
}

impl<Head, Tail> HList for HCons<Head, Tail>
where
    Tail: HList,
{
    const LEN: usize = Tail::LEN + 1;
}

pub(crate) mod macros {
    macro_rules! hlist {
        () => { $crate::hlist::HNil };
        (...$rest:expr) => { $rest };
        ($a:expr) => { $crate::hlist::macros::hlist![$a,] };
        ($a:expr, $($tok:tt)*) => {
            $crate::hlist::HCons {
                head: $a,
                tail: $crate::hlist::macros::hlist![$($tok)*],
            }
        };
    }

    macro_rules! hlist_pat {
        () => { $crate::hlist::HNil };
        (...) => { _ };
        (...$rest:pat) => { $rest };
        ($a:pat) => { $crate::hlist::macros::hlist_pat![$a,] };
        ($a:pat, $($tok:tt)*) => {
            $crate::hlist::HCons {
                head: $a,
                tail: $crate::hlist::macros::hlist_pat![$($tok)*],
            }
        };
    }

    macro_rules! HList {
        () => {
            $crate::hlist::HNil
        };
        ($head:ty $(,)?) => {
            $crate::hlist::HCons<$head, $crate::hlist::macros::HList!()>
        };
        ($head:ty, $($tail:ty),* $(,)?) => {
            $crate::hlist::HCons<$head, $crate::hlist::macros::HList!($($tail),*)>
        };
    }

    macro_rules! impl_tuple {
        ($($t:ident$(,)?)*) => {
            impl<$($t,)*> $crate::hlist::FromHList<$crate::hlist::macros::HList!($($t,)*)> for ($($t,)*) {
                #[allow(clippy::unused_unit, non_snake_case)]
                fn from_hlist(value: $crate::hlist::macros::HList!($($t,)*)) -> Self {
                    let $crate::hlist::macros::hlist_pat! [ $($t,)* ] = value;
                    ($($t,)*)
                }
            }

            impl<$($t,)*> $crate::hlist::IntoHList<$crate::hlist::macros::HList!($($t,)*)> for ($($t,)*) {
                #[allow(non_snake_case)]
                fn into_hlist(self) -> $crate::hlist::macros::HList!($($t,)*) {
                    let ($($t,)*) = self;
                    $crate::hlist::macros::hlist!($($t,)*)
                }
            }
        };
        ($({$($t:ident$(,)?)*})*) => { $(impl_tuple!($($t,)*);)* };
    }

    impl_tuple! [
        {}
        {A}
        {A, B}
        {A, B, C}
        {A, B, C, D}
        {A, B, C, D, E}
        {A, B, C, D, E, F}
        {A, B, C, D, E, F, G}
        {A, B, C, D, E, F, G, H}
        {A, B, C, D, E, F, G, H, I}
        {A, B, C, D, E, F, G, H, I, J}
        {A, B, C, D, E, F, G, H, I, J, K}
        {A, B, C, D, E, F, G, H, I, J, K, L}
    ];

    pub(crate) use {hlist, hlist_pat, HList};
}

impl<U, T: Homogenous<U>> Homogenous<U> for HCons<U, T> {
    fn into_vec(self) -> Vec<U> {
        let mut base = self.tail.into_vec();
        base.insert(0, self.head);
        base
    }
}

impl<T> Homogenous<T> for HNil {
    fn into_vec(self) -> Vec<T> {
        vec![]
    }
}
