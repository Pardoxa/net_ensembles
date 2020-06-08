use criterion::{criterion_group, criterion_main, Criterion, black_box};
use core::time::Duration;
use net_ensembles::{*, rand::SeedableRng};
use rand_pcg::Pcg64;


pub fn bench_randomization(c: &mut Criterion, seed: u64, n: usize, conectivity: f64){
    let rng = Pcg64::seed_from_u64(seed);
    let mut er = ErEnsembleC::<EmptyNode, _>::new(
        n,
        conectivity,
        rng
    );
    let name = format!("randomize_er_c_{}", n);
    c.bench_function(&name, |b| {
        b.iter(||
            {
                er.randomize();
            });
        }
    );
}

pub fn bench(c: &mut Criterion){
    bench_randomization(c, black_box(10), black_box(10), black_box(2.0));
    bench_randomization(c, black_box(1230), black_box(100), black_box(2.0));
    bench_randomization(c, black_box(103450), black_box(1000), black_box(2.0));
}


criterion_group!{
    name = randomize_er_c;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::new(30, 0));
    targets = bench
}

criterion_main!(randomize_er_c);