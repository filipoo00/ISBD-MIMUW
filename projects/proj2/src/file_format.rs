use std::fs::File;
use std::io::{self, Read, Write};
use std::mem;

use crate::batch::Column;
use crate::codec::boolean::{decode_boolean_column, encode_boolean_column};
use crate::codec::int64::{decode_int64, encode_int64};
use crate::codec::varchar::{decode_varchar_column, encode_varchar_column};
use crate::constants::{TYPE_INT64, TYPE_VARCHAR, TYPE_BOOLEAN};


pub struct FileMetadata {
    pub number_of_cols: u32,
    pub number_of_rows: u64,
}

impl FileMetadata {
    pub fn write_to(&self, file: &mut File) -> io::Result<()> {
        file.write_all(&self.number_of_cols.to_le_bytes())?;
        file.write_all(&self.number_of_rows.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(file: &mut File) -> io::Result<Self> {
        let mut cols_buffer = [0u8; 4];
        let mut rows_buffer =  [0u8; 8];
        file.read_exact(&mut cols_buffer)?;
        file.read_exact(&mut rows_buffer)?;
        let number_of_cols = u32::from_le_bytes(cols_buffer);
        let number_of_rows = u64::from_le_bytes(rows_buffer);
        Ok( Self { number_of_cols, number_of_rows})
    }
}


pub struct ColumnMetadata {
    pub column_type: u8,
    pub original_size: u64,
    pub payload_size: u64,
}

impl ColumnMetadata {
    pub fn write_to(&self, file: &mut File) -> io::Result<()> {
        file.write_all(&self.column_type.to_le_bytes())?;
        file.write_all(&self.original_size.to_le_bytes())?;
        file.write_all(&self.payload_size.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(file: &mut File) -> io::Result<Self> {
        let mut column_type_buffer = [0u8; 1];
        let mut original_buffer = [0u8; 8];
        let mut payload_buffer = [0u8; 8];
        file.read_exact(&mut column_type_buffer)?;
        file.read_exact(&mut original_buffer)?;
        file.read_exact(&mut payload_buffer)?;

        let column_type = u8::from_le_bytes(column_type_buffer);
        let original_size = u64::from_le_bytes(original_buffer);
        let payload_size = u64::from_le_bytes(payload_buffer);
        Ok( Self { column_type, original_size, payload_size })
    }
}


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


pub fn read_column(
    file: &mut File,
    number_of_rows: u64,
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
