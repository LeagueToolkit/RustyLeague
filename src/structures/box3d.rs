use crate::structures::vector3::Vector3;
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;

#[derive(Copy, Clone, PartialEq)]
pub struct Box3D
{
    pub min: Vector3,
    pub max: Vector3
}

impl Box3D
{
    pub const ZERO: Box3D = Box3D { min: Vector3::ZERO, max: Vector3::ZERO };

    pub fn zero() -> Self
    {
        Box3D
        {
            min: Vector3::zero(),
            max: Vector3::zero()
        }
    }
    pub fn new(min: Vector3, max: Vector3) -> Self
    {
        Box3D
        {
            min,
            max
        }
    }
    pub fn read(reader: &mut BinaryReader) -> Self
    {
        Box3D
        {
            min: Vector3::read(reader),
            max: Vector3::read(reader)
        }
    }

    pub fn write(&self, writer: &mut BinaryWriter)
    {
        self.min.write(writer);
        self.max.write(writer);
    }
}