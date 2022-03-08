extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

enum ZWC {
    ZeroWidthNonJoiner,
    ZeroWidthJoiner,
    WordJoiner,
    InvisibleTimes,
    InvisibleSeparator,
    // ファイルのエンコーディングを示すBOMとしても利用されているので非推奨
    // ZERO_WIDTH_NON_BREAK,

    // iOS, IEで表示される
    // InvisiblePlus
}

impl ZWC {
    fn value(&self) -> char {
        match *self {
            ZWC::ZeroWidthNonJoiner => '\u{200C}',
            ZWC::ZeroWidthJoiner => '\u{200D}',
            ZWC::WordJoiner => '\u{2060}',
            ZWC::InvisibleTimes => '\u{2062}',
            ZWC::InvisibleSeparator => '\u{2063}',
            // ZWC::ZERO_WIDTH_NON_BREAK => '\u{FEFE}',
            // ZWC::InvisiblePlus => '\u{2064}',
        }
    }
}

fn encode_table() -> [char; 4] {
    [
        ZWC::ZeroWidthNonJoiner.value(),
        ZWC::ZeroWidthJoiner.value(),
        ZWC::WordJoiner.value(),
        ZWC::InvisibleTimes.value(),
    ]
}

/// 入力された文字をゼロ幅文字を用いて2ビットエンコードします。
#[wasm_bindgen]
pub fn encode(str: &str) -> String {
    let bytes = str.as_bytes();
    let encode_table = encode_table();
    bytes
        .iter()
        .map(|b| convert_to_zero_width(*b, &encode_table))
        .collect()
}

fn convert_to_zero_width(byte: u8, encode_table: &[char]) -> String {
    let zero_widths: String = (0..4)
        .map(|i| (byte >> 2 * i) & 0b00000011)
        .map(|bit| match bit {
            0 => encode_table[0],
            1 => encode_table[1],
            2 => encode_table[2],
            3 => encode_table[3],
            _ => panic!("Invalid binary digit: {}", bit),
        })
        .rev()
        .collect();
    zero_widths
}

/// ゼロ幅文字で2ビットエンコードされた文字列をUTF8でデコードします｡
///
/// エンコードされた文字列が不正な場合はエラーを返します。
/// # Example
/// ```
///  let input = "\u{200d}\u{200c}\u{2060}\u{200c}\u{200d}\u{2060}\u{200d}\u{200d}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{2062}\u{2062}\u{200c}\u{2060}\u{200c}\u{200c}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{2060}\u{2062}\u{2062}\u{200d}\u{2062}\u{200c}\u{2060}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{200d}\u{200c}\u{200c}\u{2060}\u{200c}\u{200d}";
///  let expect = "Hello World!";
///  assert_eq!(expect, decode(&input).unwrap());
/// ```
#[wasm_bindgen(catch)]
pub fn decode(zero_width_code: &str) -> Result<String, String> {
    let encode_table = encode_table();
    if !validate_before_decode(zero_width_code, &encode_table) {
        return Err(format!("Invalid zero-width code: {}", zero_width_code));
    }

    let char_per_byte = 4;
    let count = zero_width_code.chars().count();
    let bytes_count = count / char_per_byte;
    // 1バイトずつ処理
    let bytes = (0..bytes_count)
        .map(|i| {
            let start = i * char_per_byte;
            let byte_words = zero_width_code
                .chars()
                .skip(start)
                .take(char_per_byte)
                .collect::<String>();
            let decoded = convert_from_zero_width(&byte_words, &encode_table);
            decoded
        })
        .collect::<Vec<u8>>();

    // UTF8でデコード
    let decoded = String::from_utf8(bytes);
    match decoded {
        Ok(decoded) => Ok(decoded),
        Err(e) => Err(format!("Invalid UTF8: {}", e)),
    }
}

fn validate_before_decode(zero_width_code: &str, encode_table: &[char]) -> bool {
    zero_width_code.chars().all(|c| encode_table.contains(&c))
}

fn convert_from_zero_width(string: &str, encode_table: &[char]) -> u8 {
    string
        .chars()
        .map(|c| match c {
            c if c == encode_table[0] => 0,
            c if c == encode_table[1] => 1,
            c if c == encode_table[2] => 2,
            c if c == encode_table[3] => 3,
            _ => panic!("Invalid zero width character: {}", c),
        })
        .fold(0, |acc, x| acc << 2 | x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let input = "Hello World!";
        let expect = "\u{200d}\u{200c}\u{2060}\u{200c}\u{200d}\u{2060}\u{200d}\u{200d}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{2062}\u{2062}\u{200c}\u{2060}\u{200c}\u{200c}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{2060}\u{2062}\u{2062}\u{200d}\u{2062}\u{200c}\u{2060}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{200d}\u{200c}\u{200c}\u{2060}\u{200c}\u{200d}";
        assert_eq!(expect, encode(&input));
    }

    #[test]
    fn test_decode() {
        let input = "\u{200d}\u{200c}\u{2060}\u{200c}\u{200d}\u{2060}\u{200d}\u{200d}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{2062}\u{2062}\u{200c}\u{2060}\u{200c}\u{200c}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{2060}\u{2062}\u{2062}\u{200d}\u{2062}\u{200c}\u{2060}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{200d}\u{200c}\u{200c}\u{2060}\u{200c}\u{200d}";
        let expect = "Hello World!";
        assert_eq!(decode(&input).unwrap(), expect);
    }

    #[test]
    fn test_decode_invalid_input_01() {
        let invalid_input = "Hello World!";
        let expect = "Invalid zero-width code: Hello World!";
        assert_eq!(expect, decode(&invalid_input).unwrap_err());
    }

    #[test]
    fn test_decode_invalid_input() {
        let invalid_input = "\u{200d}\u{200c}\u{200c}\u{200d}\u{2060}\u{200d}\u{200d}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{2062}\u{2062}\u{200c}\u{2060}\u{200c}\u{200c}\u{200d}\u{200d}\u{200d}\u{2062}\u{200d}\u{2060}\u{2062}\u{2062}\u{200d}\u{2062}\u{200c}\u{2060}\u{200d}\u{2060}\u{2062}\u{200c}\u{200d}\u{2060}\u{200d}\u{200c}\u{200c}\u{2060}\u{200c}\u{200d}";
        let expect = "Invalid UTF8:";
        assert!(decode(invalid_input).unwrap_err().contains(&expect))
    }

    #[test]
    fn test_validate_before_decode() {
        let encode_table = encode_table();
        let input = format!(
            "{}{}{}{}",
            encode_table[0], encode_table[1], encode_table[2], encode_table[3]
        );
        assert!(validate_before_decode(&input, &encode_table));
        let invalid_input = "hello world";
        assert!(!validate_before_decode(invalid_input, &encode_table));
    }

    #[test]
    fn test_convert_from_zero_width() {
        let encode_table = encode_table();
        let input: String = [
            encode_table[0],
            encode_table[0],
            encode_table[0],
            encode_table[0],
        ]
        .iter()
        .collect();
        let expect: u8 = 0;
        assert_eq!(convert_from_zero_width(&input, &encode_table), expect);

        let input: String = [
            encode_table[2],
            encode_table[2],
            encode_table[2],
            encode_table[2],
        ]
        .iter()
        .collect();
        let expect: u8 = 170;
        assert_eq!(convert_from_zero_width(&input, &encode_table), expect);

        let input: String = [
            encode_table[3],
            encode_table[3],
            encode_table[3],
            encode_table[3],
        ]
        .iter()
        .collect();
        let expect: u8 = 255;
        assert_eq!(convert_from_zero_width(&input, &encode_table), expect);
    }

    #[test]
    fn test_convert_to_zero_width() {
        let input: u8 = 0;

        let encode_table = encode_table();
        let expect: String = [
            encode_table[0],
            encode_table[0],
            encode_table[0],
            encode_table[0],
        ]
        .iter()
        .collect();
        assert_eq!(expect, convert_to_zero_width(input, &encode_table));

        let input: u8 = 4;
        let expect: String = [
            encode_table[0],
            encode_table[0],
            encode_table[1],
            encode_table[0],
        ]
        .iter()
        .collect();
        assert_eq!(expect, convert_to_zero_width(input, &encode_table));

        let input: u8 = 170;
        let expect: String = [
            encode_table[2],
            encode_table[2],
            encode_table[2],
            encode_table[2],
        ]
        .iter()
        .collect();
        assert_eq!(expect, convert_to_zero_width(input, &encode_table));

        let input: u8 = 255;
        let expect: String = [
            encode_table[3],
            encode_table[3],
            encode_table[3],
            encode_table[3],
        ]
        .iter()
        .collect();
        assert_eq!(expect, convert_to_zero_width(input, &encode_table));
    }
}
