use lz4_flex::block::{compress, decompress};
use std::io;

use crate::table::VarcharColumn;
use crate::codec::int64::{encode_int64_column, decode_int64_column};

const OFFSETS_SIZE_LEN: usize = 4;

fn compress_data(data: &[u8]) -> Vec<u8> {
    compress(data)
}

fn decompress_data(
    compressed: &[u8],
    uncompressed_size: usize,
) -> Result<Vec<u8>, lz4_flex::block::DecompressError> {
    decompress(compressed, uncompressed_size)
}

pub fn encode_varchar_column(column: &VarcharColumn) -> Vec<u8> {
    let offsets: Vec<i64> = column.offsets().iter().map(|&o| i64::from(o)).collect();

    let encoded_offsets = encode_int64_column(&offsets);
    let offsets_size = u32::try_from(encoded_offsets.len())
        .expect("encoded offsets exceed 4 GB");

    let encoded_data = compress_data(column.data().as_bytes());

    let mut encoded = Vec::with_capacity(
        OFFSETS_SIZE_LEN + encoded_offsets.len() + encoded_data.len()
    );
    encoded.extend_from_slice(&offsets_size.to_le_bytes());
    encoded.extend_from_slice(&encoded_offsets);
    encoded.extend_from_slice(&encoded_data);

    encoded
}

pub fn decode_varchar_column(encoded: &[u8], number_of_rows: usize) -> io::Result<VarcharColumn> {
    if encoded.len() < OFFSETS_SIZE_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Varchar column has wrong number of bytes.",
        ))
    }

    let (size_bytes, rest) = encoded.split_at(OFFSETS_SIZE_LEN);
    let offsets_size = u32::from_le_bytes(size_bytes.try_into().unwrap()) as usize;
    if rest.len() < offsets_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Varchar column has wrong number of bytes.",
        ))
    }

    let (encoded_offsets, encoded_data) = rest.split_at(offsets_size);

    let mut offsets: Vec<u32> = Vec::with_capacity(number_of_rows + 1);
    for o in decode_int64_column(encoded_offsets, number_of_rows + 1)? {
        let offset = u32::try_from(o).map_err(|_| io::Error::new(
            io::ErrorKind::InvalidData,
            "Varchar offset out of range.",
        ))?;
        offsets.push(offset);
    }

    let uncompressed_size = *offsets.last().unwrap() as usize;
    let bytes = decompress_data(encoded_data, uncompressed_size).map_err(|_| io::Error::new(
        io::ErrorKind::InvalidData,
        "Varchar data decompression failed.",
    ))?;
    let text = String::from_utf8(bytes).map_err(|_| io::Error::new(
        io::ErrorKind::InvalidData,
        "Varchar data is not valid UTF-8.",
    ))?;

    VarcharColumn::from_parts(text, offsets)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn build_column(values: &[&str]) -> VarcharColumn {
        let mut column = VarcharColumn::new();
        for value in values {
            column.push(value);
        }
        column
    }

    fn roundtrip(values: &[&str]) {
        let column = build_column(values);

        let encoded = encode_varchar_column(&column);
        let decoded = decode_varchar_column(&encoded, values.len()).unwrap();

        assert_eq!(decoded, column);
    }

    #[test]
    fn lz4_roundtrip() {
        let original = b"hello and bye";

        let compressed = compress_data(original);
        let decompressed =
            decompress_data(&compressed, original.len()).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn varchar_column_roundtrip() {
        roundtrip(&["hello", "bye"]);
    }

    #[test]
    fn empty_column_roundtrip() {
        roundtrip(&[]);
    }

    #[test]
    fn empty_strings_roundtrip() {
        roundtrip(&["", "a", ""]);
    }

    #[test]
    fn unicode_roundtrip() {
        roundtrip(&["ąąąąlol", "Ęęc"]);
    }

    #[test]
    fn long_string_roundtrip() {
        let long = "a".repeat(100000);
        roundtrip(&[&long, "short"]);
    }

    #[test]
    fn wrong_number_of_rows_is_error() {
        let encoded = encode_varchar_column(&build_column(&["a", "b"]));

        assert!(decode_varchar_column(&encoded, 3).is_err());
    }

    #[test]
    fn truncated_payload_is_error() {
        let encoded = encode_varchar_column(&build_column(&["hello", "bye"]));

        assert!(decode_varchar_column(&encoded[..2], 2).is_err());
        assert!(decode_varchar_column(&encoded[..encoded.len() - 1], 2).is_err());
    }
}
