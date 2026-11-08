// Reads a file and computes metrics for every column:
//  INT64:   mean
//  BOOLEAN: binary entropy
//  VARCHAR: number of ASCII characters
//
// Usage: metrics <path>

use std::env;
use std::io;
use std::path::Path;
use std::process;

use proj2::file_format::reader::Reader;
use proj2::metrics::{ColumnsMetric, MetricResult};


fn print_results(results: &[MetricResult]) {
    for (i, result) in results.iter().enumerate() {
        match result {
            MetricResult::Int64 { mean: Some(mean) } => {
                println!("Column {i} (INT64): mean = {mean}");
            }
            MetricResult::Int64 { mean: None } => {
                println!("Column {i} (INT64): mean = none (no rows)");
            }
            MetricResult::Boolean { entropy: Some(entropy) } => {
                println!("Column {i} (BOOLEAN): entropy = {entropy}");
            }
            MetricResult::Boolean { entropy: None } => {
                println!("Column {i} (BOOLEAN): entropy = none (no rows)");
            }
            MetricResult::Varchar { ascii_count } => {
                println!("Column {i} (VARCHAR): ascii_count = {ascii_count}");
            }
        }
    }
}


fn run(path: &Path) -> io::Result<()> {
    let mut reader = Reader::open(path)?;

    let mut metrics: Option<ColumnsMetric> = None;
    let mut number_of_batches = 0;
    let mut number_of_rows = 0;

    while let Some(batch) = reader.next_batch()? {
        let metrics = metrics.get_or_insert_with(|| ColumnsMetric::new(&batch));
        metrics.update(&batch)?;

        number_of_batches += 1;
        number_of_rows += batch.number_of_rows();
    }

    println!("Read {number_of_rows} rows in {number_of_batches} batches.");

    match metrics {
        Some(metrics) => print_results(&metrics.calculate()),
        None => println!("File has no batches."),
    }

    Ok(())
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path>", args[0]);
        process::exit(1);
    }

    if let Err(error) = run(Path::new(&args[1])) {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
