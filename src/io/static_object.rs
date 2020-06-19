use crate::io::binary_reader::BinaryReader;
use crate::structures::box3d::Box3D;
use crate::structures::color::ColorRgba;
use crate::structures::vector2::Vector2;
use crate::structures::vector3::Vector3;
use bitflags;
use palette::LinSrgba;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Seek};
use std::ops::SubAssign;
use std::path::Path;

bitflags! {
    struct StaticObjectFlags : u32
    {
        const VERTEX_COLORS = 1;
        const LOCAL_ORIGIN_LOCATOR_AND_PIVOT = 2;
    }
}
#[derive(Debug)]
pub struct StaticObject {
    pub name: String,
    submeshes: Vec<StaticObjectSubmesh>,
    bounding_box: Box3D,
}

#[derive(Debug)]
pub struct StaticObjectSubmesh {
    pub name: String,
    vertices: Vec<StaticObjectVertex>,
    indices: Vec<u32>,
}
#[derive(Debug)]
pub struct StaticObjectVertex {
    pub position: Vector3,
    pub uv: Vector2,
    pub color: Option<LinSrgba>,
}

struct StaticObjectFace {
    indices: [u32; 3],
    material: String,
    uvs: [Vector2; 3],
}

impl StaticObject {
    pub fn read_scb_from_file(file_location: &Path) -> io::Result<Self> {
        StaticObject::read_scb(&mut BinaryReader::from_location(file_location))
    }
    pub fn read_scb_from_buffer(buffer: Cursor<Vec<u8>>) -> io::Result<Self> {
        StaticObject::read_scb(&mut BinaryReader::from_buffer(buffer))
    }
    pub fn read_scb<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        let magic = reader.read_string(8)?;
        if &magic != "r3d2Mesh" {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Incorrect file signature",
            ));
        }

        let major = reader.read_u16()?;
        let minor = reader.read_u16()?;
        if (major != 3 && major != 2) || (minor != 1 && minor != 2) {
            return Err(Error::new(ErrorKind::InvalidData, "Unsupported version"));
        }

        let name = reader.read_padded_string(128)?;
        let vertex_count = reader.read_u32()?;
        let face_count = reader.read_u32()?;
        let flags = StaticObjectFlags::from_bits(reader.read_u32()?);
        let bounding_box = Box3D::read(reader)?;
        let has_vertex_colors = if major == 3 && minor == 2 {
            reader.read_u32()? == 1
        } else {
            false
        };

        let mut vertices: Vec<Vector3> = Vec::with_capacity(vertex_count as usize);
        let mut vertex_colors: Vec<LinSrgba> = Vec::default();
        for i in 0..vertex_count {
            vertices.push(Vector3::read(reader)?);
        }
        if has_vertex_colors {
            vertex_colors.reserve(vertex_count as usize);

            for i in 0..vertex_count {
                vertex_colors.push(LinSrgba::read_rgba_u8(reader)?);
            }
        }

        let central_point = Vector3::read(reader)?;

        let mut faces: Vec<StaticObjectFace> = Vec::with_capacity(face_count as usize);
        for i in 0..face_count {
            faces.push(StaticObjectFace::read_binary(reader)?);
        }

        Ok(StaticObject {
            name,
            submeshes: StaticObject::create_submeshes(&vertices, &vertex_colors, &faces),
            bounding_box,
        })
    }

    fn create_submeshes(
        vertices: &Vec<Vector3>,
        vertex_colors: &Vec<LinSrgba>,
        faces: &Vec<StaticObjectFace>,
    ) -> Vec<StaticObjectSubmesh> {
        let has_vertex_colors = vertex_colors.len() != 0;
        let submesh_map = StaticObject::create_submesh_map(faces);
        let mut submeshes: Vec<StaticObjectSubmesh> = Vec::with_capacity(submesh_map.len());

        for (submesh_name, faces) in submesh_map {
            // Collect all indices and build UV map
            let mut indices: Vec<u32> = Vec::with_capacity(faces.len() * 3);
            let mut uv_map: HashMap<u32, Vector2> = HashMap::with_capacity(faces.len() * 3);
            for face in faces {
                indices.push(face.indices[0]);
                indices.push(face.indices[1]);
                indices.push(face.indices[2]);

                for i in 0..3 {
                    if !uv_map.contains_key(&face.indices[i]) {
                        uv_map.insert(face.indices[i], face.uvs[i]);
                    }
                }
            }

            // Get vertex range from indices
            let mut min_vertex = std::u32::MAX;
            let mut max_vertex = std::u32::MIN;
            for index in &indices {
                if min_vertex > *index {
                    min_vertex = *index;
                }

                if max_vertex < *index {
                    max_vertex = *index;
                }
            }

            // Build vertex list
            let vertex_count = max_vertex - min_vertex;
            let mut submesh_vertices: Vec<StaticObjectVertex> =
                Vec::with_capacity(vertex_count as usize);
            for i in min_vertex..vertex_count {
                let uv = *uv_map.get(&i).unwrap();

                if has_vertex_colors {
                    submesh_vertices.push(StaticObjectVertex::new_color(
                        vertices[i as usize],
                        uv,
                        vertex_colors[i as usize],
                    ));
                } else {
                    submesh_vertices.push(StaticObjectVertex::new_basic(vertices[i as usize], uv));
                }
            }

            //Normalize indices
            for index in &mut indices {
                index.sub_assign(min_vertex);
            }

            submeshes.push(StaticObjectSubmesh::new(
                submesh_name.clone(),
                submesh_vertices,
                indices,
            ));
        }

        return submeshes;
    }
    fn create_submesh_map(
        faces: &Vec<StaticObjectFace>,
    ) -> HashMap<String, Vec<&StaticObjectFace>> {
        let mut submesh_map = HashMap::new();

        //Group faces by material
        for face in faces {
            if !submesh_map.contains_key(&face.material) {
                submesh_map.insert(face.material.clone(), Vec::new());
            }

            submesh_map.get_mut(&face.material).unwrap().push(face);
        }

        return submesh_map;
    }
}

impl StaticObjectSubmesh {
    pub fn new(name: String, vertices: Vec<StaticObjectVertex>, indices: Vec<u32>) -> Self {
        StaticObjectSubmesh {
            name,
            vertices,
            indices,
        }
    }

    pub fn set_data(&mut self, vertices: Vec<StaticObjectVertex>, indices: Vec<u32>) {
        self.vertices = vertices;
        self.indices = indices;
    }

    pub fn vertices(&mut self) -> &mut Vec<StaticObjectVertex> {
        &mut self.vertices
    }
    pub fn indices(&mut self) -> &mut Vec<u32> {
        &mut self.indices
    }
}

impl StaticObjectVertex {
    pub fn new_basic(position: Vector3, uv: Vector2) -> Self {
        StaticObjectVertex {
            position,
            uv,
            color: Option::None,
        }
    }
    pub fn new_color(position: Vector3, uv: Vector2, color: LinSrgba) -> Self {
        StaticObjectVertex {
            position,
            uv,
            color: Option::Some(color),
        }
    }
}

impl StaticObjectFace {
    fn read_binary<T: Read + Seek>(reader: &mut BinaryReader<T>) -> io::Result<Self> {
        Ok(StaticObjectFace {
            indices: [reader.read_u32()?, reader.read_u32()?, reader.read_u32()?],
            material: reader.read_padded_string(64)?,
            uvs: [
                Vector2::read(reader)?,
                Vector2::read(reader)?,
                Vector2::read(reader)?,
            ],
        })
    }
}
