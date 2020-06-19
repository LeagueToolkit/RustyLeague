use std::fs::File;
use std::io::Read;
use std::io::SeekFrom;
use std::io::{BufReader, Cursor, ErrorKind, Seek};
use std::path::Path;
use std::{io, vec};

pub struct BinaryReader<T: Read = File> {
    reader: BufReader<T>,
}

impl BinaryReader<Cursor<Vec<u8>>> {
    pub fn from_buffer(buffer: Cursor<Vec<u8>>) -> Self {
        BinaryReader {
            reader: BufReader::new(buffer),
        }
    }
}
impl BinaryReader<File> {
    pub fn from_location(file_location: &Path) -> Self {
        let file = File::open(file_location).unwrap();

        BinaryReader {
            reader: BufReader::new(file),
        }
    }
    pub fn from_file(file: File) -> Self {
        BinaryReader {
            reader: BufReader::new(file),
        }
    }
}

impl<T: Read + Seek> BinaryReader<T> {
    pub fn read_char(&mut self) -> io::Result<char> {
        match self.read_u8() {
            Ok(x) => Ok(x as char),
            Err(error) => Err(error),
        }
    }
    pub fn read_i8(&mut self) -> io::Result<i8> {
        let mut buffer = [0; 1];

        self.reader.read_exact(&mut buffer)?;

        Ok(i8::from_le_bytes(buffer))
    }
    pub fn read_u8(&mut self) -> io::Result<u8> {
        let mut buffer = [0; 1];

        self.reader.read_exact(&mut buffer)?;

        Ok(u8::from_le_bytes(buffer))
    }
    pub fn read_i16(&mut self) -> io::Result<i16> {
        let mut buffer = [0; 2];

        self.reader.read_exact(&mut buffer)?;

        Ok(i16::from_le_bytes(buffer))
    }
    pub fn read_u16(&mut self) -> io::Result<u16> {
        let mut buffer = [0; 2];

        self.reader.read_exact(&mut buffer)?;

        Ok(u16::from_le_bytes(buffer))
    }
    pub fn read_i32(&mut self) -> io::Result<i32> {
        let mut buffer = [0; 4];

        self.reader.read_exact(&mut buffer)?;

        Ok(i32::from_le_bytes(buffer))
    }
    pub fn read_u32(&mut self) -> io::Result<u32> {
        let mut buffer = [0; 4];

        self.reader.read_exact(&mut buffer)?;

        Ok(u32::from_le_bytes(buffer))
    }
    pub fn read_i64(&mut self) -> io::Result<i64> {
        let mut buffer = [0; 8];

        self.reader.read_exact(&mut buffer)?;

        Ok(i64::from_le_bytes(buffer))
    }
    pub fn read_u64(&mut self) -> io::Result<u64> {
        let mut buffer = [0; 8];

        self.reader.read_exact(&mut buffer)?;

        Ok(u64::from_le_bytes(buffer))
    }
    pub fn read_f32(&mut self) -> io::Result<f32> {
        let mut buffer = [0; 4];

        self.reader.read_exact(&mut buffer)?;

        Ok(f32::from_le_bytes(buffer))
    }
    pub fn read_f64(&mut self) -> io::Result<f64> {
        let mut buffer = [0; 8];

        self.reader.read_exact(&mut buffer)?;

        Ok(f64::from_le_bytes(buffer))
    }

    pub fn read_bytes(&mut self, size: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; size];

        self.reader.read_exact(&mut buffer)?;

        Ok(buffer)
    }
    pub fn read_string(&mut self, length: usize) -> io::Result<String> {
        let buffer = self.read_bytes(length)?;

        match String::from_utf8(buffer) {
            Ok(x) => Ok(x),
            Err(error) => Err(io::Error::new(ErrorKind::InvalidData, error.to_string())),
        }
    }
    pub fn read_sized_string(&mut self) -> io::Result<String> {
        let length = self.read_u32()? as usize;

        self.read_string(length)
    }
    pub fn read_padded_string(&mut self, length: usize) -> io::Result<String> {
        let string = self.read_string(length)?;
        Ok(string[0..string.find('\0').unwrap()].to_string())
    }
    pub fn read_null_terminated_string(&mut self) -> io::Result<String> {
        let mut string = String::new();

        let mut c: char;
        loop {
            c = self.read_char()?;

            match c {
                '\0' => break,
                _ => string.push(c),
            }
        }

        Ok(string)
    }

    pub fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.reader.seek(position)
    }
    pub fn position(&mut self) -> u64 {
        return self.reader.seek(SeekFrom::Current(0)).unwrap();
    }
}
