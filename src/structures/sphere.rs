use super::vector3::Vector3;
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;

#[derive(Copy, Clone, PartialEq)]
pub struct Sphere
{
    pub center: Vector3,
    pub radius: f32
}

impl Sphere
{
    pub const ZERO: Sphere = Sphere { center: Vector3::ZERO, radius: 0.0 };

    pub fn zero() -> Self
    {
        Sphere
        {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: 0.0
        }
    }
    pub fn new(center: Vector3, radius: f32) -> Self
    {
        Sphere
        {
            center,
            radius
        }
    }
    pub fn read(reader: &mut BinaryReader) -> Self
    {
        Sphere
        {
            center: Vector3::read(reader),
            radius: reader.read_f32()
        }
    }

    pub fn write(&self, writer: &mut BinaryWriter)
    {
        self.center.write(writer);
        writer.write(self.radius);
    }
}