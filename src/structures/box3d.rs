use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use crate::structures::vector3::Vector3;
use std::io;
use std::io::{Read, Seek, Write};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Box3D {
    pub min: Vector3,
    pub max: Vector3,
}

impl Box3D {
    pub const ZERO: Box3D = Box3D {
        min: Vector3::ZERO,
        max: Vector3::ZERO,
    };

    pub fn zero() -> Self {
        Box3D {
            min: Vector3::zero(),
            max: Vector3::zero(),
        }
    }
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Box3D { min, max }
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        Ok(Box3D {
            min: Vector3::read(reader)?,
            max: Vector3::read(reader)?,
        })
    }

    pub fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        self.min.write(writer)?;
        self.max.write(writer)?;

        Ok(())
    }
}
