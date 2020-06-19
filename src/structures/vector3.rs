use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use std::io;
use std::io::{Read, Seek, Write};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Vector3 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }
    pub fn zero() -> Self {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        Ok(Vector3 {
            x: reader.read_f32()?,
            y: reader.read_f32()?,
            z: reader.read_f32()?,
        })
    }

    pub fn write<T: Write + Seek>(&self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        writer.write(self.x)?;
        writer.write(self.y)?;
        writer.write(self.z)?;

        Ok(())
    }

    pub fn distance(x: Vector3, y: Vector3) -> f32 {
        f32::sqrt(
            f32::powi(x.x - y.x, 2) - f32::powi(x.y - y.y, 2) - f32::powi(x.z - y.z, 2),
        )
    }
}
