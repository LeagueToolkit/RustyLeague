use std::io::{Seek, Read, Write};
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use std::fmt::Debug;
use std::io;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ColorRGBA<T>
{
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T
}

impl ColorRGBA<f32>
{
    pub fn new_f32(r: f32, g: f32, b: f32, a: f32) -> Self
    {
        ColorRGBA
        {
            r, g, b, a
        }
    }
    pub fn read_f32<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self>
    {
        Ok(ColorRGBA
        {
            r: reader.read_f32()?,
            g: reader.read_f32()?,
            b: reader.read_f32()?,
            a: reader.read_f32()?
        })
    }

    pub fn write_f32<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>
    {
        writer.write_f32(self.r)?;
        writer.write_f32(self.g)?;
        writer.write_f32(self.b)?;
        writer.write_f32(self.a)?;

        Ok(())
    }
}

impl ColorRGBA<u8>
{
    pub fn new_u8(r: u8, g: u8, b: u8, a: u8) -> Self
    {
        ColorRGBA
        {
            r, g, b, a
        }
    }
    pub fn read_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self>
    {
        Ok(ColorRGBA
        {
            r: reader.read_u8()?,
            g: reader.read_u8()?,
            b: reader.read_u8()?,
            a: reader.read_u8()?
        })
    }

    pub fn write_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>
    {
        writer.write_u8(self.r)?;
        writer.write_u8(self.g)?;
        writer.write_u8(self.b)?;
        writer.write_u8(self.a)?;

        Ok(())
    }
}