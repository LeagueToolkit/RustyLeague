use std::io::{Read, Seek, Write};
use std::io;
use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use glam::Mat4;

pub trait Mat4Ext: Sized {
    fn read_row_major<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self>;
    fn write_row_major<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()>;
}

impl Mat4Ext for Mat4 {
    fn read_row_major<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let mut matrix = [[0.0, 0.0, 0.0, 0.0]; 4];

        for row in matrix.iter_mut() {
            for entry in row.iter_mut() {
                *entry = reader.read_f32()?;
            }
        }

        // By transposing we convert the matrix to column major
        Ok(Mat4::from_cols_array_2d(&matrix).transpose())
    }

    fn write_row_major<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        let matrix = self.transpose(); // By transposing we convert the matrix to row major
        let matrix = matrix.to_cols_array_2d();

        for row in matrix.iter() {
            for entry in row.iter() {
                writer.write_f32(*entry)?;
            }
        }

        Ok(())
    }
}