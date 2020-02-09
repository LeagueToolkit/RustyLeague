use std::io::{Seek, Read, Write};
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use std::fmt::Debug;
use std::io;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color<T>
{
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T
}

impl Color<f32>
{
    pub fn new_rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {
            r, g, b, a
        }
    }
    pub fn read_rgba_f32<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Color {
            r: reader.read_f32()?,
            g: reader.read_f32()?,
            b: reader.read_f32()?,
            a: reader.read_f32()?
        })
    }

    pub fn write_rgba_f32<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>
    {
        writer.write_f32(self.r)?;
        writer.write_f32(self.g)?;
        writer.write_f32(self.b)?;
        writer.write_f32(self.a)?;

        Ok(())
    }
}

impl Color<u8>
{
    pub fn new_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r, g, b, a
        }
    }
    pub fn new_bgr_u8(b: u8, g: u8, r: u8 ) -> Self {
        Color {
            b,
            g,
            r,
            a: 255
        }
    }

    pub fn read_rgba_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Color {
            r: reader.read_u8()?,
            g: reader.read_u8()?,
            b: reader.read_u8()?,
            a: reader.read_u8()?
        })
    }
    pub fn read_bgr_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Color{
            b: reader.read_u8()?,
            g: reader.read_u8()?,
            r: reader.read_u8()?,
            a: 255,
        })
    }

    pub fn write_rgba_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>
    {
        writer.write_u8(self.r)?;
        writer.write_u8(self.g)?;
        writer.write_u8(self.b)?;
        writer.write_u8(self.a)?;

        Ok(())
    }
    pub fn write_bgr_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_u8(self.b)?;
        writer.write_u8(self.g)?;
        writer.write_u8(self.r)?;

        Ok(())
    }
}