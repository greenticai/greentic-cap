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

    // Every worker performs the full workload, so total work grows with the thread count.
    // Normalize wall time per worker to measure contention rather than assuming the CI runner
    // has two or four physical cores available. A 1.75x allowance still catches strongly
    // superlinear degradation while tolerating scheduler and thread-startup noise.
    let per_worker_1 = t1;
    let per_worker_2 = t2.div_f64(2.0);
    let per_worker_4 = t4.div_f64(4.0);

    assert!(
        per_worker_2 <= per_worker_1.mul_f64(1.75),
        "2-thread contention exceeded limit: total(t1={:?}, t2={:?}), per-worker(t1={:?}, t2={:?})",
        t1,
        t2,
        per_worker_1,
        per_worker_2,
    );

    assert!(
        per_worker_4 <= per_worker_2.mul_f64(1.75),
        "4-thread contention exceeded limit: total(t2={:?}, t4={:?}), per-worker(t2={:?}, t4={:?})",
        t2,
        t4,
        per_worker_2,
        per_worker_4,
    );
}
