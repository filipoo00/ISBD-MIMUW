// Generates a sample file with columns: INT64, BOOLEAN, VARCHAR, VARCHAR.
//
// Usage: generate <path> [number_of_rows]

use std::env;
use std::io;
use std::path::Path;
use std::process;

use proj2::table::BATCH_SIZE;
use proj2::file_format::writer::Writer;
use proj2::table::{Batch, Column, ColumnType, VarcharColumn};

const DEFAULT_NUMBER_OF_ROWS: usize = 10000;
const FIRST_ID: i64 = -5000;
const WORDS: [&str; 6] = ["a", "ab", "abc", "ą", "ąb", "ąbc"];
const COLUMN_TYPES: [ColumnType; 4] = [
    ColumnType::Int64,
    ColumnType::Boolean,
    ColumnType::Varchar,
    ColumnType::Varchar,
];


fn make_batch(first_row: usize, number_of_rows: usize) -> io::Result<Batch> {
    let mut ids = Vec::with_capacity(number_of_rows);
    let mut flags = Vec::with_capacity(number_of_rows);
    let mut names = VarcharColumn::new();
    let mut words = VarcharColumn::new();

    for row in first_row..first_row + number_of_rows {
        ids.push(FIRST_ID + row as i64);
        flags.push(row % 3 == 0);
        names.push(&format!("row_{row}"));
        words.push(WORDS[row % WORDS.len()]);
    }

    Batch::new(vec![
        Column::Int64(ids),
        Column::Boolean(flags),
        Column::Varchar(names),
        Column::Varchar(words),
    ])
}


fn run(path: &Path, number_of_rows: usize) -> io::Result<()> {
    let mut writer = Writer::create(path, COLUMN_TYPES.to_vec())?;

    let mut first_row = 0;
    while first_row < number_of_rows {
        let batch_rows = BATCH_SIZE.min(number_of_rows - first_row);
        writer.write_batch(&make_batch(first_row, batch_rows)?)?;
        first_row += batch_rows;
    }

    writer.finish()?;

    println!("Written {number_of_rows} rows to {}.", path.display());
    Ok(())
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <path> [number_of_rows]", args[0]);
        process::exit(1);
    }

    let path = Path::new(&args[1]);
    let number_of_rows = match args.get(2) {
        Some(value) => match value.parse() {
            Ok(number) => number,
            Err(_) => {
                eprintln!("Error: number_of_rows must be a non-negative integer.");
                process::exit(1);
            }
        },
        None => DEFAULT_NUMBER_OF_ROWS,
    };

    if let Err(error) = run(path, number_of_rows) {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
