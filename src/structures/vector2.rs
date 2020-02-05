use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use crate::structures::vector3::Vector3;
use std::io::{Read, Seek, Write};

#[derive(Copy, Clone, PartialEq)]
pub struct Vector2
{
    pub x: f32,
    pub y: f32
}

impl Vector2
{
    pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self
    {
        Vector2 { x, y }
    }
    pub fn zero() -> Self
    {
        Vector2 { x: 0.0, y: 0.0 }
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> Self
    {
        Vector2
        {
            x: reader.read_f32(),
            y: reader.read_f32()
        }
    }

    pub fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>)
    {
        writer.write(self.x);
        writer.write(self.y);
    }
}