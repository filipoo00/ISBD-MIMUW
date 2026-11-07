use lz4_flex::block::{compress, decompress};

use crate::table::VarcharColumn;
use crate::codec::int64::{encode_int64, decode_int64};

const OFFSET_SIZE_LEN: usize = 4;

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

    let encoded_offsets = encode_int64(&offsets);
    let offsets_size = u32::try_from(encoded_offsets.len())
        .expect("encoded offsets exceed 4 GB");

    let encoded_data = compress_data(column.data().as_bytes());

    let mut encoded = Vec::with_capacity(
        OFFSET_SIZE_LEN + encoded_offsets.len() + encoded_data.len()
    );
    encoded.extend_from_slice(&offsets_size.to_le_bytes());
    encoded.extend_from_slice(&encoded_offsets);
    encoded.extend_from_slice(&encoded_data);

    encoded
}

pub fn decode_varchar_column(data: &[u8]) -> VarcharColumn {
    let (size_bytes, rest) = data.split_at(OFFSET_SIZE_LEN);
    let offsets_size = u32::from_le_bytes(size_bytes.try_into().unwrap()) as usize;
    let (encoded_offsets, encoded_data) = rest.split_at(offsets_size);

    let offsets: Vec<u32> = decode_int64(encoded_offsets)
        .into_iter()
        .map(|o| o as u32)
        .collect();

    let uncompressed_size = *offsets.last().unwrap() as usize;
    let bytes = decompress_data(encoded_data, uncompressed_size).unwrap();
    let text = String::from_utf8(bytes).unwrap();

    VarcharColumn::from_parts(text, offsets)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lz4_roundtrip() {
        let original = b"hello and bye";

        let compressed = compress_data(original);
        let decompressed =
            decompress_data(&compressed, original.len()).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn varchar_column_rountrip() {
        let mut column = VarcharColumn::new();
        column.push("hello");
        column.push("bye");

        let encoded = encode_varchar_column(&column);
        let decoded = decode_varchar_column(&encoded);

        assert_eq!(decoded, column);
    }
}
