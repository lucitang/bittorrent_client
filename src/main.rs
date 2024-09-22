use serde_json;
use std::env;
// use serde::de::Unexpected::Option;
// Available if you need it!
// use serde_bencode;
use std::option::Option;

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let char = encoded_value.chars().next().unwrap();
    // Integers are encoded as i<number>e.
    if char == 'i' {
        // For example, 52 is encoded as i52e and -52 is encoded as i-52e.
        let end_index = encoded_value.find('e').unwrap();
        let string_value = &encoded_value[1..end_index];
        let value = string_value.parse::<i64>().unwrap();
        return serde_json::Value::Number(serde_json::Number::from(value));
    }
    // If encoded_value starts with a digit, it's a number
    if char.is_digit(10) {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        return serde_json::Value::String(string.to_string());
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}
enum ValueKind {
    Number,
    String,
    List,
    Dict,
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];

        let mut iterator = encoded_value.chars();
        let char = iterator.next().unwrap();
        // Lists are encoded as l<bencoded_elements>e.
        if char == 'l' {
            // For example, ["hello", 52] would be encoded as l5:helloi52ee.
            // Note that there are no separators between the elements
            let mut t = iterator.next();

            let mut s = String::new();
            let mut l = 0;
            let mut kind: Option<ValueKind> = None;
            let mut values = vec![];
            while t.is_some() {
                let mut c = t.unwrap();
                // Detect the end of the value
                match kind {
                    Some(ValueKind::Number) => {
                        if c == 'e' {
                            kind = None;
                            values.push(decode_bencoded_value(&s));
                            s = String::new();
                        } else {
                            s += stringify!(c);
                        }
                    }
                    Some(ValueKind::String) => {
                        if c == ':' {
                            kind = None;
                            s = String::new();
                        } else if l == 0 {
                            values.push(decode_bencoded_value(&s));
                            s = String::new();
                        } else {
                            s += stringify!(c);
                            l -= 1;
                        }
                    }
                    _ => {
                        if c == 'i' {
                            kind = Option::from(ValueKind::Number);
                        } else if c.is_digit(10) {
                            kind = Option::from(ValueKind::String);
                            l = c.to_digit(10).unwrap();
                        }

                        t = iterator.next();
                        continue;
                    }
                }

                t = iterator.next();
            }
            // should print [“hello”,52]
            return println!("{}", serde_json::Value::Array(values));
        }

        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
