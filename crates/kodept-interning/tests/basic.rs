use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_interning::metrics::InterningMetrics;
use kodept_interning::InterningCodeHolder;
use std::ops::{Deref, Range};

#[derive(Copy, Clone)]
struct SampleCodeHolder;

impl CodeHolder for SampleCodeHolder {
    type Str = &'static str;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        const FIRST_PATTERN: Range<usize> = 0..5;
        const SECOND_PATTERN: Range<usize> = 5..10;

        match at.as_range() {
            FIRST_PATTERN => "12345",
            SECOND_PATTERN => "67890",
            _ => "TEST",
        }
    }
}

static OBJECT: InterningCodeHolder<SampleCodeHolder> = InterningCodeHolder::new(SampleCodeHolder);

#[test]
fn test() {
    let first = OBJECT.get_chunk(CodePoint::new(5, 0));
    assert_eq!(first.deref(), "12345");

    let second = OBJECT.get_chunk(CodePoint::new(5, 5));
    assert_eq!(second.deref(), "67890");

    let foo = second.clone();

    let metrics = InterningMetrics::gather();
    dbg!(&metrics, foo);
    assert!(matches!(
        metrics,
        InterningMetrics {
            total_shares: 3,
            total_items: 2,
            ..
        }
    ));
}
