use criterion::*;
use mempool_sync::oddsketch::Oddsketch;
use rand::Rng;

fn random_oddsketch() -> Oddsketch {
    let mut rng = rand::thread_rng();
    let mut oddsketch = Oddsketch::default();

    for _ in 0..1024 {
        oddsketch.insert(rng.gen());
    }
    oddsketch
}

fn bench_insert_single(c: &mut Criterion) {
    c.bench_function("oddsketch insert single", move |b| {
        b.iter_batched(
            // Setup
            || (random_oddsketch(), rand::thread_rng().gen()),
            // Loop
            |(mut oddsketch, i)| oddsketch.insert(black_box(i)),
            BatchSize::SmallInput,
        )
    });
}

fn bench_insert_million(c: &mut Criterion) {
    c.bench_function("oddsketch insert 1 million", move |b| {
        b.iter_batched(
            // Setup
            || {
                let inputs: Vec<u64> = (0..1_000_000).map(|_| rand::thread_rng().gen()).collect();
                (random_oddsketch(), inputs)
            },
            // Loop
            |(mut oddsketch, inputs)| {
                for i in inputs {
                    oddsketch.insert(black_box(i))
                }
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_decode(c: &mut Criterion) {
    c.bench_function("oddsketch decode", move |b| {
        b.iter_batched(
            // Setup
            || (random_oddsketch(), random_oddsketch()),
            // Function
            |(oddsketch_a, oddsketch_b)| (black_box(oddsketch_a) ^ black_box(oddsketch_b)).size(),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_insert_single,
    bench_decode,
    bench_insert_million
);
criterion_main!(benches);
