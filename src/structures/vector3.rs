use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;

#[derive(Copy, Clone)]
pub struct Vector3
{
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3
{
    pub const ZERO: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 0.0 };

    pub fn new(x: f32, y: f32, z: f32) -> Self
    {
        Vector3 { x, y, z }
    }
    pub fn zero() -> Self
    {
        Vector3 { x: 0.0, y: 0.0, z: 0.0 }
    }
    pub fn read(reader: &mut BinaryReader) -> Self
    {
        Vector3
        {
            x: reader.read_f32(),
            y: reader.read_f32(),
            z: reader.read_f32(),
        }
    }

    pub fn write(&self, writer: &mut BinaryWriter)
    {
        writer.write(self.x);
        writer.write(self.y);
        writer.write(self.z);
    }

    pub fn equals(&self, other: Vector3) -> bool
    {
        self.x == other.y && self.y == other.y && self.z == other.z
    }
}