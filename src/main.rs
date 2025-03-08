// Binary parser

use std::iter::Peekable;

enum DataType {
    Integer(u8),
    Float(u8),
    String(u32),
}

#[derive(Debug)]
enum ParsedValue {
    Integer8(u8),
    Integer16(u16),
    Integer32(u32),
    Float32(f32),
    String(String),
}

struct BinaryParser<'a> {
    data: &'a [u8],
    offset: usize,
    parsed_data: Vec<ParsedValue>
}

impl <'a> BinaryParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0, parsed_data: Vec::new() }
    }

    fn display(&self) {
        let mut string = String::new();
        for value in &self.parsed_data {
            match value {
                ParsedValue::Integer8(val) => {
                    println!("{}", val);
                    string.insert_str(string.len(), val.to_owned().to_string().as_str());
                },
                ParsedValue::Integer16(val) => {
                    println!("{}", val);
                    string.insert_str(string.len(), val.to_owned().to_string().as_str());
                },
                ParsedValue::Integer32(val) => {
                    println!("{}", val);
                    string.insert_str(string.len(), val.to_owned().to_string().as_str());
                },
                ParsedValue::Float32(val) => {
                    println!("{}", val);
                    string.insert_str(string.len(), val.to_owned().to_string().as_str());
                },
                ParsedValue::String(val) => {
                    println!("{}", val);
                    string.push_str(val);
                },
            }
        }
        println!("{}", string);
    }

    fn parse_integer(&mut self, size: u8) -> ParsedValue {
        match size {
            1 => {
                let data = u8::from_le_bytes(self.data[self.offset..self.offset+1].try_into().unwrap());
                self.offset += 1;
                ParsedValue::Integer8(data)
            },
            2 => {
                let data: u16 = u16::from_le_bytes(self.data[self.offset..self.offset+2].try_into().unwrap());
                self.offset += 2;
                ParsedValue::Integer16(data)
            },
            4 => {
                let data: u32 = u32::from_le_bytes(self.data[self.offset..self.offset+4].try_into().unwrap());
                self.offset += 4;
                ParsedValue::Integer32(data)
            },
            _ => {
                ParsedValue::Integer8(0)
            }
        }
    }

    fn parse_float(&mut self) -> ParsedValue {
        let data: f32 = f32::from_le_bytes(self.data[self.offset..self.offset+4].try_into().unwrap());
        self.offset += 4;
        ParsedValue::Float32(data)
    }

    fn parse_string(&mut self, length: u32) -> ParsedValue {
        let mut string = String::new();
        for _ in 0..length {
            let data = self.data[self.offset];
            string.push(data as char);
            self.offset += 1;
        }
        ParsedValue::String(string)
    }

    fn parse(&mut self){
        let mut stream: Peekable<std::slice::Iter<'_, u8>> = self.data.iter().peekable();
        while let Some(&byte) = stream.peek() {
            match byte {
                0x01 => {
                    stream.next();
                    self.offset += 1;
                    match self.read(DataType::Integer(8), &mut stream) {
                        Some(value) => {
                            self.parsed_data.push(value);
                        },
                        None => {
                            println!("Error: Corrupted data");
                        }
                    }
                },
                0x02 => {
                    stream.next();
                    self.offset += 1;
                    match self.read(DataType::Integer(16), &mut stream) {
                        Some(value) => {
                            self.parsed_data.push(value);
                        },
                        None => {
                            println!("Error: Corrupted data");
                        }
                    }
                },
                0x03 => {
                    println!("u32 byte{}", byte);
                    stream.next();
                    self.offset += 1;
                    match self.read(DataType::Integer(32), &mut stream) {
                        Some(value) => {
                            self.parsed_data.push(value);
                        },
                        None => {
                            println!("Error: Corrupted data");
                        }
                    }
                },
                0x04 => {
                    stream.next();
                    self.offset += 1;
                    match self.read(DataType::Float(32), &mut stream) {
                        Some(value) => {
                            self.parsed_data.push(value);
                        },
                        None => {
                            println!("Error: Corrupted data");
                        }
                    }
                },
                0x05 => {
                    stream.next();
                    self.offset += 1;
                    let mut str_len: u32 = 0;
                    while let Some(byte) = stream.peek() {
                        match byte {
                            &ch if ch.is_ascii_graphic() || ch.is_ascii_whitespace()  => {
                                println!("String byte: {}", ch);
                                match self.read(DataType::String(str_len), &mut stream){
                                    Some(value) => {
                                        self.parsed_data.push(value);
                                    },
                                    None => {
                                        println!("Error: Corrupted data");
                                    }
                                }
                                break;
                            },

                            &ch if *ch >= 0x01 && *ch <= 0xFF => {
                                let bytes = [*ch, 0, 0, 0];
                                str_len += u32::from_le_bytes(bytes);
                                stream.next();
                                self.offset += 1;
                            }
                            _ => {
                                println!("Error: Unknown data type");
                                break;
                            }
                        }   
                    }
                },
                _ => {
                    println!("Error: Unknown data type");
                    break;
                }
                
            }
        }
    }

    fn read(&mut self, byte_type: DataType, stream: &mut Peekable<std::slice::Iter<'_, u8>>) -> Option<ParsedValue> {
        match byte_type {
            DataType::Integer(bit_length) => {
                match bit_length {
                    8 => {
                        let val = self.parse_integer(1);
                        stream.next();
                        Some(val)
                    }
                    16 => {
                        let val = self.parse_integer(2);
                        for _ in 0..=1 {
                            stream.next();
                        }
                        Some(val)
                    }
                    32 => {
                        let val = self.parse_integer(4);
                        for _ in 0..=3 {
                            stream.next();
                        }
                        Some(val)
                    }
                    _ => {
                        return None;
                    }
                }
            },
            DataType::Float(bit_length) => {
                match bit_length {
                    32 => {
                        let val = self.parse_float();
                        for _ in 0..=3 {
                            stream.next();
                        }
                        Some(val)
                    }
                    64 => {
                        let val = self.parse_float();
                        for _ in 0..=7 {
                            stream.next();
                        }
                        Some(val)
                    }
                    _ => {
                        return None;
                    }
                }
            },
            DataType::String(length) => {
                let val = self.parse_string(length);
                for _ in 0..length {
                    stream.next();
                }
                Some(val)
            },
        }
    }
}

fn main(){
    let binary_data: &[u8] = &[
        0x01, 0x7F,                      // u8: 127
        0x02, 0x34, 0x12,                // u16: 4660
        0x03, 0x78, 0x56, 0x34, 0x12,    // u32: 0x12345678
        0x05, 0x0B, b'H', b'e', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd', b'!',    // String: "Hello world!"
        0x04, 0xCD, 0xCC, 0x8C, 0x3F,    // f32: 1.1
    ];

    let mut parser = BinaryParser::new(binary_data);
    parser.parse();
    parser.display();
}