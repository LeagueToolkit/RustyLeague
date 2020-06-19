use std::io::{Seek, Read, Write};
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use std::io;
use palette::{LinSrgba, LinSrgb};

pub trait ColorRgba: Sized {
    fn read_rgba_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self>;
    fn read_rgba_f32<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self >;

    fn write_rgba_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>;
    fn write_rgba_f32<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>;
}
pub trait ColorRgb: Sized {
    fn read_rgb_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self>;
    fn read_bgr_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self>;

    fn write_rgb_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>;
    fn write_bgr_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>;
}

impl ColorRgba for LinSrgba {
    fn read_rgba_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(LinSrgba::new(reader.read_u8()? as f32 / 255.0,
                         reader.read_u8()? as f32 / 255.0,
                         reader.read_u8()? as f32 / 255.0,
                         reader.read_u8()? as f32 / 255.0))
    }
    fn read_rgba_f32<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(LinSrgba::new(reader.read_f32()?,
                         reader.read_f32()?,
                         reader.read_f32()?,
                         reader.read_f32()?))
    }

    fn write_rgba_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_u8((self.color.red * 255.0) as u8)?;
        writer.write_u8((self.color.green * 255.0) as u8)?;
        writer.write_u8((self.color.blue * 255.0) as u8)?;
        writer.write_u8((self.alpha * 255.0) as u8)?;

        Ok(())
    }
    fn write_rgba_f32<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_f32(self.color.red)?;
        writer.write_f32(self.color.green)?;
        writer.write_f32(self.color.blue)?;
        writer.write_f32(self.alpha)?;

        Ok(())
    }
}

impl ColorRgb for LinSrgb {
    fn read_rgb_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(LinSrgb::new(reader.read_u8()? as f32 / 255.0,
                        reader.read_u8()? as f32 / 255.0,
                        reader.read_u8()? as f32 / 255.0))
    }
    fn read_bgr_u8<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let b = reader.read_u8()? as f32 / 255.0;
        let g = reader.read_u8()? as f32 / 255.0;
        let r = reader.read_u8()? as f32 / 255.0;

        Ok(LinSrgb::new(r,g,b))
    }

    fn write_rgb_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_u8((self.red * 255.0) as u8)?;
        writer.write_u8((self.green * 255.0) as u8)?;
        writer.write_u8((self.blue * 255.0) as u8)?;

        Ok(())
    }
    fn write_bgr_u8<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_u8((self.blue * 255.0) as u8)?;
        writer.write_u8((self.green * 255.0) as u8)?;
        writer.write_u8((self.red * 255.0) as u8)?;

        Ok(())
    }
}