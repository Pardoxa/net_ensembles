use net_ensembles::{*, rand::SeedableRng};
use criterion::{criterion_group, criterion_main, Criterion};
use rand_pcg::Pcg64;
use core::time::Duration;

const SIZE: u32 = 50;
const C: f64 = 4.0;
const SEED: u64 = 123_239_010;
const M: usize = 100;

pub fn bench_vertex_load<'a, T, A, E, F> (c: &mut Criterion, name: &str, mut init: F)
where
    T: Node,
    A: AdjContainer<T>,
    E: WithGraph<T, GenericGraph<T, A>> + SimpleSample,
    GenericGraph<T, A>: Clone,
    F: FnMut() -> E
{
    let name_vertex_load = format!("vertex_load_{}", name);
    let mut e = init();
    c.bench_function(&name_vertex_load, |b| {
            e.randomize();
            b.iter(||
                {
                    e.graph().vertex_load(true);
                });

        }

    );
}

pub fn er_c_bench(c: &mut Criterion) {
    bench_vertex_load(c, "er_c", || {
        let rng = Pcg64::seed_from_u64(SEED);
        ErEnsembleC::<EmptyNode, _>::new(
            SIZE,
            C,
            rng
        )
    });
}

pub fn er_m_bench(c: &mut Criterion) {
    bench_vertex_load(c, "er_m", || {
        let rng = Pcg64::seed_from_u64(SEED);
        ErEnsembleM::<EmptyNode, Pcg64>::new(SIZE, M, rng)
    });
}

pub fn sw_bench(c: &mut Criterion) {
    bench_vertex_load(c, "sw", || {
        let rng = Pcg64::seed_from_u64(SEED);
        SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng)
    });
}

criterion_group!{
    name = v_er_m;
    config = Criterion::default()
        .sample_size(200)
        .warm_up_time( Duration::new(1, 1))
        .measurement_time(Duration::new(30, 0));
    targets = er_c_bench, er_m_bench, sw_bench
}

criterion_main!(v_er_m);
