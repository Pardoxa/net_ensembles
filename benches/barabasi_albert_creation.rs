use net_ensembles::*;
use criterion::{criterion_group, criterion_main, Criterion};
use core::time::Duration;
use rand_pcg::Pcg64;
use rand::SeedableRng;


pub fn bench_ba<T, Rng>(c: &mut Criterion, mut ba_ensemble: BAensemble<T,Rng>, name: &str)
where T: Node,
    Rng: rand::Rng
{
    c.bench_function(name, |b| {
        b.iter(||
            {
                ba_ensemble.randomize()
            });
        }
    );
}

pub fn bench(c: &mut Criterion){

    let rng = Pcg64::seed_from_u64(12);
    let ba: BAensemble<EmptyNode, _> =  BAensemble::new(100, rng, 1, 2);

    bench_ba(c, ba, "ba_randomize_n100m1C2");

    let rng = Pcg64::seed_from_u64(122321232);
    let mut er: ErEnsembleC<EmptyNode, _> = ErEnsembleC::new(10, 3.0, rng);
    // create valid graph
    while er.graph().container_iter().any(|container| container.degree() < 1) {
        er.randomize();
    }
    let rng = Pcg64::seed_from_u64(1878321232);
    let ba = BAensemble::new_from_graph(20, rng, 2, er.graph());

    bench_ba(c, ba, "ba_randomize_n20m2_er10c3");
    
    let rng= Pcg64::seed_from_u64(1878321232);
    let sw: SwEnsemble<EmptyNode, _> = SwEnsemble::new(10, 0.1, rng);
    let rng= Pcg64::seed_from_u64(78321232);
    let ba = BAensemble::new_from_generic_graph(50, rng, 2, sw);

    bench_ba(c, ba, "ba_randomize_n50m2_sw10p0.1");
}


criterion_group!{
    name = measure_complete_graph;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::new(30, 0));
    targets = bench
}

criterion_main!(measure_complete_graph);