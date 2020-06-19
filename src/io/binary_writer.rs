use std::fs::{File, OpenOptions};
use std::io;
use std::io::SeekFrom;
use std::io::Write;
use std::io::{BufWriter, Cursor, Seek};
use std::path::Path;

pub struct BinaryWriter<T: Write = File> {
    writer: BufWriter<T>,
}

impl BinaryWriter<File> {
    pub fn from_location(file_location: &Path) -> Self {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_location);

        BinaryWriter {
            writer: BufWriter::new(file.unwrap()),
        }
    }

    pub fn from_file(file: File) -> Self {
        BinaryWriter {
            writer: BufWriter::new(file),
        }
    }
}

impl BinaryWriter<Cursor<Vec<u8>>> {
    pub fn from_buffer(buffer: Cursor<Vec<u8>>) -> Self {
        BinaryWriter {
            writer: BufWriter::new(buffer),
        }
    }
}

impl<T: Write + Seek> BinaryWriter<T> {
    pub fn write<W: BinaryWriterWriteable>(&mut self, to_write: W) -> io::Result<usize> {
        to_write.write(self)
    }

    pub fn write_char(&mut self, to_write: char) -> io::Result<usize> {
        self.writer.write(&(to_write as u8).to_le_bytes())
    }
    pub fn write_i8(&mut self, to_write: i8) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_u8(&mut self, to_write: u8) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_i16(&mut self, to_write: i16) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_u16(&mut self, to_write: u16) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_i32(&mut self, to_write: i32) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_u32(&mut self, to_write: u32) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_i64(&mut self, to_write: i64) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_u64(&mut self, to_write: u64) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_f32(&mut self, to_write: f32) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }
    pub fn write_f64(&mut self, to_write: f64) -> io::Result<usize> {
        self.writer.write(&to_write.to_le_bytes())
    }

    pub fn write_bytes(&mut self, to_write: Vec<u8>) -> io::Result<usize> {
        self.writer.write(to_write.as_slice())
    }
    pub fn write_string(&mut self, to_write: &str) -> io::Result<usize> {
        self.writer.write(to_write.as_bytes())
    }
    pub fn write_padded_string(&mut self, to_write: &str, length: usize) -> io::Result<usize> {
        let pad_count = length - to_write.len();
        let mut pad_buffer: Vec<u8> = Vec::with_capacity(pad_count);
        for i in 0..pad_count {
            pad_buffer.push(0);
        }

        self.write_string(to_write)?;
        self.write_bytes(pad_buffer)
    }
    pub fn write_sized_string(&mut self, to_write: &str) -> io::Result<usize> {
        self.write_u32(to_write.len() as u32)?;
        self.write_string(to_write)
    }
    pub fn write_null_terminated_string(&mut self, to_write: &str) -> io::Result<usize> {
        self.write_string(to_write)?;
        self.write_u8(0)
    }

    pub fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.writer.seek(position)
    }
    pub fn position(&mut self) -> u64 {
        return self.writer.seek(SeekFrom::Current(0)).unwrap();
    }
}

pub trait BinaryWriterWriteable {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize>;
}
impl BinaryWriterWriteable for char {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_char(*self)
    }
}
impl BinaryWriterWriteable for i8 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_i8(*self)
    }
}
impl BinaryWriterWriteable for u8 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_u8(*self)
    }
}
impl BinaryWriterWriteable for i16 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_i16(*self)
    }
}
impl BinaryWriterWriteable for u16 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_u16(*self)
    }
}
impl BinaryWriterWriteable for i32 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_i32(*self)
    }
}
impl BinaryWriterWriteable for u32 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_u32(*self)
    }
}
impl BinaryWriterWriteable for i64 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_i64(*self)
    }
}
impl BinaryWriterWriteable for u64 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_u64(*self)
    }
}
impl BinaryWriterWriteable for f32 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_f32(*self)
    }
}
impl BinaryWriterWriteable for f64 {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_f64(*self)
    }
}
impl BinaryWriterWriteable for &[u8] {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_bytes(self.to_vec())
    }
}
impl BinaryWriterWriteable for Vec<u8> {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_bytes(self.to_vec())
    }
}
impl BinaryWriterWriteable for String {
    fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<usize> {
        writer.write_string(self)
    }
}
