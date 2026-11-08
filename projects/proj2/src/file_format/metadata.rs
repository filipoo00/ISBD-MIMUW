// File format - version 5
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
//          payload_size:       size in bytes of the encoded column data
//
// FileMetadata (footer)
//  number_of_batches:          number of batches in the file
//  number_of_cols:             number of columns in the file
//  column_types:               one byte per column (0 = int64, 1 = varchar, 2 = boolean)
//
// FileMetadataSize
//  footer_size:                size in bytes of the footer
//
// The reader first reads footer_size (last 4 bytes), seeks back by footer_size
// and reads FileMetadata, so it knows the schema and the number of batches
// before reading any batch.


use std::io::{self, Read, Write};

use crate::table::ColumnType;

pub const FOOTER_METADATA_SIZE: usize = 4;


fn column_type_to_u8(column_type: ColumnType) -> u8 {
    match column_type {
        ColumnType::Int64 => 0,
        ColumnType::Varchar => 1,
        ColumnType::Boolean => 2,
    }
}

fn column_type_from_u8(byte: u8) -> io::Result<ColumnType> {
    match byte {
        0 => Ok(ColumnType::Int64),
        1 => Ok(ColumnType::Varchar),
        2 => Ok(ColumnType::Boolean),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown column type"))
    }
}

pub struct FileMetadataSize {
    pub footer_size: u32,
}

impl FileMetadataSize {
    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.footer_size.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut footer_buffer = [0u8; 4];
        reader.read_exact(&mut footer_buffer)?;
        let footer_size = u32::from_le_bytes(footer_buffer);
        Ok( Self { footer_size })
    }
}


pub struct FileMetadata {
    pub number_of_batches: u32,
    pub column_types: Vec<ColumnType>
}

impl FileMetadata {
    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.number_of_batches.to_le_bytes())?;
        writer.write_all(&(self.column_types.len() as u32).to_le_bytes())?;
        for &column_type in &self.column_types {
            writer.write_all(&[column_type_to_u8(column_type)])?;
        }
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut batches_buffer = [0u8; 4];
        let mut cols_buffer = [0u8; 4];
        reader.read_exact(&mut batches_buffer)?;
        reader.read_exact(&mut cols_buffer)?;
        let number_of_batches = u32::from_le_bytes(batches_buffer);
        let number_of_cols = u32::from_le_bytes(cols_buffer);
        let mut column_types = Vec::new();
        let mut column_buffer = [0u8; 1];
        for _ in 0..number_of_cols {
            reader.read_exact(&mut column_buffer)?;
            column_types.push(column_type_from_u8(column_buffer[0])?);
        }
        Ok( Self { number_of_batches, column_types })
    }

    pub fn size(&self) -> u32 {
        4 + 4 + self.column_types.len() as u32
    }
}


pub struct BatchMetadata {
    pub number_of_rows: u16
}

impl BatchMetadata {
    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.number_of_rows.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut rows_buffer = [0u8; 2];
        reader.read_exact(&mut rows_buffer)?;
        let number_of_rows = u16::from_le_bytes(rows_buffer);
        Ok( Self { number_of_rows })
    }
}


pub struct ColumnMetadata {
    pub payload_size: u64,
}

impl ColumnMetadata {
    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.payload_size.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut payload_buffer = [0u8; 8];
        reader.read_exact(&mut payload_buffer)?;

        let payload_size = u64::from_le_bytes(payload_buffer);
        Ok( Self { payload_size })
    }
}