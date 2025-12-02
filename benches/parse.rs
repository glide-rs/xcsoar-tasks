use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use xcsoar_tasks::from_str;

const AAT_TASK: &str = include_str!("../fixtures/aat-task.tsk");
const RACING_TASK: &str = include_str!("../fixtures/racing-task.tsk");
const FAI_TASK: &str = include_str!("../fixtures/fai-task.tsk");
const ALL_OZ_TYPES: &str = include_str!("../fixtures/all-oz-types.tsk");

fn bench_from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_str");

    group.bench_function("aat_task", |b| {
        b.iter(|| from_str(black_box(AAT_TASK)).unwrap())
    });

    group.bench_function("racing_task", |b| {
        b.iter(|| from_str(black_box(RACING_TASK)).unwrap())
    });

    group.bench_function("fai_task", |b| {
        b.iter(|| from_str(black_box(FAI_TASK)).unwrap())
    });

    group.bench_function("all_oz_types", |b| {
        b.iter(|| from_str(black_box(ALL_OZ_TYPES)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, bench_from_str);
criterion_main!(benches);
