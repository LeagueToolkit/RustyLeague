use crate::io::binary_reader::BinaryReader;

pub struct Vector3
{
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3
{
    pub fn new(x: f32, y: f32, z: f32) -> Self
    {
        Vector3 { x, y, z }
    }

    pub fn empty() -> Self
    {
        Vector3 { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn read(file: &mut BinaryReader) -> Self
    {
        Vector3
        {
            x: file.read_f32(),
            y: file.read_f32(),
            z: file.read_f32(),
        }
    }
}

