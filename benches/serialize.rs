use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use xcsoar_tasks::{from_str, to_string, to_string_pretty};

const AAT_TASK: &str = include_str!("../fixtures/aat-task.tsk");
const RACING_TASK: &str = include_str!("../fixtures/racing-task.tsk");
const FAI_TASK: &str = include_str!("../fixtures/fai-task.tsk");
const ALL_OZ_TYPES: &str = include_str!("../fixtures/all-oz-types.tsk");

fn bench_to_string(c: &mut Criterion) {
    let aat_task = from_str(AAT_TASK).unwrap();
    let racing_task = from_str(RACING_TASK).unwrap();
    let fai_task = from_str(FAI_TASK).unwrap();
    let all_oz_types = from_str(ALL_OZ_TYPES).unwrap();

    let mut group = c.benchmark_group("to_string");

    group.bench_function("aat_task", |b| {
        b.iter(|| to_string(black_box(&aat_task)).unwrap())
    });

    group.bench_function("racing_task", |b| {
        b.iter(|| to_string(black_box(&racing_task)).unwrap())
    });

    group.bench_function("fai_task", |b| {
        b.iter(|| to_string(black_box(&fai_task)).unwrap())
    });

    group.bench_function("all_oz_types", |b| {
        b.iter(|| to_string(black_box(&all_oz_types)).unwrap())
    });

    group.finish();
}

fn bench_to_string_pretty(c: &mut Criterion) {
    let aat_task = from_str(AAT_TASK).unwrap();
    let racing_task = from_str(RACING_TASK).unwrap();
    let fai_task = from_str(FAI_TASK).unwrap();
    let all_oz_types = from_str(ALL_OZ_TYPES).unwrap();

    let mut group = c.benchmark_group("to_string_pretty");

    group.bench_function("aat_task", |b| {
        b.iter(|| to_string_pretty(black_box(&aat_task)).unwrap())
    });

    group.bench_function("racing_task", |b| {
        b.iter(|| to_string_pretty(black_box(&racing_task)).unwrap())
    });

    group.bench_function("fai_task", |b| {
        b.iter(|| to_string_pretty(black_box(&fai_task)).unwrap())
    });

    group.bench_function("all_oz_types", |b| {
        b.iter(|| to_string_pretty(black_box(&all_oz_types)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, bench_to_string, bench_to_string_pretty);
criterion_main!(benches);
