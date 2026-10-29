use std::path::Path;
use std::fs::File;
use std::io::{self, Write};
use std::mem;

use crate::file_format::metadata::{ColumnMetadata, FileMetadata};
use crate::table::{Column, Table};
use crate::codec::int64::encode_int64;
use crate::codec::varchar::encode_varchar_column;
use crate::codec::boolean::encode_boolean_column;
use crate::constants::{TYPE_INT64, TYPE_VARCHAR, TYPE_BOOLEAN};

pub fn write_column(file: &mut File, column: &Column) -> io::Result<()> {
    let (column_type, encoded, original_size) = match &column {
        Column::Int64(values) => {
            let encoded = encode_int64(values);
            let original_size = (values.len() * mem::size_of::<i64>()) as u64;
            (TYPE_INT64, encoded, original_size)
        },
        Column::Varchar(values) => {
            let (encdoded, original_size) = encode_varchar_column(values);
            (TYPE_VARCHAR, encdoded, original_size)
        }
        Column::Boolean(values) => {
            let encoded = encode_boolean_column(&values);
            let original_size = (values.len() * mem::size_of::<bool>()) as u64;
            (TYPE_BOOLEAN, encoded, original_size)
        }
    };


    let metadata = ColumnMetadata {
        column_type: column_type,
        original_size: original_size,
        payload_size: encoded.len() as u64,
    };

    metadata.write_to(file)?;
    file.write_all(&encoded)?;

    Ok(())
}

pub fn write_file(path: &Path, table: &Table) -> io::Result<()> {
    let mut file = File::create(path).unwrap();

    let columns = &table.columns;
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
