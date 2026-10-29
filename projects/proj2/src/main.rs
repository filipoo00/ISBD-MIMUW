mod codec;
mod batch;
mod file_format;
mod writer;
mod reader;
mod constants;

use std::eprintln;
use std::path::Path;
use crate::writer::write_file;
use crate::reader::read_file;
use crate::batch::Column;


fn main() {
    let path = Path::new("xd.bin");
    let column1 = Column::Int64(vec![10, 20]);
    let column2 = Column::Int64(vec![1, 2]);
    let column3 = Column::Varchar(vec!["xd".to_string(), "lol".to_string()]);
    let column4 = Column::Boolean(vec![true, false]);
    let columns = vec![column1, column2, column3, column4]; 
    if let Err(error) = write_file(path, &columns) {
        eprintln!("Cannot write table: {error}");
        return;
    }
    read_file(path).unwrap();
}