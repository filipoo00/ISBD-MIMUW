use std::cmp;

use crate::constants::BYTE_LEN;


pub fn encode_boolean_column(values: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::new();

    for chunk in values.chunks(BYTE_LEN) {
        let mut byte: u8 = 0;
        
        for (i, &value) in chunk.iter().enumerate() {
            if value {
                byte |= 1 << i
            }
        }
        bytes.push(byte);
    }

    bytes
}

pub fn decode_boolean_column(data: &[u8], number_of_values: u64) -> Vec<bool> {
    let mut values: Vec<bool> = Vec::new();
    let mut remaining = number_of_values;

    for byte in data {
        let how_many_values = cmp::min(remaining, BYTE_LEN as u64);
        for i in 0..how_many_values {
            let bit = byte & (1 << i);
            if bit != 0 {
                values.push(true);
            } else {
                values.push(false);
            }
        }
        remaining -= how_many_values;

        if remaining == 0 {
            break;
        }
    }

    values
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_column_roundtrip() {
        let original = vec![false, true, true];
        let encoded = encode_boolean_column(&original);
        let decoded = decode_boolean_column(&encoded, original.len() as u64);

        assert_eq!(decoded, original)
    }
}
