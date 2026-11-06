use std::path::Path;
use std::fs::File;
use std::io::{self, Write};
use std::mem;

use crate::file_format::metadata::{ColumnMetadata, FileMetadata, BatchMetadata};
use crate::table::{Batch, Column};
use crate::codec::int64::encode_int64;
use crate::codec::varchar::encode_varchar_column;
use crate::codec::boolean::encode_boolean_column;
use crate::constants::{TYPE_INT64, TYPE_VARCHAR, TYPE_BOOLEAN};


pub struct Writer {
    file: File,
    number_of_cols: u32,
    number_of_batches: u32,
}

impl Writer {
    fn write_column(&mut self, column: &Column) -> io::Result<()> {
        let (column_type, encoded, original_size) = match &column {
            Column::Int64(values) => {
                let encoded = encode_int64(&values);
                let original_size = (column.len() * mem::size_of::<i64>()) as u64;
                (TYPE_INT64, encoded, original_size)
            },
            Column::Varchar(values) => {
                let (encdoded, original_size) = encode_varchar_column(&values);
                (TYPE_VARCHAR, encdoded, original_size)
            }
            Column::Boolean(values) => {
                let encoded = encode_boolean_column(&values);
                let original_size = (column.len() * mem::size_of::<bool>()) as u64;
                (TYPE_BOOLEAN, encoded, original_size)
            }
        };

        let metdata = ColumnMetadata {
            column_type: column_type,
            original_size: original_size,
            payload_size: encoded.len() as u64,
        };

        metdata.write_to(&mut self.file)?;
        self.file.write_all(&encoded)?;

        Ok(())
    }


    pub fn create(path: &Path, number_of_cols: usize) -> io::Result<Self> {
        let file: File = File::create(path)?;

        let number_of_cols = number_of_cols as u32;

        Ok(Self { 
            file: file,
            number_of_cols: number_of_cols,
            number_of_batches: 0,
        })
    }


    pub fn write_batch(&mut self, batch: &Batch) -> io::Result<()> {
        if batch.number_of_cols() != self.number_of_cols as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Batch has a different number of columns.",
            ));
        }
        let batch_metadata = BatchMetadata {
            number_of_rows: batch.number_of_rows() as u16,
        };
        batch_metadata.write_to(&mut self.file)?;


        let columns = batch.columns();
        for column in columns {
            self.write_column(column)?;
        }

        self.number_of_batches += 1;

        Ok(())
    }


    pub fn finish(&mut self) -> io::Result<()> {
        let file_metadata = FileMetadata {
            number_of_cols: self.number_of_cols,
            number_of_batches: self.number_of_batches,
        };

        file_metadata.write_to(&mut self.file)?;
        self.file.flush()?;

        Ok(())
    }
}