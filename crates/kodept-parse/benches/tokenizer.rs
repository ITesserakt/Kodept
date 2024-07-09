use criterion::{Criterion, criterion_main};
use lazy_static::lazy_static;

use kodept_parse::token_match::TokenMatch;

const FILENAME: &str = "benches/benchmarking_file.kd";

lazy_static! {
    static ref FILE_CONTENTS: &'static str = std::fs::read_to_string(FILENAME).unwrap().leak();
    static ref TOKENS: &'static [TokenMatch<'static>] = {
        let tokenizer = kodept_parse::tokenizer::Tokenizer::new(*FILE_CONTENTS);
        tokenizer.into_vec().leak()
    };
}

mod default {
    use criterion::{BenchmarkGroup, black_box};
    use criterion::measurement::WallTime;

    use kodept_parse::tokenizer::SimpleTokenizer as Tokenizer;

    use crate::{FILE_CONTENTS};

    pub fn bench_impl(c: &mut BenchmarkGroup<WallTime>) {
        let input = *FILE_CONTENTS;
        c.bench_function("default implementation", |b| {
            b.iter(|| {
                let tokenizer = Tokenizer::new(black_box(input));
                let _ = black_box(tokenizer.into_vec());
            })
        });
    }
}

mod pest {
    use criterion::{BenchmarkGroup, black_box};
    use criterion::measurement::WallTime;

    use kodept_parse::grammar::PestKodeptParser as Tokenizer;

    use crate::FILE_CONTENTS;

    pub fn bench_impl(c: &mut BenchmarkGroup<WallTime>) {
        let input = *FILE_CONTENTS;
        c.bench_function("pest implementation", |b| {
            b.iter(|| {
                let tokenizer = Tokenizer::new(black_box(input));
                let _ = black_box(tokenizer.into_vec());
                // assert_eq!(tokenizer.into_vec(), output)
            })
        });
    }
}

mod peg {
    use criterion::{BenchmarkGroup, black_box};
    use criterion::measurement::WallTime;

    use kodept_parse::grammar::KodeptParser as Tokenizer;

    use crate::{FILE_CONTENTS};

    pub fn bench_impl(c: &mut BenchmarkGroup<WallTime>) {
        let input = *FILE_CONTENTS;
        c.bench_function("peg implementation", |b| {
            b.iter(|| {
                let tokenizer = Tokenizer::new(black_box(input));
                let _ = black_box(tokenizer.into_vec());
            })
        });
    }
}

pub fn benches() {
    let mut criterion = Criterion::default().configure_from_args();
    let mut group = criterion.benchmark_group("tokenizer");
    
    default::bench_impl(&mut group);
    pest::bench_impl(&mut group);
    peg::bench_impl(&mut group);
 
    group.finish();
}

criterion_main!(benches);
