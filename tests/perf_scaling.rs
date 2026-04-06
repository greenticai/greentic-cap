#[path = "perf_support/mod.rs"]
mod perf_support;

use std::{
    hint::black_box,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use greentic_cap_schema::check_pack_capability_compatibility;

fn run_workload(threads: usize) -> Duration {
    let case = Arc::new(perf_support::pack_case());
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let case = Arc::clone(&case);
            thread::spawn(move || {
                let mut total = 0usize;
                for _ in 0..24 {
                    let reports =
                        check_pack_capability_compatibility(&case.section, &case.component)
                            .expect("compatibility");
                    total += black_box(reports.len());
                }
                black_box(total)
            })
        })
        .collect();

    for handle in handles {
        black_box(handle.join().expect("thread"));
    }

    start.elapsed()
}

#[test]
fn scaling_should_not_degrade_badly() {
    let t1 = run_workload(1);
    let t2 = run_workload(2);
    let t4 = run_workload(4);

    assert!(
        t2 <= t1.mul_f64(1.75),
        "2 threads slower than expected: t1={:?}, t2={:?}",
        t1,
        t2
    );

    assert!(
        t4 <= t2.mul_f64(1.75),
        "4 threads slower than expected: t2={:?}, t4={:?}",
        t2,
        t4
    );
}
