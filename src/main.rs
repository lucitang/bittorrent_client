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
        // println!("encoded_value {}", encoded_value);
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
    // List,
    // Dict,
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

        let mut iterator = encoded_value.chars().peekable();
        // Lists are encoded as l<bencoded_elements>e.
        if iterator.next().unwrap() == 'l' {
            // For example, ["hello", 52] would be encoded as l5:helloi52ee.
            // Note that there are no separators between the elements
            let mut s = String::new();
            let mut l: i32 = 0;
            let mut kind: Option<ValueKind> = None;
            let mut values = vec![];
            while iterator.peek().is_some() {
                let c = iterator.next().unwrap();
                match kind {
                    Some(ValueKind::Number) => {
                        s.push(c);
                        if c == 'e' {
                            kind = None;
                            // println!("Total Number {}", s);
                            values.push(decode_bencoded_value(&s));
                            s = String::new();
                        }
                    }
                    Some(ValueKind::String) => {
                        if l == 0 {
                            s.push(c);
                            // println!("Done. Final string: {}", s);
                            values.push(decode_bencoded_value(&s));
                            // println!("Done. Final string: {}", s);
                            s = String::new();
                            kind = None;
                        } else {
                            s.push(c);
                            l -= 1;
                        }
                    }
                    // Detect the start of the value
                    _ => {
                        if c == 'i' {
                            s = String::from(c);
                            // println!("Detected Number {}", s);
                            kind = Option::from(ValueKind::Number);
                        } else if c.is_digit(10) {
                            if s.is_empty() {
                                s = String::from(c);
                            } else {
                                s.push(c);
                            }
                            // println!("Detected String {}", s);
                        } else if c == ':' {
                            l = s.parse::<i32>().unwrap() - 1;
                            s.push(c);
                            kind = Option::from(ValueKind::String);
                            // println!("Main String {}", s);
                            // println!("Length {}", l);
                        }
                    }
                }
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
