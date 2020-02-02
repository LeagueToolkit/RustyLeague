use super::vector3::Vector3;
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;

#[derive(Copy, Clone)]
pub struct Sphere
{
    pub center: Vector3,
    pub radius: f32
}

impl Sphere
{
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