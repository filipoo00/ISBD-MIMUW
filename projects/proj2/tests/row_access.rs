use proj2::table::{Batch, Column, VarcharColumn};


fn varchar_column(values: &[&str]) -> VarcharColumn {
    let mut column = VarcharColumn::new();
    for value in values {
        column.push(value);
    }
    column
}

fn sample_batch() -> Batch {
    Batch::new(vec![
        Column::Int64(vec![1, 2, 3]),
        Column::Boolean(vec![true, false, true]),
        Column::Varchar(varchar_column(&["a", "b", "c"])),
        Column::Varchar(varchar_column(&["a", "b", "c"])),
    ]).unwrap()
}


#[test]
fn read_single_row() {
    let batch = sample_batch();
    let row = 1;

    let id = batch.column(0).unwrap().as_int64().unwrap()[row];
    let flag = batch.column(1).unwrap().as_boolean().unwrap()[row];
    let s1 = batch.column(2).unwrap().as_varchar().unwrap().get(row);
    let s2 = batch.column(3).unwrap().as_varchar().unwrap().get(row);

    assert_eq!(id, 2);
    assert_eq!(flag, false);
    assert_eq!(s1, "b");
    assert_eq!(s2, "b");
}

#[test]
fn concat_two_varchar_columns() {
    let mut batch = sample_batch();

    let s1 = batch.column(2).unwrap().as_varchar().unwrap();
    let s2 = batch.column(3).unwrap().as_varchar().unwrap();

    let mut s3 = VarcharColumn::new();
    for row in 0..batch.number_of_rows() {
        s3.push(&format!("{}{}", s1.get(row), s2.get(row)));
    }
    batch.add_column(Column::Varchar(s3)).unwrap();

    let result = batch.column(4).unwrap().as_varchar().unwrap();
    assert_eq!(batch.number_of_cols(), 5);
    assert_eq!(result.get(0), "aa");
    assert_eq!(result.get(1), "bb");
    assert_eq!(result.get(2), "cc");
}

#[test]
fn wrong_type_or_index_is_none() {
    let batch = sample_batch();

    assert!(batch.column(0).unwrap().as_varchar().is_none());
    assert!(batch.column(2).unwrap().as_int64().is_none());
    assert!(batch.column(4).is_none());
}

#[test]
fn add_column_with_wrong_length_is_error() {
    let mut batch = sample_batch();

    assert!(batch.add_column(Column::Int64(vec![1, 2])).is_err());
    assert_eq!(batch.number_of_cols(), 4);
}
