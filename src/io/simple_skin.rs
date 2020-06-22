use crate::io::binary_reader::BinaryReader;
use crate::io::binary_writer::BinaryWriter;
use crate::structures::box3d::Box3D;
use crate::structures::color::LinSrgbaExt;
use crate::structures::sphere::Sphere;
use crate::structures::vector2::Vector2;
use crate::structures::vector3::Vector3;
use palette::LinSrgba;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, Write};
use std::ops::SubAssign;
use std::path::Path;

#[derive(Debug)]
pub struct SimpleSkin {
    submeshes: Vec<SimpleSkinSubmesh>,
    bounding_box: Box3D,
    bounding_sphere: Sphere,
}

#[derive(Clone, Debug)]
pub struct SimpleSkinSubmesh {
    pub name: String,
    vertices: Vec<SimpleSkinVertex>,
    indices: Vec<u16>,

    // Used exclusively for reading
    start_vertex: u32,
    vertex_count: u32,
    start_index: u32,
    index_count: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct SimpleSkinVertex {
    pub position: Vector3,
    pub influences: [u8; 4],
    pub weights: [f32; 4],
    pub normal: Vector3,
    pub uv: Vector2,
    pub color: Option<LinSrgba>,
}

impl SimpleSkin {
    pub fn new(submeshes: Vec<SimpleSkinSubmesh>) -> Self {
        SimpleSkin {
            submeshes,
            bounding_box: Box3D::zero(),
            bounding_sphere: Sphere::zero(),
        }
    }

    pub fn read_from_file(file_location: &Path) -> io::Result<Self> {
        SimpleSkin::read(&mut BinaryReader::from_location(file_location))
    }
    pub fn read_from_buffer(buffer: Cursor<Vec<u8>>) -> io::Result<Self> {
        SimpleSkin::read(&mut BinaryReader::from_buffer(buffer))
    }
    fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        let magic = reader.read_u32()?;
        if magic != 0x00112233 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Incorrect file signature",
            ));
        }

        let major = reader.read_u16()?;
        let minor = reader.read_u16()?;
        if major != 2 && major != 4 && minor != 1 {
            return Err(Error::new(ErrorKind::InvalidData, "Unsupported version"));
        }

        let submesh_count = reader.read_u32()?;
        let mut submeshes: Vec<SimpleSkinSubmesh> = Vec::with_capacity(submesh_count as usize);
        for i in 0..submesh_count {
            submeshes.push(SimpleSkinSubmesh::read(reader)?);
        }

        let flags = if major == 4 { reader.read_u32()? } else { 0 };
        let index_count = reader.read_u32()?;
        let vertex_count = reader.read_u32()?;
        let vertex_size = if major == 4 { reader.read_u32()? } else { 52 };
        let vertex_type = if major == 4 { reader.read_u32()? } else { 0 };
        let bounding_box = if major == 4 {
            Box3D::read(reader)?
        } else {
            Box3D::zero()
        };
        let bounding_sphere = if major == 4 {
            Sphere::read(reader)?
        } else {
            Sphere::zero()
        };

        if (vertex_type == 0 && vertex_size != 52) || (vertex_type == 1 && vertex_size != 56) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Vertex size doesn't match with the vertex size",
            ));
        }

        let mut indices: Vec<u16> = Vec::with_capacity(index_count as usize);
        let mut vertices: Vec<SimpleSkinVertex> = Vec::with_capacity(vertex_count as usize);

        for i in 0..index_count {
            indices.push(reader.read_u16()?);
        }
        for i in 0..vertex_count {
            vertices.push(SimpleSkinVertex::read(vertex_type, reader)?);
        }

        // Now we need to assign data to submeshes
        for submesh in &mut submeshes {
            let mut submesh_vertices: Vec<SimpleSkinVertex> =
                Vec::with_capacity(submesh.vertex_count as usize);
            let mut submesh_indices: Vec<u16> = Vec::with_capacity(submesh.index_count as usize);
            let mut min_index = std::u16::MAX;

            for i in 0..submesh.vertex_count {
                submesh_vertices.push(vertices[(i + submesh.start_vertex) as usize]);
            }
            for i in 0..submesh.index_count {
                let index = indices[(i + submesh.start_index) as usize];
                if min_index > index {
                    min_index = index;
                }

                submesh_indices.push(index);
            }

            //Normalize indices
            for index in &mut submesh_indices {
                index.sub_assign(min_index);
            }

            submesh.set_data(submesh_vertices, submesh_indices);
        }

        Ok(SimpleSkin {
            submeshes,
            bounding_box,
            bounding_sphere,
        })
    }

    pub fn write_to_file(&mut self, file_location: &Path) -> io::Result<()> {
        self.write(&mut BinaryWriter::from_location(file_location))
    }
    pub fn write_to_buffer(&mut self, buffer: Cursor<Vec<u8>>) -> io::Result<()> {
        self.write(&mut BinaryWriter::from_buffer(buffer))
    }
    fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        writer.write_u32(0x00112233)?; // Magic
        writer.write_u16(4)?; // Major
        writer.write_u16(1)?; // Minor
        writer.write_u32(self.submeshes.len() as u32)?;

        let mut index_offset = 0u32;
        let mut vertex_offset = 0u32;
        let mut contains_vertex_color = false;
        for submesh in &self.submeshes {
            submesh.write(vertex_offset, index_offset, writer)?;

            vertex_offset += submesh.vertices.len() as u32;
            index_offset += submesh.indices.len() as u32;

            if submesh.contains_vertex_color() {
                contains_vertex_color = true;
            }
        }

        writer.write(0u32)?; // Flags
        writer.write(index_offset)?; // Vertex Count
        writer.write(vertex_offset)?; // Index Count
        writer.write(if contains_vertex_color { 56 } else { 52 })?; // Vertex Size
        writer.write(contains_vertex_color as u32)?; // Vertex Type

        self.bounding_box().write(writer)?;
        self.bounding_sphere().write(writer)?;

        let mut index_offset = 0u16;
        for submesh in self.submeshes() {
            for index in submesh.indices() {
                writer.write(*index + index_offset)?;
            }

            index_offset += submesh.indices.len() as u16;
        }
        for submesh in self.submeshes() {
            for vertex in submesh.vertices() {
                if vertex.color.is_none() && contains_vertex_color {
                    // Copy vertex so we don't modify mesh data but save a correct file
                    let mut color_vertex = *vertex;
                    color_vertex.color = Option::from(LinSrgba::new(0.0, 0.0, 0.0, 0.0));
                    color_vertex.write(writer)?;
                } else {
                    vertex.write(writer)?;
                }
            }
        }

        Ok(())
    }

    pub fn add_submesh(&mut self, submesh: SimpleSkinSubmesh) {
        self.submeshes.push(submesh);
    }
    pub fn remove_submesh_name(&mut self, name: String) {
        let index = self
            .submeshes
            .iter()
            .position(|submesh| submesh.name == name);

        if let Some(x) = index { self.submeshes.remove(x); }
    }
    pub fn remove_submesh_index(&mut self, index: usize) {
        if index < self.submeshes.len() {
            self.submeshes.remove(index);
        }
    }

    pub fn submeshes(&mut self) -> &mut [SimpleSkinSubmesh] { &mut self.submeshes }

    pub fn central_point(&mut self) -> Vector3 {
        let bounds = self.bounding_box();

        Vector3 {
            x: 0.5 * (bounds.max.x - bounds.min.x),
            y: 0.5 * (bounds.max.y - bounds.min.y),
            z: 0.5 * (bounds.max.z - bounds.min.z),
        }
    }
    pub fn bounding_box(&mut self) -> Box3D {
        if self.bounding_box == Box3D::ZERO {
            let mut min = Vector3::new(std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY);
            let mut max = Vector3::new(
                std::f32::NEG_INFINITY,
                std::f32::NEG_INFINITY,
                std::f32::NEG_INFINITY,
            );

            for submesh in &mut self.submeshes {
                for vertex in submesh.vertices() {
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
            }

            self.bounding_box = Box3D::new(min, max);
        }

        self.bounding_box
    }
    pub fn bounding_sphere(&mut self) -> Sphere {
        if self.bounding_sphere == Sphere::ZERO {
            let bounds = self.bounding_box();
            let central_point = self.central_point();
            self.bounding_sphere =
                Sphere::new(central_point, Vector3::distance(central_point, bounds.max));
        }

        self.bounding_sphere
    }
}

impl SimpleSkinSubmesh {
    pub fn new(name: String, vertices: Vec<SimpleSkinVertex>, indices: Vec<u16>) -> Self {
        SimpleSkinSubmesh {
            name,
            vertices,
            indices,
            start_vertex: 0,
            vertex_count: 0,
            start_index: 0,
            index_count: 0,
        }
    }
    fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<SimpleSkinSubmesh> {
        Ok(SimpleSkinSubmesh {
            name: reader.read_padded_string(64)?,
            vertices: Vec::default(),
            indices: Vec::default(),
            start_vertex: reader.read_u32()?,
            vertex_count: reader.read_u32()?,
            start_index: reader.read_u32()?,
            index_count: reader.read_u32()?,
        })
    }

    fn write<T: Write + Seek>(
        &self,
        start_vertex: u32,
        start_index: u32,
        writer: &mut BinaryWriter<T>,
    ) -> io::Result<()> {
        writer.write_padded_string(&self.name, 64)?;
        writer.write_u32(start_vertex)?;
        writer.write_u32(self.vertices.len() as u32)?;
        writer.write_u32(start_index)?;
        writer.write_u32(self.indices.len() as u32)?;

        Ok(())
    }

    pub fn set_data(&mut self, vertices: Vec<SimpleSkinVertex>, indices: Vec<u16>) {
        self.vertices = vertices;
        self.indices = indices;
    }
    fn contains_vertex_color(&self) -> bool {
        for vertex in &self.vertices {
            if vertex.color.is_some() {
                return true;
            }
        }

        return false;
    }

    pub fn vertices(&mut self) -> &mut [SimpleSkinVertex] { &mut self.vertices }
    pub fn indices(&mut self) -> &mut [u16] { &mut self.indices }
}

impl SimpleSkinVertex {
    pub fn new_basic(
        position: Vector3,
        influences: [u8; 4],
        weights: [f32; 4],
        normal: Vector3,
        uv: Vector2,
    ) -> Self {
        SimpleSkinVertex {
            position,
            influences,
            weights,
            normal,
            uv,
            color: Option::None,
        }
    }
    pub fn new_color(
        position: Vector3,
        influences: [u8; 4],
        weights: [f32; 4],
        normal: Vector3,
        uv: Vector2,
        color: LinSrgba,
    ) -> Self {
        SimpleSkinVertex {
            position,
            influences,
            weights,
            normal,
            uv,
            color: Option::Some(color),
        }
    }
    fn read<T: Read + Seek>(vertex_type: u32, reader: &mut BinaryReader<T>) -> io::Result<Self> {
        Ok(SimpleSkinVertex {
            position: Vector3::read(reader)?,
            influences: [
                reader.read_u8()?,
                reader.read_u8()?,
                reader.read_u8()?,
                reader.read_u8()?,
            ],
            weights: [
                reader.read_f32()?,
                reader.read_f32()?,
                reader.read_f32()?,
                reader.read_f32()?,
            ],
            normal: Vector3::read(reader)?,
            uv: Vector2::read(reader)?,
            color: if vertex_type == 1 {
                Option::Some(LinSrgba::read_rgba_u8(reader)?)
            } else {
                Option::None
            },
        })
    }

    fn write<T: Write + Seek>(&mut self, writer: &mut BinaryWriter<T>) -> io::Result<()> {
        self.position.write(writer)?;

        for i in 0..4 {
            writer.write(self.influences[i])?;
        }
        for i in 0..4 {
            writer.write(self.weights[i])?;
        }

        self.normal.write(writer)?;
        self.uv.write(writer)?;

        match self.color {
            Some(x) => x.write_rgba_u8(writer),
            None => Ok(()),
        }
    }
}
