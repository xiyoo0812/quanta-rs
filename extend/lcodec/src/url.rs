#![allow(non_snake_case)]
#![allow(dead_code)]

fn to_hex(x: u8) -> u8 {
    if x > 9 { x + 55 } else { x + 48 }
}

fn from_hex(x: u8) -> u8 {
    match x {
        b'A'..=b'Z' => x - b'A' + 10,
        b'a'..=b'z' => x - b'a' + 10,
        b'0'..=b'9' => x - b'0',
        _ => x,
    }
}

pub fn url_encode(s: String) -> String {
    let mut temp = String::new();
    for byte in s.as_bytes() {
        match *byte {
            b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' | b'.' | b'~' => temp.push(*byte as char),
            b' ' => temp.push('+'),
            _ => {
                temp.push('%');
                temp.push(to_hex((*byte >> 4) as u8) as char);
                temp.push(to_hex((*byte & 0x0F) as u8) as char);
            },
        }
    }
    temp
}

pub fn url_decode(s: String) -> String {
    let mut temp = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => temp.push(' '),
            b'%' => {
                let high = from_hex(bytes[i + 1]);
                let low = from_hex(bytes[i + 2]);
                temp.push((high << 4 | low) as char);
                i += 2; // Skip the next two characters since we've processed them
            },
            val => temp.push(val as char),
        }
        i += 1;
    }
    temp
}