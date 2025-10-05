use std::hash::BuildHasher;
use foldhash::fast::{FixedState, FoldHasher};

// Some random bytes taken from random.org :)
const FIXED_STATE: FixedState = FixedState::with_seed(0xdacb82e18c642297);

#[derive(Default)]
pub(crate) struct FixedHasher;

impl BuildHasher for FixedHasher {
    type Hasher = FoldHasher<'static>;

    fn build_hasher(&self) -> Self::Hasher {
        FIXED_STATE.build_hasher()
    }
}
