use std::io::{self, Read};
use std::path::Path;
use std::fs::File;


use crate::file_format::metadata::{ColumnMetadata, FileMetadata, RowGroupMetadata};
use crate::table::{Column, Table};
use crate::codec::int64::decode_int64;
use crate::codec::varchar::decode_varchar_column;
use crate::codec::boolean::decode_boolean_column;
use crate::constants::{TYPE_INT64, TYPE_VARCHAR, TYPE_BOOLEAN};


fn read_column(
    file: &mut File,
    number_of_rows: u16,
    column_metadata: &ColumnMetadata,
) -> io::Result<Column> {
    let column_type = column_metadata.column_type;
    let original_size = column_metadata.original_size;
    let payload_size = column_metadata.payload_size as usize;
    let mut buffer = vec![0u8; payload_size];

    file.read_exact(&mut buffer)?;

    let values = match column_type {
        TYPE_INT64 => {
            Column::Int64(
                decode_int64(&buffer)
            )
        },
        TYPE_VARCHAR => {
            Column::Varchar(
                decode_varchar_column(&buffer, original_size)   
            )
        },
        TYPE_BOOLEAN => {
            Column::Boolean(
                decode_boolean_column(&buffer, number_of_rows)
            )
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown column type",
            ));
        }
    };

    Ok(values)
}


pub fn read_file(path: &Path) -> io::Result<Table> {
    let mut file = File::open(path)?;
    let file_metadata = FileMetadata::read_from(&mut file)?;
    let number_of_cols = file_metadata.number_of_cols;
    let number_of_row_groups = file_metadata.number_of_row_groups;

    let mut columns = Vec::new();

    for group_index in 0..number_of_row_groups {
        let row_group_metadata = RowGroupMetadata::read_from(&mut file)?;
        let number_of_rows = row_group_metadata.number_of_rows;

        for column_index in 0..number_of_cols {
            let column_metadata = 
                ColumnMetadata::read_from(&mut file).unwrap();

            let column = read_column(&mut file, number_of_rows, &column_metadata).unwrap();
            if group_index == 0 {
                columns.push(column);
            } else {
                columns[column_index as usize].extend(column);
            }
        }

        println!("{:?}", columns);
    }

    Ok(Table { columns })
}