// File format - version 3
// FileMetadata is stored at the end of the file (footer).
// This allows the writer to write the file batch by batch
// without knowing the number of batches.
//
// File structure:
//
// For each batch:
//  BatchMetadata
//      number_of_rows:         number of rows in this batch
//
//  For each column:
//      ColumnMetadata 
//          column_type:        int64 / varchar / boolean
//          original_size:      size in bytes of the original column data
//          payload_size:       size in bytes of the encoded column data
//
// FileMetadata (footer)
//  number_of_cols:             number of columns in the file
//  number_of_batches:          number of batches in the file
//
// The reader first reads the FileMetadata (last 8 bytes).
// In this case the reader knows how many columns and batches there are in the file.


use std::fs::File;
use std::io::{self, Read, Write};

pub const FILE_METADATA_SIZE: usize = 8;


pub struct FileMetadata {
    pub number_of_cols: u32,
    pub number_of_batches: u32,
}

impl FileMetadata {
    pub fn write_to(&self, file: &mut File) -> io::Result<()> {
        file.write_all(&self.number_of_cols.to_le_bytes())?;
        file.write_all(&self.number_of_batches.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(file: &mut File) -> io::Result<Self> {
        let mut cols_buffer = [0u8; 4];
        let mut batches_buffer =  [0u8; 4];
        file.read_exact(&mut cols_buffer)?;
        file.read_exact(&mut batches_buffer)?;
        let number_of_cols = u32::from_le_bytes(cols_buffer);
        let number_of_batches = u32::from_le_bytes(batches_buffer);
        Ok( Self { number_of_cols, number_of_batches})
    }
}

pub struct BatchMetadata {
    pub number_of_rows: u16
}

impl BatchMetadata {
    pub fn write_to(&self, file: &mut File) -> io::Result<()> {
        file.write_all(&self.number_of_rows.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(file: &mut File) -> io::Result<Self> {
        let mut rows_buffer = [0u8; 2];
        file.read_exact(&mut rows_buffer)?;
        let number_of_rows = u16::from_le_bytes(rows_buffer);
        Ok( Self { number_of_rows })
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