use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use std::io::{Read, Seek, Write};
use std::io;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vector4
{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

impl Vector4
{
    pub const ZERO: Vector4 = Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self
    {
        Vector4 { x, y, z, w }
    }
    pub fn zero() -> Self
    {
        Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self>
    {
        Ok(Vector4
        {
            x: reader.read_f32()?,
            y: reader.read_f32()?,
            z: reader.read_f32()?,
            w: reader.read_f32()?
        })
    }

    pub fn write<T: Write + Seek>(&self, writer: &mut BinaryWriter<T>) -> io::Result<()>
    {
        writer.write(self.x)?;
        writer.write(self.y)?;
        writer.write(self.z)?;
        writer.write(self.w)?;

        Ok(())
    }
}