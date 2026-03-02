//! NER latency benchmark: measures ONNX NER inference time if NER_MODEL_DIR is set.
//! Run: NER_MODEL_DIR=models/bert-base-NER-onnx cargo bench --bench ner_latency

use unblock_proxy::ner::NerEngine;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let engine = NerEngine::from_env().expect("ner engine");

    let samples = [
        "The company was founded in New York by John Smith.",
        "Dr. Jane Doe works at the University of California.",
        "Please send the report to the office in London by Monday.",
        "Contact user@example.com or 555-123-4567 for help. SSN 123-45-6789.",
    ];

    let warmup = 5;
    let runs = 50;

    // Warmup
    for _ in 0..warmup {
        for text in &samples {
            rt.block_on(engine.detect_async(text.to_string()))
                .expect("detect");
        }
    }

    // Measure
    let mut all_ms: Vec<f64> = Vec::new();
    for _ in 0..runs {
        for text in &samples {
            let start = std::time::Instant::now();
            let spans = rt
                .block_on(engine.detect_async(text.to_string()))
                .expect("detect");
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            all_ms.push(elapsed_ms);
            // suppress unused warning
            let _ = spans.len();
        }
    }

    all_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean: f64 = all_ms.iter().sum::<f64>() / all_ms.len() as f64;
    let median = all_ms[all_ms.len() / 2];
    let p95 = all_ms[(all_ms.len() as f64 * 0.95) as usize];
    let p99 = all_ms[(all_ms.len() as f64 * 0.99) as usize];

    let onnx_mode = std::env::var("NER_MODEL_DIR").is_ok();
    println!(
        "mode={}",
        if onnx_mode {
            "ONNX+regex"
        } else {
            "regex-only"
        }
    );
    println!("samples={}", samples.len());
    println!("runs={}", runs);
    println!("total_inferences={}", all_ms.len());
    println!("mean_ms={:.3}", mean);
    println!("median_ms={:.3}", median);
    println!("p95_ms={:.3}", p95);
    println!("p99_ms={:.3}", p99);
    println!(
        "target=<5ms  result={}",
        if mean < 5.0 { "PASS" } else { "FAIL" }
    );
}
