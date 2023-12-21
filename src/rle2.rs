//! Run-length encoding: Stage 2
//!
//! The previous transform, move-to-front, tends to convert runs of the same symbol to be runs of
//! zeros. This tranform efficiently encodes runs of zeros by transforming them into sequences of
//! [`Symbol`]s.

/// The output of this transformation.
///
/// `RunA` and `RunB` are used to encode runs of zeros in similar fashion to binary numbers. To
/// convert a sequence of `RunA` and `RunB` symbols to a number of zeros, we use the equation
///
/// ZeroCnt + 1 == (1 << RunLen) | RunSyms
///
/// Bytes that are not zero are not transformed.
#[derive(Debug, PartialEq)]
pub(super) enum Symbol {
    /// Represents 1 times its position in the sequence.
    RunA,
    /// Represents 2 times its position in the sequence.
    RunB,
    /// Represents a non-zero byte. We store a `u8` in order to interoperate with other software
    /// which _may_ do the wrong thing.
    Byte(u8),
}

/// Encode the bytes into `Symbol`s.
pub(super) fn encode(mut data: &[u8]) -> Vec<Symbol> {
    let mut output = Vec::new();
    while !data.is_empty() {
        let (mut symbols, rest) = get_symbols(data);
        output.append(&mut symbols);
        data = rest;
    }
    output
}

/// Decode the `Symbol`s back to bytes.
pub(super) fn decode(mut data: &[Symbol]) -> Vec<u8> {
    let mut output = Vec::new();
    while !data.is_empty() {
        let (mut bytes, rest) = get_bytes(data);
        output.append(&mut bytes);
        data = rest;
    }
    output
}

/// Breaks the input into a leading `Symbol`s and the remaining bytes.
///
/// Example:
/// [0, 0, 0, 1] -> ([A, A], [1])
/// [1, 0, 0, 0] -> ([1], [0, 0, 0])
fn get_symbols(input: &[u8]) -> (Vec<Symbol>, &[u8]) {
    assert!(!input.is_empty());
    if input[0] != 0 {
        (vec![Symbol::Byte(input[0])], &input[1..])
    } else {
        let first_non_zero = input.iter().enumerate().find(|&(_, &byte)| byte != 0);
        match first_non_zero {
            // Encode how many zeros are in the current run and return the remaining bytes.
            Some((length, _)) => (encode_run(length), &input[length..]),
            // After this function returns, there is nothing else to encode.
            None => (encode_run(input.len()), &[]),
        }
    }
}

/// Breaks the input into leading bytes and remaining unprocessed `Symbol`s.
///
/// Example:
/// [A, A, 1] -> ([0, 0, 0], [1])
/// [1, A, A] -> ([1], [A, A])
fn get_bytes(input: &[Symbol]) -> (Vec<u8>, &[Symbol]) {
    assert!(!input.is_empty());
    if let Symbol::Byte(byte) = input[0] {
        (vec![byte], &input[1..])
    } else {
        let first_byte = input
            .iter()
            .enumerate()
            .find(|(_, symbol)| matches!(symbol, Symbol::Byte(_)));
        match first_byte {
            // Return the number of zeros that were encoded and the remaining `Symbol`s.
            Some((length, _)) => (decode_run(&input[..length]), &input[length..]),
            // After this function returns, there's nothing else to decode.
            None => (decode_run(input), &[]),
        }
    }
}

/// Represents an integer with a sequence of `Symbol::RunA` and `Symbol::RunB`.
fn encode_run(length: usize) -> Vec<Symbol> {
    assert!(length != 0);
    let mut output = Vec::new();
    let repr = length + 1;
    let num_symbols = repr.ilog2();
    let mut repr = (1 << num_symbols) ^ repr;
    for _ in 0..num_symbols {
        let rmb = 1 & repr;
        match rmb {
            0 => output.push(Symbol::RunA),
            1 => output.push(Symbol::RunB),
            _ => unreachable!(),
        }
        repr >>= 1;
    }
    output
}

/// Encodes a sequence of `Symbol::RunA` and `Symbol::RunB` as a run of zeros.
fn decode_run(run: &[Symbol]) -> Vec<u8> {
    assert!(!run.is_empty());
    let mut repr = 0;
    for symbol in run.iter().rev() {
        match symbol {
            // Put a 0 in the LSB to represent A.
            Symbol::RunA => repr <<= 1,
            // Put a 1 in the LSB to represent B.
            Symbol::RunB => repr = (repr << 1) | 1,
            _ => unreachable!(),
        }
    }
    let zero_count = ((1 << run.len()) | repr) - 1;
    vec![0; zero_count]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let data = [
            208, 27, 126, 113, 87, 4, 183, 227, 251, 11, 144, 165, 58, 129, 250, 46, 112, 3, 120,
            89, 221, 7, 172, 28, 129, 77, 68, 210, 134, 71, 179, 226, 70, 169, 167, 209, 78, 20,
            133, 177, 120, 141, 35, 198, 16, 248, 16, 34, 140, 73, 2, 122, 49, 145, 174, 44, 152,
            159, 166, 205, 137, 234, 238, 105, 230, 201, 15, 89, 5, 102, 107, 128, 109, 20, 209, 1,
            30, 38, 82, 87, 234, 168, 192, 235, 58, 161, 20, 88, 1, 4, 65, 195, 29, 158, 161, 218,
            138, 0, 174, 30,
        ];

        let encoded = encode(&data);
        let decoded = decode(&encoded);

        assert_eq!(data, &decoded[..]);
    }

    mod encode {
        use super::*;

        #[test]
        fn all_zeros() {
            let data = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

            let encoded = encode(&data);

            let expected = [Symbol::RunB, Symbol::RunA, Symbol::RunA, Symbol::RunA];
            assert_eq!(encoded, expected);
        }

        #[test]
        fn simple() {
            let data = [0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 0, 0];

            let encoded = encode(&data);

            let expected = [
                Symbol::RunA,
                Symbol::RunB,
                Symbol::Byte(1),
                Symbol::RunB,
                Symbol::Byte(2),
                Symbol::RunB,
                Symbol::RunA,
            ];
            assert_eq!(encoded, expected);
        }
    }

    mod decode {
        use super::*;

        #[test]
        fn simple() {
            let data = [
                Symbol::RunA,
                Symbol::RunB,
                Symbol::Byte(1),
                Symbol::RunB,
                Symbol::Byte(2),
                Symbol::RunB,
                Symbol::RunA,
            ];

            let decoded = decode(&data);

            let expected = [0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 0, 0];
            assert_eq!(decoded, expected);
        }
    }
}
