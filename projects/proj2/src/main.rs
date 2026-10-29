mod codec;
mod table;
mod constants;
mod file_format;

use std::{eprintln, println};
use std::path::Path;
use crate::file_format::writer::write_file;
use crate::file_format::reader::read_file;
use crate::table::{Column, Table};


fn main() {
    let path = Path::new("xd.bin");
    let column1 = Column::Int64(vec![10, 20]);
    let column2 = Column::Int64(vec![1, 2]);
    let column3 = Column::Varchar(vec!["xd".to_string(), "lol".to_string()]);
    let column4 = Column::Boolean(vec![true, false]);
    let table = Table {
        columns: vec![column1, column2, column3, column4],
    };
    if let Err(error) = write_file(path, &table) {
        eprintln!("Cannot write table: {error}");
        return;
    }
    let table = read_file(path).unwrap();
    println!("{:?}", table.columns);
} 