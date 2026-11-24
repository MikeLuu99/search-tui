// Debug helper: time each engine individually and the full fan-out, mirroring
// what the TUI does on startup. Run with:
//   cargo run --release --example engine_timing -- "rust programming"
use std::sync::Arc;
use std::time::{Duration, Instant};

use metadata_search_engine_rs::engines::{
    BraveEngine, DuckDuckGoEngine, SearchEngine, StartpageEngine, YahooEngine, build_http_client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let query = std::env::args().nth(1).unwrap_or_else(|| "rust".into());
    let client = Arc::new(build_http_client()?);
    let engines: Vec<Arc<dyn SearchEngine>> = vec![
        Arc::new(DuckDuckGoEngine::new(Arc::clone(&client))),
        Arc::new(BraveEngine::new(Arc::clone(&client))),
        Arc::new(StartpageEngine::new(Arc::clone(&client))),
        Arc::new(YahooEngine::new(Arc::clone(&client))),
    ];

    for engine in &engines {
        let t0 = Instant::now();
        let n = match engine.search(&query, 10).await {
            Ok(rs) => rs.len(),
            Err(e) => {
                println!(
                    "{:>10}  {:>8.1}s  ERROR {e}",
                    engine.name(),
                    t0.elapsed().as_secs_f64()
                );
                continue;
            }
        };
        println!(
            "{:>10}  {:>8.1}s  {} results",
            engine.name(),
            t0.elapsed().as_secs_f64(),
            n
        );
    }

    // Full fan-out with the TUI's limits (concurrency 1, no deadline).
    let limits = metadata_search_engine_rs::cache::EngineLimits::new(1, Duration::from_secs(5));
    let t0 = Instant::now();
    let (successes, failures) =
        metadata_search_engine_rs::aggregator::query_all_engines(&engines, &limits, &query, 10)
            .await;
    println!(
        "\nfan-out (no deadline): {:.1}s, {} ok / {} failed ({})",
        t0.elapsed().as_secs_f64(),
        successes.len(),
        failures.len(),
        failures
            .iter()
            .map(|f| f.0.clone())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Same fan-out with the TUI's new deadline.
    let limits = limits.with_deadline(Some(Duration::from_millis(3000)));
    let t0 = Instant::now();
    let (successes, failures) =
        metadata_search_engine_rs::aggregator::query_all_engines(&engines, &limits, &query, 10)
            .await;
    println!(
        "fan-out (3s deadline): {:.1}s, {} ok / {} failed ({})",
        t0.elapsed().as_secs_f64(),
        successes.len(),
        failures.len(),
        failures
            .iter()
            .map(|f| f.0.clone())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}
