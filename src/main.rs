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
        println!("encoded_value {}", encoded_value);
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

        let mut iterator = encoded_value.chars();
        // Lists are encoded as l<bencoded_elements>e.
        if iterator.next().unwrap() == 'l' {
            // For example, ["hello", 52] would be encoded as l5:helloi52ee.
            // Note that there are no separators between the elements
            let mut t = iterator.next();

            let mut s = String::new();
            let mut l: i32 = 0;
            let mut kind: Option<ValueKind> = None;
            let mut values = vec![];
            while t.is_some() {
                let c = t.unwrap();
                // println!("Current char '{}'", c);
                // println!("Current length '{}'", l);
                // Detect the end of the value
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
                        if c == ':' {
                            l = s.parse::<i32>().unwrap() - 1;
                            s.push(c);
                            // println!("Main String {}", s);
                            // println!("Length {}", l);
                        } else if l == 0 {
                            s.push(c);
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
                        s = String::from(c);
                        if c == 'i' {
                            // println!("Detected Number {}", s);
                            kind = Option::from(ValueKind::Number);
                        } else if c.is_digit(10) {
                            // println!("Detected String {}", s);
                            kind = Option::from(ValueKind::String);
                        }
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
