//! Latency benchmark: mask + reidentify overhead. Target <20ms.

use unblock_proxy::masking::{mask_text, reidentify_map, spans_from_regex_only};

fn main() {
    let text = "Contact Jane at jane@example.com or 555-123-4567 for the report.";
    let runs = 1000;
    let mut total_mask_ns = 0_u64;
    let mut total_reid_ns = 0_u64;

    for _ in 0..runs {
        let start = std::time::Instant::now();
        let spans = spans_from_regex_only(text);
        let (masked, mapping) = mask_text(text, &spans);
        total_mask_ns += start.elapsed().as_nanos() as u64;

        let map: std::collections::HashMap<String, String> = mapping.into_iter().collect();
        let start2 = std::time::Instant::now();
        let _ = reidentify_map(&masked, &map);
        total_reid_ns += start2.elapsed().as_nanos() as u64;
    }

    let mask_mean_us = total_mask_ns as f64 / runs as f64 / 1000.0;
    let reid_mean_us = total_reid_ns as f64 / runs as f64 / 1000.0;
    let total_mean_ms = (total_mask_ns + total_reid_ns) as f64 / runs as f64 / 1_000_000.0;

    println!("runs={}", runs);
    println!("mask_mean_us={:.2}", mask_mean_us);
    println!("reidentify_mean_us={:.2}", reid_mean_us);
    println!("total_overhead_ms={:.4}", total_mean_ms);
    assert!(
        total_mean_ms < 20.0,
        "total overhead must be <20ms (got {:.4}ms)",
        total_mean_ms
    );
}
