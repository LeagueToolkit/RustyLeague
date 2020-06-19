use super::vector3::Vector3;
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use std::io;
use std::io::{Read, Seek, Write};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
}

impl Sphere {
    pub const ZERO: Sphere = Sphere {
        center: Vector3::ZERO,
        radius: 0.0,
    };

    pub fn zero() -> Self {
        Sphere {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: 0.0,
        }
    }
    pub fn new(center: Vector3, radius: f32) -> Self {
        Sphere { center, radius }
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        Ok(Sphere {
            center: Vector3::read(reader)?,
            radius: reader.read_f32()?,
        })
    }

    pub fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        self.center.write(writer)?;
        writer.write(self.radius)?;

        Ok(())
    }
}
