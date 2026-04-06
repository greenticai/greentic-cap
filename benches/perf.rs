#[path = "../tests/perf_support/mod.rs"]
mod perf_support;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use greentic_cap_resolver::resolve_root;
use greentic_cap_schema::check_pack_capability_compatibility;

fn bench_resolve_root(c: &mut Criterion) {
    let case = perf_support::resolution_case();
    c.bench_function("resolve_root_hot_path", |b| {
        b.iter(|| {
            black_box(resolve_root(black_box(&case.declaration)).expect("resolve"));
        })
    });
}

fn bench_pack_compatibility(c: &mut Criterion) {
    let case = perf_support::pack_case();
    c.bench_function("pack_compatibility_hot_path", |b| {
        b.iter(|| {
            black_box(
                check_pack_capability_compatibility(
                    black_box(&case.section),
                    black_box(&case.component),
                )
                .expect("compatibility"),
            );
        })
    });
}

criterion_group!(benches, bench_resolve_root, bench_pack_compatibility);
criterion_main!(benches);
