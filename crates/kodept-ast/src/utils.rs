use derive_more::From;

#[derive(Default, From, Debug)]
pub enum Skip<E> {
    #[default]
    #[from(ignore)]
    Skipped,
    Error(E),
}
