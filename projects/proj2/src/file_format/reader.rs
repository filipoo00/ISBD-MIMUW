use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::fs::File;


use crate::file_format::metadata::{
    BatchMetadata, ColumnMetadata, FOOTER_METADATA_SIZE, FileMetadata, FileMetadataSize
};
use crate::table::{Batch, Column, ColumnType};
use crate::codec::int64::decode_int64_column;
use crate::codec::varchar::decode_varchar_column;
use crate::codec::boolean::decode_boolean_column;

pub struct Reader {
    file: File,
    file_metadata: FileMetadata,
    data_end: u64,
    current_batch: usize,
}


impl Reader {
    fn read_column(
        &mut self,
        column_type: ColumnType,
        number_of_rows: usize,
    ) -> io::Result<Column> {
        let column_metadata =
            ColumnMetadata::read_from(&mut self.file)?;

        let position = self.file.stream_position()?;
        if column_metadata.payload_size > self.data_end.saturating_sub(position) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Column payload exceeds file data.",
            ));
        }

        let payload_size = column_metadata.payload_size as usize;
        let mut buffer = vec![0u8; payload_size];

        self.file.read_exact(&mut buffer)?;

        let column = match column_type {
            ColumnType::Int64 => {
                Column::Int64(decode_int64_column(&buffer, number_of_rows)?)
            },
            ColumnType::Varchar => {
                Column::Varchar(decode_varchar_column(&buffer, number_of_rows)?)
            },
            ColumnType::Boolean => {
                Column::Boolean(decode_boolean_column(&buffer, number_of_rows)?)
            },
        };

        Ok(column)
    }


    pub fn open(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len();

        if file_size < FOOTER_METADATA_SIZE as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File is too small.",
            ));
        }

        // read the footer_size
        let footer_start = file_size - FOOTER_METADATA_SIZE as u64;
        file.seek(SeekFrom::Start(footer_start))?;
        let footer_size = FileMetadataSize::read_from(&mut file)?.footer_size;

        if u64::from(footer_size) > footer_start {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Footer size exceeds file size.",
            ));
        }

        // the footer (FileMetadata) is right before footer_size
        let data_end = footer_start - u64::from(footer_size);
        file.seek(SeekFrom::Start(data_end))?;
        let file_metadata = FileMetadata::read_from(&mut file)?;

        if file_metadata.size() != footer_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Footer size does not match footer content.",
            ));
        }

        // batches start at the beginning of the file
        file.seek(SeekFrom::Start(0))?;

        Ok(Self {
            file,
            file_metadata,
            data_end,
            current_batch: 0,
        })
    }

    pub fn next_batch(&mut self) -> io::Result<Option<Batch>> {
        if self.current_batch >= self.file_metadata.number_of_batches as usize {
            if self.file.stream_position()? != self.data_end {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Number of batches does not match file data.",
                ));
            }
            return Ok(None);
        }
        let batch_metadata = BatchMetadata::read_from(&mut self.file)?;
        let number_of_rows = batch_metadata.number_of_rows as usize;
        let number_of_cols = self.file_metadata.column_types.len();

        let mut columns = Vec::with_capacity(number_of_cols);
        for i in 0..number_of_cols {
            let column_type = self.file_metadata.column_types[i];
            columns.push(self.read_column(column_type, number_of_rows)?);
        }

        self.current_batch += 1;

        Ok(Some(Batch::new(columns)?))
    }

    pub fn schema(&self) -> &[ColumnType] {
        &self.file_metadata.column_types
    }
}