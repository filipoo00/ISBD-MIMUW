use std::fs::File;
use std::io::{self, Read, Write};

// rozkminka
// FileMetadata -> at the top of the file:
//  number_of_cols -> will not change, so we can have this info at the top
//  number_of_groups_row -> how many groups we will have, to be able to iterate through all the groups

// RowGroupMetadata -> after the FileMetadata and before every RowGroup
//  number_of_rows -> number_of_rows in this group

// ColumnMetadata -> each column metadata in each RowGroup
//  column_type -> type of the column
//  original_size -> size in bytes of the original array
//  payload_size -> size in bytes of the written array


pub struct FileMetadata {
    pub number_of_cols: u32,
    pub number_of_row_groups: u32,
}

impl FileMetadata {
    pub fn write_to(&self, file: &mut File) -> io::Result<()> {
        file.write_all(&self.number_of_cols.to_le_bytes())?;
        file.write_all(&self.number_of_row_groups.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(file: &mut File) -> io::Result<Self> {
        let mut cols_buffer = [0u8; 4];
        let mut row_groups_buffer =  [0u8; 4];
        file.read_exact(&mut cols_buffer)?;
        file.read_exact(&mut row_groups_buffer)?;
        let number_of_cols = u32::from_le_bytes(cols_buffer);
        let number_of_row_groups = u32::from_le_bytes(row_groups_buffer);
        Ok( Self { number_of_cols, number_of_row_groups})
    }
}

pub struct RowGroupMetadata {
    pub number_of_rows: u16
}

impl RowGroupMetadata {
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