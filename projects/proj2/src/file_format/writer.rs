use std::path::Path;
use std::fs::File;
use std::io::{self, Write};
use std::mem;

use crate::file_format::metadata::{ColumnMetadata, FileMetadata, RowGroupMetadata};
use crate::table::{Column, Table};
use crate::codec::int64::encode_int64;
use crate::codec::varchar::encode_varchar_column;
use crate::codec::boolean::encode_boolean_column;
use crate::constants::{TYPE_INT64, TYPE_VARCHAR, TYPE_BOOLEAN, BATCH_SIZE};


fn write_file_metadata(
    file: &mut File, 
    columns: &[Column], 
    number_of_rows: usize
) -> io::Result<FileMetadata> {
    let number_of_cols: u32 = columns.len().try_into().map_err(|_| {
        io::Error::new(
        io::ErrorKind::InvalidInput,
        "Too many columns."
        )
    })?;

    let number_of_row_groups = number_of_rows.div_ceil(BATCH_SIZE);
    let file_metadata = FileMetadata {
        number_of_cols: number_of_cols,
        number_of_row_groups: number_of_row_groups as u32,
    };

    file_metadata.write_to(file)?;
    Ok(file_metadata)
}

fn write_row_group_metadata(
    file: &mut File,
    number_of_rows: usize,
) -> io::Result<()> {
    let row_group_metadata = RowGroupMetadata {
        number_of_rows: number_of_rows as u16,
    };

    row_group_metadata.write_to(file)?;
    Ok(())
}


fn write_column_chunk(
    file: &mut File,
    column: &Column,
    start: usize,
    end: usize,
) -> io::Result<()> {
    let (column_type, encoded, original_size) = match &column {
        Column::Int64(values) => {
            let encoded = encode_int64(&values[start..end]);
            let original_size = ((end - start) * mem::size_of::<i64>()) as u64;
            (TYPE_INT64, encoded, original_size)
        },
        Column::Varchar(values) => {
            let (encdoded, original_size) = encode_varchar_column(&values[start..end]);
            (TYPE_VARCHAR, encdoded, original_size)
        }
        Column::Boolean(values) => {
            let encoded = encode_boolean_column(&values[start..end]);
            let original_size = ((end - start) * mem::size_of::<bool>()) as u64;
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
    let mut file: File = File::create(path)?;

    let columns = &table.columns;
    let number_of_rows = columns.first().map_or(0, |column| column.len());
    if columns.iter().any(|column| column.len() != number_of_rows) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Every column have to have the same number of rows."
        ));
    };

    let file_metadata = write_file_metadata(&mut file, columns, number_of_rows)?;
    let number_of_row_groups = file_metadata.number_of_row_groups as usize;

    for group_index in 0..number_of_row_groups {
        let start = group_index * BATCH_SIZE;
        let end = (start + BATCH_SIZE).min(number_of_rows);
        let group_len = end - start;

        write_row_group_metadata(&mut file, group_len)?;
        for column in columns {
            write_column_chunk(&mut file, column, start, end)?;
        }

    }

    Ok(())
}
