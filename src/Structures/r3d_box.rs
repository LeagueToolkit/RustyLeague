use crate::structures::vector3::Vector3;
use crate::io::binary_reader::BinaryReader;

pub struct R3DBox
{
    pub min: Vector3,
    pub max: Vector3
}

impl R3DBox
{
    pub fn new(min: Vector3, max: Vector3) -> Self
    {
        R3DBox
        {
            min,
            max
        }
    }

    pub fn read(file: &mut BinaryReader) -> Self
    {
        R3DBox
        {
            min: Vector3::read(file),
            max: Vector3::read(file)
        }

    }
}
