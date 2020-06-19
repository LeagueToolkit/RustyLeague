use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use crate::structures::box3d::Box3D;
use crate::structures::vector3::Vector3;
use std::io;
use std::io::{Read, Seek, Write};

#[derive(Clone)]
pub struct RenderBucketGrid {
    bounds: Box3D,
    vertices: Vec<Vector3>,
    indices: Vec<u16>,
    buckets: Vec<Vec<RenderBucket>>,
}

#[derive(Copy, Clone, PartialEq)]
pub struct RenderBucket {
    max_stick_out_x: f32,
    max_stick_out_z: f32,
    start_index: u32,
    base_vertex: u32,
    inside_face_count: u16,
    sticking_out_face_count: u16,
}

impl RenderBucketGrid {
    pub fn empty() -> Self {
        RenderBucketGrid {
            bounds: Box3D::new(Vector3::zero(), Vector3::zero()),
            vertices: Vec::new(),
            indices: Vec::new(),
            buckets: Vec::new(),
        }
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        let min_x = reader.read_f32()?;
        let min_z = reader.read_f32()?;
        let max_x = reader.read_f32()?;
        let max_z = reader.read_f32()?;
        let max_stick_out_x = reader.read_f32()?;
        let max_stick_out_z = reader.read_f32()?;
        let bucket_size_x = reader.read_f32()?;
        let bucket_size_z = reader.read_f32()?;

        let buckets_per_side = reader.read_u16()?;
        let unknown = reader.read_u16()?;
        let vertex_count = reader.read_u32()?;
        let index_count = reader.read_u32()?;
        let mut vertices: Vec<Vector3> = Vec::with_capacity(vertex_count as usize);
        let mut indices: Vec<u16> = Vec::with_capacity(index_count as usize);
        let mut buckets: Vec<Vec<RenderBucket>> = Vec::with_capacity(buckets_per_side as usize);

        for i in 0..vertex_count {
            vertices.push(Vector3::read(reader)?);
        }
        for i in 0..index_count {
            indices.push(reader.read_u16()?);
        }
        for i in 0..buckets_per_side {
            let mut bucket_row: Vec<RenderBucket> = Vec::with_capacity(buckets_per_side as usize);

            for j in 0..buckets_per_side {
                bucket_row.push(RenderBucket::read(reader)?);
            }

            buckets.push(bucket_row);
        }

        Ok(RenderBucketGrid {
            bounds: {
                let min_vector = Vector3::new(min_x, std::f32::NEG_INFINITY, min_z);
                let max_vector = Vector3::new(max_x, std::f32::INFINITY, max_z);
                Box3D::new(min_vector, max_vector)
            },
            vertices,
            indices,
            buckets,
        })
    }

    pub fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        let bounds = self.bounds();
        let (max_stick_out_x, max_stick_out_z) = self.max_stick_out();
        let (bucket_size_x, bucket_size_z) = self.bucket_size();
        let buckets_per_side = self.buckets_per_side();

        writer.write(bounds.min.x)?;
        writer.write(bounds.min.z)?;
        writer.write(bounds.max.x)?;
        writer.write(bounds.max.x)?;
        writer.write(max_stick_out_x)?;
        writer.write(max_stick_out_z)?;
        writer.write(bucket_size_x)?;
        writer.write(bucket_size_z)?;
        writer.write(buckets_per_side)?;
        writer.write(0 as u16)?;
        writer.write(self.vertices.len() as u32)?;
        writer.write(self.indices.len() as u32)?;

        for vertex in &mut self.vertices {
            vertex.write(writer)?;
        }
        for index in &self.indices {
            writer.write(*index)?;
        }
        for bucketRow in &mut self.buckets {
            for bucket in bucketRow {
                bucket.write(writer)?;
            }
        }

        Ok(())
    }

    pub fn bounds(&mut self) -> Box3D {
        if self.bounds == Box3D::ZERO {
            let mut min = Vector3::new(
                self.vertices[0].x,
                std::f32::NEG_INFINITY,
                self.vertices[0].z,
            );
            let mut max = Vector3::new(self.vertices[0].x, std::f32::INFINITY, self.vertices[0].z);

            for vertex in &self.vertices {
                if min.x > vertex.x {
                    min.x = vertex.x
                };
                if min.z > vertex.z {
                    min.z = vertex.z
                };
                if max.x < vertex.x {
                    max.x = vertex.x
                };
                if max.z < vertex.z {
                    max.z = vertex.z
                };
            }

            self.bounds = Box3D::new(min, max);
        }

        return self.bounds;
    }
    pub fn max_stick_out(&self) -> (f32, f32) {
        let mut max_stick_out_x: f32 = 0.0;
        let mut max_stick_out_z: f32 = 0.0;

        for bucketRow in &self.buckets {
            for bucket in bucketRow {
                let (bucket_max_stick_out_x, bucket_max_stick_out_z) = bucket.max_stick_out();

                if max_stick_out_x < bucket_max_stick_out_x {
                    max_stick_out_x = bucket_max_stick_out_x
                }
                if max_stick_out_z < bucket_max_stick_out_z {
                    max_stick_out_z = bucket_max_stick_out_z
                }
            }
        }

        return (max_stick_out_x, max_stick_out_z);
    }
    pub fn bucket_size(&mut self) -> (f32, f32) {
        let bounds = self.bounds();
        let length_x = f32::abs(bounds.min.x) + f32::abs(bounds.max.x);
        let length_z = f32::abs(bounds.min.z) + f32::abs(bounds.max.z);
        let buckets_per_side = self.buckets_per_side() as f32;

        (length_x / buckets_per_side, length_z / buckets_per_side)
    }
    pub fn buckets_per_side(&self) -> u16 {
        return self.buckets.len() as u16;
    }
    pub fn vertices(&self) -> &Vec<Vector3> {
        return &self.vertices;
    }
    pub fn indices(&self) -> &Vec<u16> {
        return &self.indices;
    }
    pub fn buckets(&self) -> &Vec<Vec<RenderBucket>> {
        return &self.buckets;
    }
}

impl RenderBucket {
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        Ok(RenderBucket {
            max_stick_out_x: reader.read_f32()?,
            max_stick_out_z: reader.read_f32()?,
            start_index: reader.read_u32()?,
            base_vertex: reader.read_u32()?,
            inside_face_count: reader.read_u16()?,
            sticking_out_face_count: reader.read_u16()?,
        })
    }

    pub fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        writer.write(self.max_stick_out_x)?;
        writer.write(self.max_stick_out_z)?;
        writer.write(self.start_index)?;
        writer.write(self.base_vertex)?;
        writer.write(self.inside_face_count)?;
        writer.write(self.sticking_out_face_count)?;

        Ok(())
    }

    pub fn max_stick_out(&self) -> (f32, f32) {
        return (self.max_stick_out_x, self.max_stick_out_z);
    }
    pub fn start_index(&self) -> u32 {
        return self.start_index;
    }
    pub fn base_vertex(&self) -> u32 {
        return self.base_vertex;
    }
    pub fn inside_face_count(&self) -> u16 {
        return self.inside_face_count;
    }
    pub fn sticking_out_face_count(&self) -> u16 {
        return self.sticking_out_face_count;
    }
}
