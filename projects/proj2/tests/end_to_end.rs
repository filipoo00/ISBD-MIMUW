use std::fs;
use std::io;
use std::path::Path;

use proj2::file_format::reader::Reader;
use proj2::file_format::writer::Writer;
use proj2::metrics::{ColumnsMetric, MetricResult};
use proj2::table::{Batch, Column, ColumnType, VarcharColumn, BATCH_SIZE};
use tempfile::NamedTempFile;

const SCHEMA: [ColumnType; 4] = [
    ColumnType::Int64,
    ColumnType::Boolean,
    ColumnType::Varchar,
    ColumnType::Varchar,
];

const WORDS: [&str; 4] = ["a", "", "ą", "abcde"];


fn varchar_column(values: &[&str]) -> VarcharColumn {
    let mut column = VarcharColumn::new();
    for value in values {
        column.push(value);
    }
    column
}

fn make_batch(first_row: usize, number_of_rows: usize) -> Batch {
    let mut ids = Vec::with_capacity(number_of_rows);
    let mut flags = Vec::with_capacity(number_of_rows);
    let mut names = VarcharColumn::new();
    let mut words = VarcharColumn::new();

    for row in first_row..first_row + number_of_rows {
        let value = row as i64 * 1000001;
        ids.push(if row % 2 == 0 { value } else { -value });
        flags.push(row % 3 == 0);
        names.push(&format!("row_{row}"));
        words.push(WORDS[row % WORDS.len()]);
    }

    Batch::new(vec![
        Column::Int64(ids),
        Column::Boolean(flags),
        Column::Varchar(names),
        Column::Varchar(words),
    ]).unwrap()
}

fn write_file(path: &Path, batches: &[Batch]) {
    let mut writer = Writer::create(path, SCHEMA.to_vec()).unwrap();
    for batch in batches {
        writer.write_batch(batch).unwrap();
    }
    writer.finish().unwrap();
}

fn read_all(path: &Path) -> io::Result<Vec<Batch>> {
    let mut reader = Reader::open(path)?;
    let mut batches = Vec::new();
    while let Some(batch) = reader.next_batch()? {
        batches.push(batch);
    }
    Ok(batches)
}


#[test]
fn roundtrip_multiple_batches() {
    let file = NamedTempFile::new().unwrap();
    let batches = vec![
        make_batch(0, BATCH_SIZE),
        make_batch(BATCH_SIZE, BATCH_SIZE),
        make_batch(2 * BATCH_SIZE, 500),
    ];

    write_file(file.path(), &batches);

    let mut reader = Reader::open(file.path()).unwrap();
    for expected in &batches {
        let batch = reader.next_batch().unwrap().unwrap();
        assert_eq!(&batch, expected);
    }
    assert!(reader.next_batch().unwrap().is_none());
}

#[test]
fn schema_is_read_from_footer() {
    let file = NamedTempFile::new().unwrap();
    write_file(file.path(), &[make_batch(0, 10)]);

    let reader = Reader::open(file.path()).unwrap();

    assert_eq!(reader.schema(), &SCHEMA);
}

#[test]
fn file_without_batches() {
    let file = NamedTempFile::new().unwrap();
    write_file(file.path(), &[]);

    let mut reader = Reader::open(file.path()).unwrap();

    assert_eq!(reader.schema(), &SCHEMA);
    assert!(reader.next_batch().unwrap().is_none());
}

#[test]
fn writer_rejects_wrong_schema() {
    let file = NamedTempFile::new().unwrap();
    let mut writer = Writer::create(file.path(), SCHEMA.to_vec()).unwrap();

    let wrong_type = Batch::new(vec![
        Column::Int64(vec![1]),
        Column::Int64(vec![2]),
        Column::Varchar(varchar_column(&["a"])),
        Column::Varchar(varchar_column(&["b"])),
    ]).unwrap();
    let wrong_count = Batch::new(vec![Column::Int64(vec![1])]).unwrap();

    assert!(writer.write_batch(&wrong_type).is_err());
    assert!(writer.write_batch(&wrong_count).is_err());

    let good = make_batch(0, 3);
    writer.write_batch(&good).unwrap();
    writer.finish().unwrap();

    assert_eq!(read_all(file.path()).unwrap(), vec![good]);
}

#[test]
fn metrics_on_known_data() {
    let file = NamedTempFile::new().unwrap();
    let first = Batch::new(vec![
        Column::Int64(vec![1, 2]),
        Column::Boolean(vec![true, false]),
        Column::Varchar(varchar_column(&["a", "b"])),
        Column::Varchar(varchar_column(&["c", "d"])),
    ]).unwrap();
    let second = Batch::new(vec![
        Column::Int64(vec![3, 4]),
        Column::Boolean(vec![true, false]),
        Column::Varchar(varchar_column(&["ą", ""])),
        Column::Varchar(varchar_column(&["", "ę"])),
    ]).unwrap();
    write_file(file.path(), &[first, second]);

    let mut reader = Reader::open(file.path()).unwrap();
    let mut metrics = ColumnsMetric::new(reader.schema());
    while let Some(batch) = reader.next_batch().unwrap() {
        metrics.update(&batch).unwrap();
    }

    assert_eq!(metrics.calculate(), vec![
        MetricResult::Int64 { mean: Some(2.5) },
        MetricResult::Boolean { entropy: Some(1.0) },
        MetricResult::Varchar { ascii_count: 2 },
        MetricResult::Varchar { ascii_count: 2 },
    ]);
}

#[test]
fn truncated_file_is_error() {
    let file = NamedTempFile::new().unwrap();
    write_file(file.path(), &[make_batch(0, 100), make_batch(100, 100)]);
    let bytes = fs::read(file.path()).unwrap();

    for cut in [1, 10, bytes.len() / 2] {
        fs::write(file.path(), &bytes[..bytes.len() - cut]).unwrap();
        assert!(read_all(file.path()).is_err(), "file cut by {cut} bytes was read");
    }
}