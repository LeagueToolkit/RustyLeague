use crate::io::binary_reader::BinaryReader;

pub struct Vector2
{
    pub x: f32,
    pub y: f32
}

impl Vector2
{
    pub fn new(x: f32, y: f32) -> Self
    {
        Vector2 { x, y }
    }

    pub fn empty() -> Self
    {
        Vector2 { x: 0.0, y: 0.0 }
    }

    pub fn read(file: &mut BinaryReader) -> Self
    {
        Vector2
        {
            x: file.read_f32(),
            y: file.read_f32()
        }
    }
}