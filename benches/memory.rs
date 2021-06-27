use {
    criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput},
    fixity::{
        path::{MapSegment, Path},
        Fixity,
    },
    tokio::runtime::Runtime,
};
const MUL_LEN: u32 = 10;
fn single_insertion_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_insertion_depth");
    for mul in 0..MUL_LEN {
        let depth = (mul * 5) + 1;
        let path = Path::from((0..depth).map(MapSegment::new).collect::<Vec<_>>());
        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, _depth| {
            let rt = Runtime::new().unwrap();
            b.to_async(rt).iter_with_large_drop(|| async {
                let f = Fixity::memory();
                {
                    let mut m = f.map(path.clone());
                    m.insert(0, 0).await.unwrap();
                }
                f
            });
        });
    }
    group.finish();
}
fn multi_insertion_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_insertion_depth");
    for mul in 0..MUL_LEN {
        let depth = (mul * 5) + 1;
        let path = Path::from((0..depth).map(MapSegment::new).collect::<Vec<_>>());
        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, _depth| {
            let rt = Runtime::new().unwrap();
            b.to_async(rt).iter_with_large_drop(|| async {
                let f = Fixity::memory();
                {
                    let mut m = f.map(path.clone());
                    for i in 0..5 {
                        m.insert(i, i).await.unwrap();
                    }
                }
                f
            });
        });
    }
    group.finish();
}

criterion_group!(benches, single_insertion_depth, multi_insertion_depth,);
criterion_main!(benches);
