use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::fs::File;


use crate::file_format::metadata::{
    ColumnMetadata, FileMetadata, BatchMetadata, FILE_METADATA_SIZE
};
use crate::table::{Batch, Column};
use crate::codec::int64::decode_int64;
use crate::codec::varchar::decode_varchar_column;
use crate::codec::boolean::decode_boolean_column;
use crate::constants::{TYPE_INT64, TYPE_VARCHAR, TYPE_BOOLEAN};

pub struct Reader {
    file: File,
    file_metadata: FileMetadata,
    current_batch: usize,
}


impl Reader {
    fn read_column(
        &mut self,
        number_of_rows: u16,
    ) -> io::Result<Column> {
        let column_metadata = 
            ColumnMetadata::read_from(&mut self.file)?;

        let column_type = column_metadata.column_type;
        let payload_size = column_metadata.payload_size as usize;

        let mut buffer = vec![0u8; payload_size];

        self.file.read_exact(&mut buffer)?;

        let values = match column_type {
            TYPE_INT64 => {
                Column::Int64(
                    decode_int64(&buffer)
                )
            },
            TYPE_VARCHAR => {
                Column::Varchar(
                    decode_varchar_column(&buffer)
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


    pub fn open(path: &Path) -> io::Result<Self> {
        let mut file: File = File::open(path)?;
        
        // read the footer (last FILE_METADATA_SIZE bytes)
        file.seek(SeekFrom::End(-(FILE_METADATA_SIZE as i64)))?;
        
        let file_metadata = FileMetadata::read_from(&mut file)?;

        // go back to the beggining
        file.seek(SeekFrom::Start(0))?;

        Ok(Self {
            file: file, 
            file_metadata: file_metadata,
            current_batch: 0,
        })
    }

    pub fn next_batch(&mut self) -> io::Result<Option<Batch>> {
        if self.current_batch >= self.file_metadata.number_of_batches as usize{
            return Ok(None);
        }
        let batch_metadata = BatchMetadata::read_from(&mut self.file)?;
        let number_of_rows = batch_metadata.number_of_rows;
        let number_of_cols = self.file_metadata.number_of_cols;

        let mut columns = Vec::new();
        for _ in 0..number_of_cols {
            let column = self.read_column(number_of_rows)?;
            columns.push(column);
        }

        self.current_batch += 1;

        let batch = Batch::new(columns)?;
        Ok(Some(batch))
    }
}