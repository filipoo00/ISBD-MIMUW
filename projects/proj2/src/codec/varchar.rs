use lz4_flex::block::{compress, decompress};

fn compress_column(data: &[u8]) -> Vec<u8> {
    compress(data)
}

fn decompress_column(
    data: &[u8],
    original_size: usize,
) -> Result<Vec<u8>, lz4_flex::block::DecompressError> {
    decompress(data, original_size)
}

pub fn encode_varchar_column(values: &[String]) -> (Vec<u8>, u64) {
    let mut bytes = Vec::new();

    for value in values {
        let value_bytes = value.as_bytes();
        let value_bytes_len = value_bytes.len() as u32;

        bytes.extend_from_slice(&value_bytes_len.to_le_bytes());
        bytes.extend_from_slice(value_bytes);
    }

    let original_size = bytes.len() as u64;

    (compress_column(&bytes), original_size)
}

pub fn decode_varchar_column(data: &[u8], original_size: u64) -> Vec<String> {
    let mut values = Vec::new();
    let bytes = decompress_column(data, original_size as usize).unwrap();

    let mut pos = 0;
    while pos < bytes.len() {
        let len = 
            u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;

        pos += 4;

        let value = 
            String::from_utf8(bytes[pos..pos + len].try_into().unwrap()).unwrap();
        values.push(value);
        pos += len;
    }

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lz4_roundtrip() {
        let original = b"hello and bye";

        let compressed = compress_column(original);
        let decompressed =
            decompress_column(&compressed, original.len()).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn varchar_column_rountrip() {
        let value1 = "hello".to_string();
        let value2 = "bye".to_string();
        let values = vec![value1, value2];

        let (encoded, original_size) = encode_varchar_column(&values);
        let decoded = decode_varchar_column(&encoded, original_size);

        assert_eq!(decoded, values);

    }
}