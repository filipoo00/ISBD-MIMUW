use std::io;
use std::path::Path;
use std::fs::File;


use crate::file_format::{ColumnMetadata, FileMetadata, read_column};


pub fn read_file(path: &Path) -> io::Result<()> {
    let mut file = File::open(path)?;
    let metadata = FileMetadata::read_from(&mut file)?;
    let number_of_cols = metadata.number_of_cols;
    let number_of_rows = metadata.number_of_rows;

    for _ in 0..number_of_cols {
        let column_metadata = 
            ColumnMetadata::read_from(&mut file).unwrap();

        let values = read_column(&mut file, number_of_rows, &column_metadata).unwrap();
        println!("{:?}", values);
    }

    Ok(())
}