use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fileemu_utoc_stream_emulator::asset_collector;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("low-sample-size");
    bench_group.significance_level(0.1).sample_size(10);
    bench_group.bench_function("asset collector", |b| b.iter(|| {
        let mods = vec!["p3rpc.catherinefont", "p3rpc.classroomcheatsheet", "p3rpc.controlleruioverhaul.xbox", 
        "p3rpc.femc", "p3rpc.isitworking", "p3rpc.modmenu", "p3rpc.nocinematicbars", "p3rpc.removetalkfromdialogue",
        "p3rpc.rewatchtv", "p3rpc.ryojioutfit", "p3rpc.usefuldescriptions"];
        let base_path = std::env::var("RELOADEDIIMODS").unwrap_or_else(|err| panic!("Environment variable \"RELOADEDIIMODS\" is missing"));
        for curr_mod in mods {
            asset_collector::add_from_folders(curr_mod, &(base_path.clone() + "/" + curr_mod + "/UnrealEssentials"));
        }
        asset_collector::print_asset_collector_results();
    }));
    bench_group.finish();
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(benches);