use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lc3sim_project::{
    executors::{core::CoreLC3, populate_from_bin, LC3},
    harnesses::{simple::FailIO, sync::step_continue},
};

macro_rules! setup_lc3 {
    ( $lc3:ident, $path:literal ) => {
        || {
            let mut lc3 = $lc3.clone();
            populate_from_bin(&mut lc3, include_bytes!("../penn_sim/lc3os.obj").as_slice());
            populate_from_bin(&mut lc3, include_bytes!($path).as_slice());
            lc3
        }
    };
}

pub fn create_new(c: &mut Criterion) {
    let mut c = c.benchmark_group("create_new");

    macro_rules! bench_new {
        ( $lc3:expr, $name: literal ) => {
            c.bench_function($name, |b| {
                b.iter($lc3);
            });
        };
    }

    bench_new!(CoreLC3::new, "core");
    #[cfg(feature = "consolidated")]
    bench_new!(
        lc3sim_project::executors::consolidated::ConsolidatedLC3::boxed,
        "consolidated"
    );
    #[cfg(feature = "cached_resolve")]
    bench_new!(
        lc3sim_project::executors::cached_resolve::CachedResolveLC3::boxed,
        "cached_resolve"
    );
    #[cfg(feature = "instruction_mem")]
    bench_new!(
        lc3sim_project::executors::instruction_mem::InstMemLC3::boxed,
        "instruction_mem"
    );
}

pub fn load_os(c: &mut Criterion) {
    let mut c = c.benchmark_group("load_os");

    fn exec_setup<E: LC3>(mut lc3: E) -> E {
        populate_from_bin(&mut lc3, include_bytes!("../penn_sim/lc3os.obj").as_slice());
        black_box(lc3)
    }

    macro_rules! bench_load {
        ( $lc3:expr, $name: literal ) => {
            c.bench_function($name, |b| {
                b.iter_batched($lc3, exec_setup, criterion::BatchSize::SmallInput);
            });
        };
    }

    bench_load!(CoreLC3::new, "core");
    #[cfg(feature = "consolidated")]
    bench_load!(
        lc3sim_project::executors::consolidated::ConsolidatedLC3::boxed,
        "consolidated"
    );
    #[cfg(feature = "cached_resolve")]
    bench_load!(
        lc3sim_project::executors::cached_resolve::CachedResolveLC3::boxed,
        "cached_resolve"
    );
    #[cfg(feature = "instruction_mem")]
    bench_load!(
        lc3sim_project::executors::instruction_mem::InstMemLC3::boxed,
        "instruction_mem"
    );
}

pub fn tiny_loop(c: &mut Criterion) {
    let mut c = c.benchmark_group("tiny_loop");
    let c = c.measurement_time(Duration::from_secs(20));

    fn exec_loop<E: LC3>(mut lc3: E) {
        step_continue(&mut FailIO, &mut lc3).unwrap();
    }

    macro_rules! bench_loop {
        ( $lc3:expr, $name: literal ) => {
            let lc3 = $lc3;
            c.bench_function($name, |b| {
                b.iter_batched(
                    setup_lc3!(lc3, "../test_data/custom/loop.obj"),
                    exec_loop,
                    criterion::BatchSize::SmallInput,
                );
            });
        };
    }

    bench_loop!(CoreLC3::new(), "core");
    #[cfg(feature = "consolidated")]
    bench_loop!(
        lc3sim_project::executors::consolidated::ConsolidatedLC3::boxed(),
        "consolidated"
    );
    #[cfg(feature = "cached_resolve")]
    bench_loop!(
        lc3sim_project::executors::cached_resolve::CachedResolveLC3::boxed(),
        "cached_resolve"
    );
    #[cfg(feature = "instruction_mem")]
    bench_loop!(
        lc3sim_project::executors::instruction_mem::InstMemLC3::boxed(),
        "instruction_mem"
    );
}

criterion_group!(speed_compare, create_new, load_os, tiny_loop);
criterion_main!(speed_compare);
