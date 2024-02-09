use derive_more::From;

#[derive(Default, From, Debug)]
pub enum Skip<E> {
    #[default]
    #[from(ignore)]
    Skipped,
    Error(E),
}

pub struct ByteSize;

impl ByteSize {
    const QUANTITIES: [&'static str; 5] = ["B", "KB", "MB", "GB", "TB"];

    const fn compress_step(value: usize, index: usize) -> (usize, &'static str) {
        if value < 1024 || index + 1 >= Self::QUANTITIES.len() {
            (value, Self::QUANTITIES[index])
        } else {
            Self::compress_step(value / 1024, index + 1)
        }
    }

    pub const fn compress(value: usize) -> (usize, &'static str) {
        Self::compress_step(value, 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::ByteSize;
    use rstest::rstest;

    #[rstest]
    #[case(1, (1, "B"))]
    #[case(1024, (1, "KB"))]
    #[case(1024 * 1024 * 10, (10, "MB"))]
    #[case(1024 * 1024 * 1024 * 10, (10, "GB"))]
    #[case(1024 * 1024 * 1024 * 1024 * 1024, (1024, "TB"))]
    fn test_human_readable_bytes(#[case] input: usize, #[case] expected: (usize, &'static str)) {
        assert_eq!(ByteSize::compress(input), expected)
    }
}
