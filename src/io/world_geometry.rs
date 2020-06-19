use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use crate::structures::box3d::Box3D;
use crate::structures::render_bucket_grid::RenderBucketGrid;
use crate::structures::sphere::Sphere;
use crate::structures::vector2::Vector2;
use crate::structures::vector3::Vector3;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, Write};
use std::path::Path;
use std::string::String;

pub struct WorldGeometry {
    models: Vec<WorldGeometryModel>,
    bucket_grid: RenderBucketGrid,
}

#[derive(Clone, PartialEq)]
pub struct WorldGeometryModel {
    pub texture: String,
    pub material: String,
    bounding_sphere: Sphere,
    bounding_box: Box3D,
    vertices: Vec<WorldGeometryVertex>,
    indices: Vec<u32>,
}

#[derive(Copy, Clone, PartialEq)]
pub struct WorldGeometryVertex {
    pub position: Vector3,
    pub uv: Vector2,
}

impl WorldGeometry {
    pub fn new(models: Vec<WorldGeometryModel>, bucket_grid_template: RenderBucketGrid) -> Self {
        WorldGeometry {
            models,
            bucket_grid: bucket_grid_template,
        }
    }
    pub fn read_from_file(file_location: &Path) -> io::Result<Self> {
        WorldGeometry::read(&mut BinaryReader::from_location(file_location))
    }
    pub fn read_from_buffer(buffer: Cursor<Vec<u8>>) -> io::Result<Self> {
        WorldGeometry::read(&mut BinaryReader::from_buffer(buffer))
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        let magic: String = reader.read_string(4)?;
        if &magic != "WGEO" {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Incorrect file signature",
            ));
        }

        let version: u32 = reader.read_u32()?;
        if version != 5 && version != 4 {
            return Err(Error::new(ErrorKind::InvalidData, "Unsupported version"));
        }

        Ok(WorldGeometry {
            models: {
                let model_count: u32 = reader.read_u32()?;
                let face_count: u32 = reader.read_u32()?;
                let mut models: Vec<WorldGeometryModel> = Vec::with_capacity(model_count as usize);

                for i in 0..model_count {
                    models.push(WorldGeometryModel::read(reader)?);
                }

                models
            },
            bucket_grid: {
                if version == 5 {
                    RenderBucketGrid::read(reader)?
                } else {
                    RenderBucketGrid::empty()
                }
            },
        })
    }

    pub fn write_to_file(&mut self, file_location: &Path) -> io::Result<()> {
        self.write(&mut BinaryWriter::from_location(file_location))
    }
    pub fn write_to_buffer(&mut self, buffer: Cursor<Vec<u8>>) -> io::Result<()> {
        self.write(&mut BinaryWriter::from_buffer(buffer))
    }
    pub fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        writer.write_string("WGEO")?;
        writer.write_u32(5)?; // Version
        writer.write_u32(self.models.len() as u32)?; // Model Count

        let face_count = {
            let mut face_count = 0;

            for (i, model) in self.models.iter().enumerate() {
                face_count += model.indices.len() as u32 / 3;
            }

            face_count
        };
        writer.write_u32(face_count)?;

        for model in &mut self.models {
            model.write(writer)?;
        }

        self.bucket_grid.write(writer)?;

        Ok(())
    }

    pub fn add_model(&mut self, model: WorldGeometryModel) {
        self.models.push(model);
    }
    pub fn remove_model(&mut self, index: usize) {
        self.models.remove(index);
    }

    pub fn models(&self) -> &Vec<WorldGeometryModel> {
        return &self.models;
    }
    pub fn bucket_grid(&self) -> &RenderBucketGrid {
        return &self.bucket_grid;
    }
}

impl WorldGeometryModel {
    pub fn new(
        texture: String,
        material: String,
        vertices: Vec<WorldGeometryVertex>,
        indices: Vec<u32>,
    ) -> Self {
        WorldGeometryModel {
            texture,
            material,
            bounding_box: Box3D::zero(),
            bounding_sphere: Sphere::zero(),
            vertices,
            indices,
        }
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        let texture = reader.read_padded_string(260)?;
        let material = reader.read_padded_string(64)?;
        let bounding_sphere = Sphere::read(reader)?;
        let bounding_box = Box3D::read(reader)?;
        let vertex_count = reader.read_u32()?;
        let index_count = reader.read_u32()?;

        let mut vertices: Vec<WorldGeometryVertex> = Vec::with_capacity(vertex_count as usize);
        for i in 0..vertex_count {
            vertices.push(WorldGeometryVertex::read(reader)?);
        }

        let mut indices: Vec<u32> = Vec::with_capacity(index_count as usize);
        if index_count <= 65536 {
            for i in 0..index_count {
                indices.push(reader.read_u16()? as u32);
            }
        } else {
            for i in 0..index_count {
                indices.push(reader.read_u32()?);
            }
        }

        Ok(WorldGeometryModel {
            texture,
            material,
            bounding_sphere,
            bounding_box,
            vertices,
            indices,
        })
    }

    pub fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        writer.write_padded_string(&self.texture, 260)?;
        writer.write_padded_string(&self.material, 64)?;

        self.bounding_sphere().write(writer)?;
        self.bounding_box().write(writer)?;

        writer.write(self.vertices.len() as u32)?;
        writer.write(self.indices.len() as u32)?;

        for vertex in &mut self.vertices {
            vertex.write(writer);
        }

        if self.indices.len() <= 65536 {
            for index in &self.indices {
                writer.write(*index as u16)?;
            }
        } else {
            for index in &self.indices {
                writer.write(*index as u32)?;
            }
        }

        Ok(())
    }

    pub fn central_point(&mut self) -> Vector3 {
        let bounds = self.bounding_box();

        Vector3 {
            x: 0.5 * (bounds.max.x - bounds.min.x),
            y: 0.5 * (bounds.max.y - bounds.min.y),
            z: 0.5 * (bounds.max.z - bounds.min.z),
        }
    }
    pub fn bounding_box(&mut self) -> Box3D {
        if self.bounding_box == Box3D::ZERO && !self.vertices.is_empty() {
            let mut min = Vector3::new(
                self.vertices[0].position.x,
                std::f32::NEG_INFINITY,
                self.vertices[0].position.z,
            );
            let mut max = Vector3::new(
                self.vertices[0].position.x,
                std::f32::INFINITY,
                self.vertices[0].position.z,
            );

            for vertex in &self.vertices {
                if min.x > vertex.position.x {
                    min.x = vertex.position.x
                };
                if min.y > vertex.position.y {
                    min.y = vertex.position.y
                };
                if min.z > vertex.position.z {
                    min.z = vertex.position.z
                };
                if max.x < vertex.position.x {
                    max.x = vertex.position.x
                };
                if max.y < vertex.position.y {
                    max.y = vertex.position.y
                };
                if max.z < vertex.position.z {
                    max.z = vertex.position.z
                };
            }

            self.bounding_box = Box3D::new(min, max);
        }

        return self.bounding_box;
    }
    pub fn bounding_sphere(&mut self) -> Sphere {
        if self.bounding_sphere == Sphere::ZERO {
            let bounds = self.bounding_box();
            let central_point = self.central_point();
            self.bounding_sphere =
                Sphere::new(central_point, Vector3::distance(central_point, bounds.max));
        }

        return self.bounding_sphere;
    }
    pub fn vertices(&self) -> &Vec<WorldGeometryVertex> {
        return &self.vertices;
    }
    pub fn indices(&self) -> &Vec<u32> {
        return &self.indices;
    }

    pub fn set_model_data(&mut self, vertices: Vec<WorldGeometryVertex>, indices: Vec<u32>) {
        self.vertices = vertices;
        self.indices = indices;
    }
}

impl WorldGeometryVertex {
    pub fn new(position: Vector3, uv: Vector2) -> Self {
        WorldGeometryVertex { position, uv }
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        Ok(WorldGeometryVertex {
            position: Vector3::read(reader)?,
            uv: Vector2::read(reader)?,
        })
    }

    pub fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        self.position.write(writer)?;
        self.uv.write(writer)?;

        Ok(())
    }
}
