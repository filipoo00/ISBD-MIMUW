use std::path::Path;
use std::fs::File;
use std::io::{self, Write};

use crate::file_format::metadata::{BatchMetadata, ColumnMetadata, FileMetadata, FileMetadataSize};
use crate::table::{Batch, Column, ColumnType};
use crate::codec::int64::encode_int64_column;
use crate::codec::varchar::encode_varchar_column;
use crate::codec::boolean::encode_boolean_column;


pub struct Writer {
    file: File,
    number_of_batches: u32,
    column_types: Vec<ColumnType>,
}

impl Writer {
    fn write_column(&mut self, column: &Column) -> io::Result<()> {
        let encoded = match column {
            Column::Int64(values) => {
                encode_int64_column(values)
            },
            Column::Varchar(varchar_column) => {
                encode_varchar_column(varchar_column)
            },
            Column::Boolean(values) => {
                encode_boolean_column(values)
            }
        };

        let metadata = ColumnMetadata {
            payload_size: encoded.len() as u64,
        };

        metadata.write_to(&mut self.file)?;
        self.file.write_all(&encoded)?;

        Ok(())
    }


    pub fn create(path: &Path, column_types: Vec<ColumnType>) -> io::Result<Self> {
        let file: File = File::create(path)?;

        Ok(Self {
            file,
            number_of_batches: 0,
            column_types,
        })
    }


    pub fn write_batch(&mut self, batch: &Batch) -> io::Result<()> {
        if !batch.matches_schema(&self.column_types) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Batch does not match the file schema.",
            ));
        }

        let batch_metadata = BatchMetadata {
            number_of_rows: batch.number_of_rows() as u16,
        };
        batch_metadata.write_to(&mut self.file)?;

        for column in batch.columns() {
            self.write_column(column)?;
        }

        self.number_of_batches += 1;

        Ok(())
    }


    pub fn finish(mut self) -> io::Result<()> {
        let file_metadata = FileMetadata {
            number_of_batches: self.number_of_batches,
            column_types: self.column_types,
        };
        let footer_size = file_metadata.size();

        file_metadata.write_to(&mut self.file)?;
        FileMetadataSize { footer_size }.write_to(&mut self.file)?;

        self.file.flush()?;

        Ok(())
    }
}