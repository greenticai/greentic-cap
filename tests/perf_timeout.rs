#[path = "perf_support/mod.rs"]
mod perf_support;

use std::time::{Duration, Instant};

use greentic_cap_schema::check_pack_capability_compatibility;

#[test]
fn workload_should_finish_quickly() {
    let case = perf_support::pack_case();
    let start = Instant::now();

    for _ in 0..48 {
        let reports = check_pack_capability_compatibility(&case.section, &case.component)
            .expect("compatibility");
        std::hint::black_box(reports);
    }

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(2),
        "workload too slow: {:?}",
        elapsed
    );
}
