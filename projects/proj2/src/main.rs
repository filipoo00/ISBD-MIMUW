mod codec;
mod table;
mod constants;
mod file_format;
mod metrics;

use std::path::Path;
use std::println;

use crate::file_format::writer::Writer;
use crate::file_format::reader::Reader;
use crate::table::{Batch, Column};
use crate::metrics::{MetricResult, ColumnsMetric};


fn print_results(results: &[MetricResult]) {
    println!("\nMetrics");

    for (i, result) in results.iter().enumerate() {
        match result {
            MetricResult::Int64 { mean } => {
                println!("Column {i}: mean = {mean:?}");
            }
            MetricResult::Boolean { entropy } => {
                println!("Column {i}: entropy = {entropy:?}");
            }
            MetricResult::Varchar { ascii_count } => {
                println!("Column {i}: ascii_count = {ascii_count}");
            }
        }
    }
}


fn main() {
    let path = Path::new("xd.bin");

    let batch1 = Batch::new(vec![
        Column::Int64(vec![1, 2]),
        Column::Int64(vec![10, 20]),
        Column::Varchar(vec!["1".to_string(), "2".to_string()]),
        Column::Boolean(vec![true, false]),
    ]).unwrap();

    let batch2 = Batch::new(vec![
        Column::Int64(vec![3, 4]),
        Column::Int64(vec![30, 40]),
        Column::Varchar(vec!["3".to_string(), "4".to_string()]),
        Column::Boolean(vec![false, true]),
    ]).unwrap();

    // Write batches
    let mut writer = Writer::create(path, 4).unwrap();

    writer.write_batch(&batch1).unwrap();
    writer.write_batch(&batch2).unwrap();

    writer.finish().unwrap();

    println!("Written 2 batches.");

    // Read batches and calculate metrics
    let mut reader = Reader::open(path).unwrap();

    let mut metrics: Option<ColumnsMetric> = None;
    let mut number_of_batch = 0;

    while let Some(batch) = reader.next_batch().unwrap() {
        println!("Batch {}", number_of_batch);
        for (i, column) in batch.columns().iter().enumerate() {
            println!("Column {}: {:?}", i, column);
        }
        if metrics.is_none() {
            metrics = Some(ColumnsMetric::new(&batch));
        }
        metrics.as_mut().unwrap().update(&batch).unwrap();

        number_of_batch += 1;
    }

    if let Some(metrics) = metrics {
        let results = metrics.calculate();
        print_results(&results);
    }
}