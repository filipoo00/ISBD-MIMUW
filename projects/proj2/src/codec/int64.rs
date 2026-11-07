use std::io;

fn delta_encode(values: &[i64]) -> Vec<i64> {
    if values.is_empty() {
        return Vec::new()
    }

    let mut deltas = Vec::with_capacity(values.len());
    deltas.push(values[0]);
    for i in 1..values.len() {
        let delta = values[i].wrapping_sub(values[i-1]);
        deltas.push(delta)
    }

    deltas
}

fn delta_decode(deltas: &[i64]) -> Vec<i64> {
    if deltas.is_empty() {
        return Vec::new()
    }

    let mut values = Vec::with_capacity(deltas.len());
    values.push(deltas[0]);

    for i in 1..deltas.len() {
        let restored_value = deltas[i].wrapping_add(values[i-1]);
        values.push(restored_value)
    }

    values
}

fn zigzag_encode_value(value: i64) -> u64 {
    let mapped_value = ((value as u64) << 1) ^ ((value >> 63) as u64);
    mapped_value
}

fn zigzag_encode(deltas: &[i64]) -> Vec<u64> {
    let mut zigzags = Vec::with_capacity(deltas.len());

    for &delta in deltas {
        zigzags.push(zigzag_encode_value(delta))
    }

    zigzags
}

fn zigzag_decode_value(zigzag: u64) -> i64 {
    let decoded_value = zigzag >> 1;

    if zigzag & 1 == 0 {
        decoded_value as i64
    } else {
        (!decoded_value) as i64
    }
}

fn zigzag_decode(zigzags: &[u64]) -> Vec<i64> {
    let mut deltas = Vec::with_capacity(zigzags.len());

    for &zigzag in zigzags {
        deltas.push(zigzag_decode_value(zigzag))
    }

    deltas
}

fn vle_encode_value(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    loop {
        let mut byte = (value & 0b0111_1111) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0b1000_0000;
        }

        bytes.push(byte);

        if value == 0 {
            break
        }
    }

    bytes
}

fn vle_encode(zigzags: &[u64]) -> Vec<u8> {
    let mut bytes = Vec::new();

    for &value in zigzags {
        bytes.extend(vle_encode_value(value))
    }

    bytes
}

fn vle_decode(encoded: &[u8]) -> io::Result<Vec<u64>> {
    let mut decoded = Vec::new();

    let mut value = 0;
    let mut shift = 0;

    for byte in encoded {
        if shift >= u64::BITS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VLE value is too long.",
            ))
        }

        let restored_bits = (byte & 0b0111_1111) as u64;
        let continuation_bit = byte & 0b1000_0000;

        value |= restored_bits << shift;

        if continuation_bit == 0 {
            decoded.push(value);
            value = 0;
            shift = 0;
        } else {
            shift += 7;
        }
    }

    if shift != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VLE value is truncated.",
        ))
    }

    Ok(decoded)
}

pub fn encode_int64_column(values: &[i64]) -> Vec<u8> {
    let deltas = delta_encode(values);
    let zigzags = zigzag_encode(&deltas);
    vle_encode(&zigzags)
}

pub fn decode_int64_column(encoded: &[u8], number_of_rows: usize) -> io::Result<Vec<i64>> {
    let zigzags = vle_decode(encoded)?;

    if zigzags.len() != number_of_rows {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Int64 column has wrong number of values.",
        ))
    }

    let deltas = zigzag_decode(&zigzags);
    Ok(delta_decode(&deltas))
}


#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(values: &[i64]) {
        let encoded = encode_int64_column(values);
        let decoded = decode_int64_column(&encoded, values.len()).unwrap();

        assert_eq!(decoded, values);
    }

    #[test]
    fn delta_roundtrip() {
        let values = vec![1, -2, 100, 5000];

        let deltas = delta_encode(&values);
        let decoded = delta_decode(&deltas);

        assert_eq!(decoded, values);
    }

    #[test]
    fn zigzag_roundtrip() {
        let values = vec![0, 1, -1, 2, -2, i64::MAX, i64::MIN];

        for value in values {
            assert_eq!(zigzag_decode_value(zigzag_encode_value(value)), value);
        }
    }

    #[test]
    fn vle_roundtrip() {
        let values = vec![0, 1, 127, 128, 300, u64::MAX];

        let encoded = vle_encode(&values);
        let decoded = vle_decode(&encoded).unwrap();

        assert_eq!(decoded, values);
    }

    #[test]
    fn int64_roundtrip() {
        roundtrip(&[1, -2, 100, 5000]);
    }

    #[test]
    fn max_values_roundtrip() {
        roundtrip(&[i64::MIN, i64::MAX, 0, -1, i64::MAX, i64::MIN]);
    }

    #[test]
    fn empty_input_roundtrip() {
        roundtrip(&[]);
    }

    #[test]
    fn sequences_roundtrip() {
        let increasing: Vec<i64> = (0..1024).collect();
        let decreasing: Vec<i64> = (0..1024).rev().collect();

        roundtrip(&increasing);
        roundtrip(&decreasing);
    }

    #[test]
    fn wrong_number_of_rows_is_error() {
        let encoded = encode_int64_column(&[1, 2, 3]);

        assert!(decode_int64_column(&encoded, 4).is_err());
    }
}