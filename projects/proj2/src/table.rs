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


#[derive(Debug)]
pub enum Column {
    Int64(Vec<i64>),
    Varchar(Vec<String>),
    Boolean(Vec<bool>),
}

impl Column {
    pub fn len(&self) -> usize {
        match self {
            Column::Int64(values) => values.len(),
            Column::Varchar(values) => values.len(),
            Column::Boolean(values) => values.len(),
        }
    } 
}


pub struct Table {
    pub columns: Vec<Column>
}

pub struct Batch {
    pub columns: Vec<Column>
}