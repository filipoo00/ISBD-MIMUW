
fn delta_encode(values: &[i64]) -> Vec<i64> {
    if values.is_empty() {
        return Vec::new()
    }

    let mut delta_values = Vec::with_capacity(values.len());
    delta_values.push(values[0]);
    for i in 1..values.len() {
        let delta = values[i] - values[i-1];
        delta_values.push(delta)
    }

    delta_values
}

fn delta_decode(delta_values: &[i64]) -> Vec<i64> {
    if delta_values.is_empty() {
        return Vec::new()
    }

    let mut values = Vec::with_capacity(delta_values.len());
    values.push(delta_values[0]);

    for i in 1..delta_values.len() {
        let restored_value = delta_values[i] + values[i-1];
        values.push(restored_value)
    }

    values
}

fn zigzag_encode_val(value: i64) -> u64 {
    let mapped_value = ((value as u64) << 1) ^ ((value >> 63) as u64);
    mapped_value
}

fn zigzag_encode_vec(delta_values: &[i64]) -> Vec<u64> {
    let mut zigzag_enc = Vec::with_capacity(delta_values.len());

    for &value in delta_values {
        zigzag_enc.push(zigzag_encode_val(value))
    }

    zigzag_enc
}

fn zigzag_decode(zigzac_encode_val: u64) -> i64 {
    let decoded_val = zigzac_encode_val >> 1;

    if zigzac_encode_val & 1 == 0 {
        decoded_val as i64
    } else {
        (!decoded_val) as i64
    }
}

fn zigzag_decode_vec(zigzac_encode_vec: &[u64]) -> Vec<i64> {
    let mut delta_values = Vec::with_capacity(zigzac_encode_vec.len());

    for &encoded_val in zigzac_encode_vec {
        let decoded_val = zigzag_decode(encoded_val);
        delta_values.push(decoded_val)
    }

    delta_values
}

fn vle_encode_val(mut value: u64) -> Vec<u8> {
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

fn vle_encode_vec(values: &[u64]) -> Vec<u8> {
    let mut bytes = Vec::new();

    for &value in values {
        bytes.extend(vle_encode_val(value))
    }

    bytes
}

fn vle_decode_vec(encoded: &[u8]) -> Vec<u64> {
    let mut decoded = Vec::new();

    let mut value = 0;
    let mut shift = 0;

    for byte in encoded {
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

    decoded
}

pub fn encode_int64(values: &[i64]) -> Vec<u8> {
    let deltas = delta_encode(values);
    let zigzags = zigzag_encode_vec(&deltas);
    vle_encode_vec(&zigzags)
}

pub fn decode_int64(bytes: &[u8]) -> Vec<i64> {
    let zigzags = vle_decode_vec(bytes);
    let deltas = zigzag_decode_vec(&zigzags);
    delta_decode(&deltas)
}


#[cfg(test)]
mod tests {
    use super::*;

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
            assert_eq!(zigzag_decode(zigzag_encode_val(value)), value);
        }
    }

    #[test]
    fn vle_roundtrip() {
        let values = vec![0, 1, 127, 128, 300, u64::MAX];

        let encoded = vle_encode_vec(&values);
        let decoded = vle_decode_vec(&encoded);

        assert_eq!(decoded, values);
    }

    #[test]
    fn int64_roundtrip() {
        let values = vec![1, -2, 100, 5000];

        let encoded = encode_int64(&values);
        let decoded = decode_int64(&encoded);

        assert_eq!(decoded, values);
    }

    #[test]
    fn empty_input_roundtrip() {
        let values: Vec<i64> = vec![];

        let encoded = encode_int64(&values);
        let decoded = decode_int64(&encoded);

        assert_eq!(decoded, values);
    }
}