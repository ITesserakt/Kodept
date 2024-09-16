use derive_more::From;

#[derive(Default, Debug, From)]
pub enum Skip<E> {
    Failed(E),
    #[default]
    #[from(ignore)]
    Skipped,
}

pub struct ByteSize;

impl ByteSize {
    const QUANTITIES: [&'static str; 5] = ["B", "KB", "MB", "GB", "TB"];

    const fn compress_step(value: u64, index: usize) -> (u64, &'static str) {
        if value < 1024 || index + 1 >= Self::QUANTITIES.len() {
            (value, Self::QUANTITIES[index])
        } else {
            Self::compress_step(value / 1024, index + 1)
        }
    }

    pub const fn compress(value: u64) -> (u64, &'static str) {
        Self::compress_step(value, 0)
    }
    
    fn compress_float_step(value: f64, index: usize) -> (f64, &'static str) {
        if value < 1024.0 || index + 1 >= Self::QUANTITIES.len() {
            (value, Self::QUANTITIES[index])
        } else {
            Self::compress_float_step(value / 1024.0, index + 1)
        }
    }
    
    pub fn compress_float(value: f64) -> (f64, &'static str) {
        Self::compress_float_step(value, 0)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::utils::ByteSize;

    #[rstest]
    #[case(1, (1, "B"))]
    #[case(1024, (1, "KB"))]
    #[case(1024 * 1024 * 10, (10, "MB"))]
    #[case(1024 * 1024 * 1024 * 10, (10, "GB"))]
    #[case(1024 * 1024 * 1024 * 1024 * 1024, (1024, "TB"))]
    fn test_human_readable_bytes(#[case] input: u64, #[case] expected: (u64, &'static str)) {
        assert_eq!(ByteSize::compress(input), expected)
    }
}
