use std::io::{BufReader, Seek, Cursor};
use std::io::Read;
use std::fs::File;
use std::io::SeekFrom;
use std::vec;

pub struct BinaryReader<T: Read = File>
{
    reader : BufReader<T>
}

impl BinaryReader<Cursor<Vec<u8>>>
{
    pub fn from_buffer(buffer: Cursor<Vec<u8>>) -> Self
    {
        BinaryReader{ reader: BufReader::new(buffer) }
    }
}
impl BinaryReader<File>
{
    pub fn from_location(file_location: &str) -> Self
    {
        let file = File::open(file_location).unwrap();

        BinaryReader{ reader: BufReader::new(file) }
    }
    pub fn from_file(file: File) -> Self
    {
        BinaryReader{ reader: BufReader::new(file) }
    }
}

impl<T: Read + Seek> BinaryReader<T>
{
    pub fn read_char(&mut self) -> char
    {
        return self.read_u8() as char;
    }
    pub fn read_i8(&mut self) -> i8
    {
        let mut buffer = [0; 1];

        self.reader.read_exact(&mut buffer);

        return i8::from_le_bytes(buffer);
    }
    pub fn read_u8(&mut self) -> u8
    {
        let mut buffer = [0; 1];

        self.reader.read_exact(&mut buffer);

        return u8::from_le_bytes(buffer);
    }
    pub fn read_i16(&mut self) -> i16
    {
        let mut buffer = [0; 2];

        self.reader.read_exact(&mut buffer);

        return i16::from_le_bytes(buffer);
    }
    pub fn read_u16(&mut self) -> u16
    {
        let mut buffer = [0; 2];

        self.reader.read_exact(&mut buffer);

        return u16::from_le_bytes(buffer);
    }
    pub fn read_i32(&mut self) -> i32
    {
        let mut buffer = [0; 4];

        self.reader.read_exact(&mut buffer);

        return i32::from_le_bytes(buffer);
    }
    pub fn read_u32(&mut self) -> u32
    {
        let mut buffer = [0; 4];

        self.reader.read_exact(&mut buffer);

        return u32::from_le_bytes(buffer);
    }
    pub fn read_i64(&mut self) -> i64
    {
        let mut buffer = [0; 8];

        self.reader.read_exact(&mut buffer);

        return i64::from_le_bytes(buffer);
    }
    pub fn read_u64(&mut self) -> u64
    {
        let mut buffer = [0; 8];

        self.reader.read_exact(&mut buffer);

        return u64::from_le_bytes(buffer);
    }
    pub fn read_f32(&mut self) -> f32
    {
        let mut buffer = [0; 4];

        self.reader.read_exact(&mut buffer);

        return f32::from_le_bytes(buffer);
    }
    pub fn read_f64(&mut self) -> f64
    {
        let mut buffer = [0; 8];

        self.reader.read_exact(&mut buffer);

        return f64::from_le_bytes(buffer);
    }

    pub fn read_bytes(&mut self, size: usize) -> Vec<u8>
    {
        let mut buffer = vec![0; size];

        self.reader.read_exact(&mut buffer);

        return buffer;
    }
    pub fn read_string(&mut self, length: usize) -> String
    {
        let buffer = self.read_bytes(length);

        return String::from_utf8(buffer).unwrap();
    }
    pub fn read_sized_string(&mut self) -> String
    {
        let length = self.read_u32() as usize;

        return self.read_string(length);
    }
    pub fn read_padded_string(&mut self, length: usize) -> String
    {
        let string = self.read_string(length);
        string[0..string.find('\0').unwrap()].to_string()
    }
    pub fn read_null_terminated_string(&mut self) -> String
    {
        let mut string = String::new();

        let mut c: char;
        loop
        {
            c = self.read_char();

            match c
            {
                '\0' => break,
                _ => { string.push(c) }
            }
        }

        return string;
    }

    pub fn seek(&mut self, position: SeekFrom)
    {
        self.reader.seek(position);
    }
}