use net_ensembles::*;
use criterion::{criterion_group, criterion_main, Criterion, black_box};
use core::time::Duration;


pub fn bench_complete_graph(c: &mut Criterion, n: usize){
    let name = format!("complete_graph_{}", n);
    c.bench_function(&name, |b| {
        b.iter(||
            {
                Graph::<EmptyNode>::complete_graph(n);
            });
        }
    );
}

pub fn bench(c: &mut Criterion){
    bench_complete_graph(c, black_box(10));
    bench_complete_graph(c, black_box(100));
    bench_complete_graph(c, black_box(1000));
}

pub fn graph_creation(c: &mut Criterion, n: usize)
{
    let name = format!("graph_creation_{}", n);
    c.bench_function(&name, |b| {
        b.iter(||
            {
                Graph::<EmptyNode>::new(n);
            });
        }
    );
}

pub fn bench_creation(c: &mut Criterion){
    graph_creation(c, black_box(10));
    graph_creation(c, black_box(100));
    graph_creation(c, black_box(1000));
    graph_creation(c, black_box(100000));
}

criterion_group!{
    name = measure_complete_graph;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::new(30, 0));
    targets = bench_creation, bench
}

criterion_main!(measure_complete_graph);