use super::vector3::Vector3;
use crate::io::binary_reader::BinaryReader;

pub struct R3DSphere
{
    pub center: Vector3,
    pub radius: f32
}

impl R3DSphere
{
    pub fn read(file: &mut BinaryReader) -> Self
    {
        R3DSphere
        {
            center: Vector3::read(file),
            radius: file.read_f32()
        }
    }
}

