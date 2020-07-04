use criterion::{criterion_group, criterion_main, Criterion, black_box};
use core::time::Duration;
use net_ensembles::{*, rand::SeedableRng};
use rand_pcg::Pcg64;


pub fn bench_depth_first_sw(crit: &mut Criterion){
    let mut rng = Pcg64::seed_from_u64(12335);
    let n_vec = vec![10, 100, 1000, 10000];
    for n in n_vec
    {
        let name = format!("check_bfs_sw_n{}", n);
        let sw = SwEnsemble::<EmptyNode, _>::new(
            black_box(n),
            0.1,
            Pcg64::from_rng(&mut rng).unwrap()
        );
        crit.bench_function(&name, |b| {
            //er.randomize();
            b.iter(||
                {
                    sw.graph().bfs_index_depth(0).count();
                });
            }
        );
    }
}



criterion_group!{
    name = bench_depth_first;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::new(30, 0));
    targets = bench_depth_first_sw
}

criterion_main!(bench_depth_first);