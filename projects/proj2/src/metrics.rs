use std::io;

use crate::table::{Batch, Column, ColumnType};


enum Metric {
    Int64 { sum: i128, count: u64 },
    Varchar { ascii_count: u64 },
    Boolean { true_count: u64, false_count: u64 },
}


pub enum MetricResult {
    Int64 {
        mean: Option<f64>,
    },
    Varchar {
        ascii_count: u64,
    },
    Boolean {
        entropy: Option<f64>,
    },
}


pub struct ColumnsMetric {
    column_types: Vec<ColumnType>,
    columns_metric: Vec<Metric>,
}


impl ColumnsMetric {
    pub fn new(column_types: &[ColumnType]) -> Self {
        let mut columns_metric = Vec::new();

        for column_type in column_types {
            let column_metric = match column_type {
                ColumnType::Int64 => {
                    Metric::Int64 { sum: 0, count: 0 }
                },
                ColumnType::Varchar => {
                    Metric::Varchar { ascii_count: 0 }
                },
                ColumnType::Boolean => {
                    Metric::Boolean { true_count: 0, false_count: 0 }
                },
            };

            columns_metric.push(column_metric);
        }

        Self { column_types: column_types.to_vec(), columns_metric }
    }


    pub fn update(&mut self, batch: &Batch) -> io::Result<()> {
        if !batch.matches_schema(&self.column_types) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Batch schema does not match metrics schema.",
            ));
        }

        for (metric, column) in self.columns_metric.iter_mut().zip(batch.columns()) {
            match (metric, column) {
                (Metric::Int64 { sum, count }, Column::Int64(values)) => {
                    for &value in values {
                        *sum = sum.checked_add(value as i128).ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Integer overflow",
                            )
                        })?;
                    }
                    *count += values.len() as u64;
                },
                (Metric::Varchar { ascii_count }, Column::Varchar(values)) => {
                    let count = values.data().bytes().filter(|b| b.is_ascii()).count();
                    *ascii_count += count as u64;
                },
                (Metric::Boolean { true_count, false_count }, Column::Boolean(values)) => {
                    for &value in values {
                        if value { *true_count += 1 } else { *false_count += 1 }
                    }
                },
                (_, _) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Batch schema does not match metrics schema.",
                    ));
                }
            }
        }

        Ok(())
    }


    pub fn calculate(&self) -> Vec<MetricResult> {
        let mut columns_metric_result = Vec::new();

        for column_metric in &self.columns_metric {
            let column_metric_result = match column_metric {
                Metric::Int64 { sum, count } => {
                    MetricResult::Int64 {
                        mean: if *count > 0 {
                            Some(*sum as f64 / *count as f64)
                        } else {
                            None
                        }
                    }
                }
                Metric::Varchar { ascii_count } => {
                    let ascii_count = *ascii_count;
                    MetricResult::Varchar { ascii_count }
                },
                Metric::Boolean { true_count, false_count } => {
                    let total = true_count + false_count;

                    let entropy = if total > 0 {
                        let p = *true_count as f64 / total as f64;
                        let q = *false_count as f64 / total as f64;

                        let h_true = if p > 0.0 { -p * p.log2() } else { 0.0 };
                        let h_false = if q > 0.0 { -q * q.log2() } else { 0.0 };

                        Some(h_true + h_false)
                    } else {
                        None
                    };
                    MetricResult::Boolean { 
                        entropy: entropy
                    }
                },
            };

            columns_metric_result.push(column_metric_result)
        }

        columns_metric_result
    }    
}
