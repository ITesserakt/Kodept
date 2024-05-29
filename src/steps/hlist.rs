pub struct HNil;
pub struct HCons<Head, Tail> {
    pub head: Head,
    pub tail: Tail,
}

pub trait HList {
    const LEN: usize;
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
    () => { $crate::steps::hlist::HNil };
    (...$rest:expr) => { $rest };
    ($a:expr) => { $crate::steps::hlist::macros::hlist![$a,] };
    ($a:expr, $($tok:tt)*) => {
        $crate::steps::hlist::HCons {
            head: $a,
            tail: $crate::steps::hlist::macros::hlist![$($tok)*],
        }
    };
}

    macro_rules! hlist_pat {
    () => { $crate::steps::hlist::HNil };
    (...) => { _ };
    (...$rest:pat) => { $rest };
    ($a:pat) => { $crate::steps::hlist::macros::hlist_pat![$a,] };
    ($a:pat, $($tok:tt)*) => {
        $crate::steps::hlist::HCons {
            head: $a,
            tail: $crate::steps::hlist::macros::hlist_pat![$($tok)*],
        }
    };
}
    
    pub(crate) use {hlist, hlist_pat};
}
