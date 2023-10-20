//! Run length encoding.
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub(crate) enum Error {
    #[error("The run length encoded array was truncated")]
    RunLengthTruncated,
}

/// Convert `data` into a run-length encoded byte array.
pub(super) fn encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    // This is safe because we checked for data being empty above.
    let mut run_start = 0;
    let mut output = vec![];

    for (i, byte) in data.iter().enumerate() {
        let run_length: u8 = (i - run_start) as u8;
        if *byte != data[run_start] || run_length == 255 {
            encode_run(&data[run_start..i], &mut output);
            run_start = i;
        }
    }

    encode_run(&data[run_start..], &mut output);

    output
}

/// De-convert `data` from a run-length encoded byte array to a byte array.
pub(super) fn decode(mut data: &[u8]) -> Result<Vec<u8>, Error> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let mut output = Vec::new();

    while !data.is_empty() {
        let run = get_run(data)?;
        data = &data[run.len()..];
        decode_run(run, &mut output);
    }

    Ok(output)
}

fn decode_run(data: &[u8], output: &mut Vec<u8>) {
    debug_assert!(
        [1, 2, 3, 5].contains(&data.len()),
        "data is an invalid length: {}",
        data.len()
    );

    if data.len() < 4 {
        output.extend_from_slice(data);
    } else {
        for _ in 0..data[data.len() - 1] + 4 {
            output.push(data[0]);
        }
    }
}

fn encode_run(data: &[u8], output: &mut Vec<u8>) {
    debug_assert!(
        data.iter().skip(1).all(|n| *n == data[0]),
        "Items in data should all be the same"
    );
    debug_assert!(data.len() <= 255, "Data cannot be longer than 255 bytes.");

    if data.len() <= 3 {
        output.extend_from_slice(data);
    } else {
        output.extend_from_slice(&data[..4]);
        output.push((data.len() - 4) as u8);
    }
}

fn get_run(data: &[u8]) -> Result<&[u8], Error> {
    let length = std::cmp::min(data.len() - 1, 3);

    for (i, byte) in data[..=length].iter().enumerate().skip(1) {
        if *byte != data[0] && i != 4 {
            return Ok(&data[..i]);
        }
    }

    if data.len() == 4 {
        Err(Error::RunLengthTruncated)
    } else {
        let length = std::cmp::min(data.len(), 5);
        Ok(&data[..length])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test the encode function.
    mod encode {
        use super::*;

        /// Test with an empty slice.
        #[test]
        fn empty() {
            let data = b"";

            let encoded = encode(data);

            assert_eq!(encoded, data);
        }

        /// Test with no repeats 4 or longer.
        #[test]
        fn no_four() {
            let data = b"abbccc";

            let encoded = encode(data);

            assert_eq!(encoded, data);
        }

        /// Test with runs longer than 255.
        #[test]
        fn big_bytes_four_too_many() {
            let data = [b'e'; 259];

            let encoded = encode(&data);

            let expected = b"eeee\xfbeeee\0";
            assert_eq!(encoded, expected);
        }

        /// Test with runs longer than 255.
        #[test]
        fn big_bytes_one_too_many() {
            let data = [b'e'; 256];

            let encoded = encode(&data);

            let expected = b"eeee\xfbe";
            assert_eq!(encoded, expected);
        }
    }

    /// Test the encode_run() function.
    mod encode_run {
        use super::*;

        #[test]
        fn big_bytes() {
            let data = [b'e'; 255];
            let mut output = Vec::new();

            encode_run(&data, &mut output);

            let expected = b"eeee\xfb";
            assert_eq!(output, expected);
        }

        #[test]
        fn five_bytes() {
            let data = b"ddddd";
            let mut output = Vec::new();

            encode_run(data, &mut output);

            let expected = b"dddd\x01";
            assert_eq!(output, expected);
        }

        #[test]
        fn four_bytes() {
            let data = b"dddd";
            let mut output = Vec::new();

            encode_run(data, &mut output);

            let expected = b"dddd\0";
            assert_eq!(output, expected);
        }

        #[test]
        fn three_bytes() {
            let data = b"ccc";
            let mut output = Vec::new();

            encode_run(data, &mut output);

            assert_eq!(output, data);
        }

        #[test]
        fn two_bytes() {
            let data = b"bb";
            let mut output = Vec::new();

            encode_run(data, &mut output);

            assert_eq!(output, data);
        }
    }

    /// Test the decode function
    mod decode {
        use super::*;

        /// Test with runs longer than 255.
        #[test]
        fn big_bytes_four_too_many() {
            let data = b"eeee\xfbeeee\0";

            let encoded = decode(data).expect("data should decode");

            let expected = [b'e'; 259];
            assert_eq!(encoded, expected);
        }

        /// Test with runs longer than 255.
        #[test]
        fn big_bytes_one_too_many() {
            let data = b"eeee\xfbe";

            let encoded = decode(data).expect("data should decode");

            let expected = [b'e'; 256];
            assert_eq!(encoded, expected);
        }

        /// Test with an empty slice.
        #[test]
        fn empty() {
            let data = b"";

            let encoded = decode(data).expect("data should decode");

            assert_eq!(encoded, data);
        }

        /// Invalid input should return an error.
        #[test]
        fn invalid_input() {
            // You cannot have a run of 4 with no length after it.
            let data = b"abbcccc";

            match decode(data) {
                Ok(_) => panic!("This should have resulted in an error"),
                Err(err) => match err {
                    Error::RunLengthTruncated => {}
                },
            }
        }

        /// Test with no repeats 4 or longer.
        #[test]
        fn no_four() {
            let data = b"abbccc";

            let encoded = decode(data).expect("data should decode");

            assert_eq!(encoded, data);
        }

        /// Test with a small run at the beginning
        #[test]
        fn small_run_beginning() {
            let data = b"eeee\x01a";

            let encoded = decode(data).expect("data should decode");

            let expected = b"eeeeea";
            assert_eq!(encoded, expected);
        }

        /// Test with a small run
        #[test]
        fn small_run_end() {
            let data = b"aeeee\x01";

            let encoded = decode(data).expect("data should decode");

            let expected = b"aeeeee";
            assert_eq!(encoded, expected);
        }

        /// Test with a small run
        #[test]
        fn small_run_middle() {
            let data = b"aeeee\x01bb";

            let encoded = decode(data).expect("data should decode");

            let expected = b"aeeeeebb";
            assert_eq!(encoded, expected);
        }
    }
}
