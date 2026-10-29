use std::path::Path;
use std::fs::File;
use std::io;

use crate::batch::Column;
use crate::file_format::{FileMetadata, write_column};


pub fn write_file(path: &Path, columns: &[Column]) -> io::Result<()> {
    let mut file = File::create(path).unwrap();

    let number_of_rows = columns.first().map_or(0, |column| column.len());
    if columns.iter().any(|column| column.len() != number_of_rows) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Every column have to have the same number of rows."
        ));
    };

    let number_of_cols: u32 = columns.len().try_into().map_err(|_| {
        io::Error::new(
        io::ErrorKind::InvalidInput,
        "Too many columns."
        )
    })?;
    let file_metadata = FileMetadata {
        number_of_cols: number_of_cols,
        number_of_rows: number_of_rows as u64,
    };

    file_metadata.write_to(&mut file).unwrap();
    for column in columns {
        write_column(&mut file, &column).unwrap();
    }

    Ok(())
}
