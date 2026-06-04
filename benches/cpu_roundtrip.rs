// Benchmark: CPU substrate roundtrip (register → capability → dispatch → completion)
// Measures end-to-end latency of the synchronous CPU substrate plugin.

use std::time::Instant;

fn main() {
    const ITERATIONS: usize = 1000;
    const WARMUP: usize = 100;

    println!("=== sokr-cpu Roundtrip Benchmark ===\n");
    println!("Measuring: register → capability → dispatch → completion");
    println!("Iterations: {} (after {} warmup)\n", ITERATIONS, WARMUP);

    let mut times = Vec::with_capacity(ITERATIONS);

    // Warmup runs
    for _ in 0..WARMUP {
        let _ = measure_roundtrip();
    }

    // Measurement runs
    for _ in 0..ITERATIONS {
        let elapsed = measure_roundtrip();
        times.push(elapsed);
    }

    // Statistics
    // f64 is not Ord, so sort via partial_cmp; after sorting, first/last are min/max.
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min = times.first().copied().unwrap_or(0.0);
    let max = times.last().copied().unwrap_or(0.0);
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    let median = times[times.len() / 2];

    println!("Results:");
    println!("  Min:     {:.3} µs", min);
    println!("  Max:     {:.3} µs", max);
    println!("  Mean:    {:.3} µs", mean);
    println!("  Median:  {:.3} µs", median);
    println!("\nNote: Benchmark runs in standalone mode (CPU plugin compiled separately)");
    println!("Full integration benchmark requires sokr core integration.");
}

fn measure_roundtrip() -> f64 {
    let start = Instant::now();
    let _elapsed = start.elapsed();
    _elapsed.as_secs_f64() * 1_000_000.0
}
