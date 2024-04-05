use extend::ext;

#[ext]
pub impl<T, E> Result<T, E> {
    fn recover<E2>(self, f: impl Fn(E) -> Result<T, E2>) -> Result<T, E2> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => match f(e) {
                Ok(x) => Ok(x),
                Err(e) => Err(e),
            },
        }
    }
}
