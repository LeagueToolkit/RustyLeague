use super::vector3::Vector3;
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;

#[derive(Copy, Clone)]
pub struct R3DSphere
{
    pub center: Vector3,
    pub radius: f32
}

impl R3DSphere
{
    pub fn read(reader: &mut BinaryReader) -> Self
    {
        R3DSphere
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