use std::cmp;
use std::io;

const BITS_PER_BYTE: usize = 8;


pub fn encode_boolean_column(values: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::new();

    for chunk in values.chunks(BITS_PER_BYTE) {
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

pub fn decode_boolean_column(encoded: &[u8], number_of_rows: usize) -> io::Result<Vec<bool>> {
    if encoded.len() != number_of_rows.div_ceil(BITS_PER_BYTE) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Boolean column has wrong number of bytes.",
        ))
    }


    let mut values: Vec<bool> = Vec::with_capacity(number_of_rows);
    let mut remaining = number_of_rows;

    for byte in encoded {
        let how_many_values = cmp::min(remaining, BITS_PER_BYTE);
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

    Ok(values)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(values: &[bool]) {
        let encoded = encode_boolean_column(values);
        let decoded = decode_boolean_column(&encoded, values.len()).unwrap();

        assert_eq!(values, decoded)
    }

    #[test]
    fn roundtrip_different_lengths() {
        for len in [0, 1, 7, 8, 9, 15, 16, 17, 1024] {
            let values: Vec<bool> = (0..len).map(|i| i % 3 == 0).collect();
            roundtrip(&values);
        }
    }

    #[test]
    fn roundtrip_all_the_same_values() {
        roundtrip(&[true; 5]);
        roundtrip(&[false; 5]);
    }

    #[test]
    fn encoded_bytes() {
        assert_eq!(encode_boolean_column(&[true, false, true]), vec![0b0000_0101]);
        assert_eq!(encode_boolean_column(&[false; 8]), vec![0b0000_0000]);
        assert_eq!(encode_boolean_column(&[true; 9]), vec![0b1111_1111, 0b0000_0001]);
    }

    #[test]
    fn uses_one_bit_per_value() {
        for len in [0, 1, 8, 9, 1024] {
            let encoded = encode_boolean_column(&vec![true; len]);
            assert_eq!(encoded.len(), len.div_ceil(BITS_PER_BYTE));
        }
    }

    #[test]
    fn takes_exact_number_of_values() {
        let encoded = vec![0b1111_1111];
        assert_eq!(decode_boolean_column(&encoded, 3).unwrap(), vec![true, true, true]);
    }

    #[test]
    fn wrong_number_of_bytes_is_error() {
        assert!(decode_boolean_column(&[0b1111_1111, 0b0000_0001], 17).is_err());
        assert!(decode_boolean_column(&[0b0000_0101, 0b0000_0000], 3).is_err());
    }
}
