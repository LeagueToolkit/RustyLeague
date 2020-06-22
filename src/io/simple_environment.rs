use std::io;
use std::io::{Seek, Read, Cursor, SeekFrom};
use crate::io::binary_reader::BinaryReader;
use crate::utilities::version::Version;
use crate::structures::vector3::Vector3;
use palette::LinSrgba;
use num_traits::{FromPrimitive, ToPrimitive};
use bitflags;
use glam::Mat4;
use std::fs::read;
use crate::structures::color::LinSrgbaExt;
use crate::structures::matrix44::Mat4Ext;
use std::path::Path;
use crate::utilities::directx9::d3dformat::D3dFormat;
use crate::structures::sphere::Sphere;
use crate::structures::box3d::Box3D;
use crate::structures::vector2::Vector2;

#[derive(Debug)]
pub struct SimpleEnvironment {
    meshes: Vec<SimpleEnvironmentMesh>
}

bitflags! {
    pub struct SimpleEnvironmentMaterialFlags: u32 {
        const GROUND = 1 << 0;
        const NO_SHADOW = 1 << 1;
        const VERTEX_ALPHA = 1 << 2;
        const LIGHTMAPPED = 1 << 3;
        const DUAL_VERTEX_COLOR = 1 << 4;
        const BACKGROUND = 1 << 5;
        const BK_WITH_FOG = 1 << 6;
    }
}

#[derive(Debug)]
pub struct SimpleEnvironmentMaterial {
    name: String,
    material_type: SimpleEnvironmentMaterialType,
    flags: SimpleEnvironmentMaterialFlags,
    channels: Vec<SimpleEnvironmentChannel>
}

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq)]
pub enum SimpleEnvironmentMaterialType {
    Default = 0,
    Decal = 1,
    WallOfGrass = 2,
    FourBlend = 3,
    AntiBrush = 4
}

#[derive(Debug)]
pub struct SimpleEnvironmentChannel {
    color: LinSrgba,
    texture: String,
    transform: Mat4
}

#[derive(Debug)]
struct SimpleEnvironmentVertexBuffer {
    size: u32,
    offset: u64
}

#[derive(Debug, Copy, Clone)]
pub enum SimpleEnvironmentVertex {
    Default { positon: Vector3, normal: Vector3, uv: Vector2, color: LinSrgba },
    Position { position: Vector3 } ,
    Uv2 { positon: Vector3, normal: Vector3, uv0: Vector2, uv1: Vector2, color: LinSrgba },
    // Uv3, - UNUSED
    Color2 { positon: Vector3, normal: Vector3, uv: Vector2, diffuse_color: LinSrgba, emissive_color: LinSrgba },
    // Decal, - UNUSED
    // PosUv1 - UNUSED
}

#[derive(Clone, Copy, Debug)]
pub enum SimpleEnvironmentVertexType {
    Default,
    Position,
    Uv2,
    Color2
}

#[derive(Debug)]
pub struct SimpleEnvironmentMesh {
    quality: i32,
    flags: u32,
    material: String,
    simple_geometry: SimpleEnvironmentMeshGeometry,
    complex_geometry: SimpleEnvironmentMeshGeometry
}

#[derive(Debug)]
pub struct SimpleEnvironmentMeshGeometry {
    vertex_type: SimpleEnvironmentVertexType,
    indices: Vec<u16>,
    vertices: Vec<SimpleEnvironmentVertex>
}

#[derive(Debug)]
enum SimpleEnvironmentMeshGeometryType {
    Simple,
    Complex
}

#[derive(FromPrimitive, ToPrimitive)]
pub enum SimpleEnvironmentQuality {
    VeryLow = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    VeryHigh = 4
}

impl SimpleEnvironment {
    pub fn read_file(path: &Path) -> io::Result<Self> {
        SimpleEnvironment::read(&mut BinaryReader::from_location(path))
    }
    pub fn read_buffer(buffer: Cursor<Vec<u8>>) -> io::Result<Self> {
        SimpleEnvironment::read(&mut BinaryReader::from_buffer(buffer))
    }
    fn read<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let magic = reader.read_string(4)?;
        if &magic != "NVR\0" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }

        let version = Version {
            major: reader.read_u16()? as u8,
            minor: reader.read_u16()? as u8
        };
        if (version.major != 8 && version.minor != 1) &&
            (version.major != 9 && version.minor != 1) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported version"));
        }

        let material_count = reader.read_u32()? as usize;
        let vertex_buffer_count = reader.read_u32()? as usize;
        let index_buffer_count = reader.read_u32()? as usize;
        let mesh_count = reader.read_u32()? as usize;
        let node_count = reader.read_u32()? as usize;

        let materials = SimpleEnvironment::read_materials(reader, material_count, version)?;
        let vertex_buffers = SimpleEnvironment::read_vertex_buffers(reader, vertex_buffer_count)?;
        let index_buffers = SimpleEnvironment::read_index_buffers(reader, index_buffer_count)?;
        let meshes = SimpleEnvironment::read_meshes(reader, mesh_count, version,
                                                    &materials, &vertex_buffers, &index_buffers)?;

        Ok(SimpleEnvironment {
            meshes
        })
    }
    fn read_materials<R: Read + Seek>(reader: &mut BinaryReader<R>, material_count: usize, version: Version)
        -> io::Result<Vec<SimpleEnvironmentMaterial>>
    {
        let mut materials: Vec<SimpleEnvironmentMaterial> = Vec::with_capacity(material_count);
        for _ in 0..material_count {
            materials.push(SimpleEnvironmentMaterial::read(reader, version)?);
        }

        Ok(materials)
    }
    fn read_vertex_buffers<R: Read + Seek>(reader: &mut BinaryReader<R>, vertex_buffer_count: usize)
        -> io::Result<Vec<SimpleEnvironmentVertexBuffer>>
    {
        let mut vertex_buffers: Vec<SimpleEnvironmentVertexBuffer> = Vec::with_capacity(vertex_buffer_count);
        for _ in 0..vertex_buffer_count {
            let size = reader.read_u32()?;
            let offset = reader.position();

            vertex_buffers.push(SimpleEnvironmentVertexBuffer{
                size,
                offset
            });

            reader.seek(SeekFrom::Current(size as i64));
        }

        Ok(vertex_buffers)
    }
    fn read_index_buffers<R: Read + Seek>(reader: &mut BinaryReader<R>, index_buffer_count: usize)
        -> io::Result<Vec<Vec<u16>>>
    {
        let mut index_buffers: Vec<Vec<u16>> = Vec::with_capacity(index_buffer_count);
        for _ in 0..index_buffer_count {
            let size = reader.read_u32()? as usize;
            let format = D3dFormat::from_u32(reader.read_u32()?)
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Invalid Index Buffer format"))?;

            match format {
                D3dFormat::D3DFMT_INDEX16 => {
                    let index_count = size / 2;
                    let mut index_buffer: Vec<u16> = Vec::with_capacity(index_count);
                    for _ in 0..index_count {
                        index_buffer.push(reader.read_u16()?);
                    }

                    index_buffers.push(index_buffer);
                },
                _ => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid Index Buffer format: {:#?}", format)))
                }
            }
        }

        Ok(index_buffers)
    }
    fn read_meshes<R: Read+ Seek>(reader: &mut BinaryReader<R>,
                                  mesh_count: usize,
                                  version: Version,
                                  materials: &Vec<SimpleEnvironmentMaterial>,
                                  vertex_buffers: &Vec<SimpleEnvironmentVertexBuffer>,
                                  index_buffers: &Vec<Vec<u16>>)
        -> io::Result<Vec<SimpleEnvironmentMesh>>
    {
        let mut meshes: Vec<SimpleEnvironmentMesh> = Vec::with_capacity(mesh_count);
        for _ in 0..mesh_count {
            meshes.push(SimpleEnvironmentMesh::read(reader, version, materials, vertex_buffers, index_buffers)?);
        }

        Ok(meshes)
    }
}

impl SimpleEnvironmentMaterial {
    fn read<R: Read + Seek>(reader: &mut BinaryReader<R>, version: Version) -> io::Result<Self> {
        let name = reader.read_padded_string(260)?;
        let material_type = SimpleEnvironmentMaterialType::from_u32(reader.read_u32()?)
            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Invalid Material Type"))?;

        if version.major == 8 && version.minor == 1 {
            let diffuse_color = LinSrgba::read_rgba_f32(reader)?;
            let diffuse_texture = reader.read_padded_string(260)?;

            let emissive_color = LinSrgba::read_rgba_f32(reader)?;
            let emissive_texture = reader.read_padded_string(260)?;

            let channels = vec![
                SimpleEnvironmentChannel::new(diffuse_color, diffuse_texture, Mat4::identity()),
                SimpleEnvironmentChannel::new(emissive_color, emissive_texture, Mat4::identity()),
                SimpleEnvironmentChannel::default(),
                SimpleEnvironmentChannel::default(),
                SimpleEnvironmentChannel::default(),
                SimpleEnvironmentChannel::default(),
                SimpleEnvironmentChannel::default(),
                SimpleEnvironmentChannel::default()
            ];

            Ok(SimpleEnvironmentMaterial {
                name,
                material_type,
                flags: SimpleEnvironmentMaterialFlags::from_bits(0u32).unwrap(),
                channels
            })
        } else {
            let flags = SimpleEnvironmentMaterialFlags::from_bits(reader.read_u32()?)
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid Material Flags")))?;

            let mut channels: Vec<SimpleEnvironmentChannel> = Vec::with_capacity(8);
            for _ in 0..8 {
                channels.push(SimpleEnvironmentChannel::read(reader)?);
            }

            Ok(SimpleEnvironmentMaterial {
                name,
                material_type,
                flags,
                channels
            })
        }
    }

    fn contains_ground_keyword(texture: &str) -> bool {
        texture.contains("_floor") ||
            texture.contains("_dirt") ||
            texture.contains("grass") ||
            texture.contains("RiverBed") ||
            texture.contains("_project") ||
            texture.contains("tile_")
    }

    fn is_ground(&self) -> bool {
        self.flags.contains(SimpleEnvironmentMaterialFlags::GROUND) &&
            SimpleEnvironmentMaterial::contains_ground_keyword(&self.channels[0].texture)
    }
}

impl SimpleEnvironmentChannel {
    pub fn new(color: LinSrgba, texture: String, transform: Mat4) -> Self {
        SimpleEnvironmentChannel {
            color,
            texture,
            transform
        }
    }

    fn read<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let color = LinSrgba::read_rgba_f32(reader)?;
        let texture = reader.read_padded_string(260)?;
        let transform = Mat4::read_row_major(reader)?;

        Ok(SimpleEnvironmentChannel {
            color,
            texture,
            transform
        })
    }
}
impl Default for SimpleEnvironmentChannel {
    fn default() -> Self {
        SimpleEnvironmentChannel {
            color: LinSrgba::new(0.0, 0.0, 0.0, 0.0),
            texture: String::with_capacity(260),
            transform: Mat4::identity()
        }
    }
}

impl SimpleEnvironmentVertexBuffer {
    fn size(&self) -> u32 { self.size }
    fn offset(&self) -> u64 { self.offset }
}

impl SimpleEnvironmentVertex {
    fn read<R: Read + Seek>(reader: &mut BinaryReader<R>, vertex_type: SimpleEnvironmentVertexType) -> io::Result<Self> {
        Ok(match vertex_type {
            SimpleEnvironmentVertexType::Default => {
                SimpleEnvironmentVertex::Default {
                    positon: Vector3::read(reader)?,
                    normal: Vector3::read(reader)?,
                    uv: Vector2::read(reader)?,
                    color: LinSrgba::read_bgra_u8(reader)?
                }
            },
            SimpleEnvironmentVertexType::Position => {
                SimpleEnvironmentVertex::Position {
                    position: Vector3::read(reader)?
                }
            },
            SimpleEnvironmentVertexType::Uv2 => {
                SimpleEnvironmentVertex::Uv2 {
                    positon: Vector3::read(reader)?,
                    normal: Vector3::read(reader)?,
                    uv0: Vector2::read(reader)?,
                    uv1: Vector2::read(reader)?,
                    color: LinSrgba::read_bgra_u8(reader)?
                }
            },
            SimpleEnvironmentVertexType::Color2 => {
                SimpleEnvironmentVertex::Color2 {
                    positon: Vector3::read(reader)?,
                    normal: Vector3::read(reader)?,
                    uv: Vector2::read(reader)?,
                    diffuse_color: LinSrgba::read_bgra_u8(reader)?,
                    emissive_color: LinSrgba::read_bgra_u8(reader)?
                }
            },
        })
    }

    fn type_from_material(material: &SimpleEnvironmentMaterial) -> SimpleEnvironmentVertexType {
        if material.material_type == SimpleEnvironmentMaterialType::FourBlend {
            SimpleEnvironmentVertexType::Uv2
        } else if material.material_type == SimpleEnvironmentMaterialType::Default &&
            material.flags.contains(SimpleEnvironmentMaterialFlags::DUAL_VERTEX_COLOR) {
            SimpleEnvironmentVertexType::Color2
        } else {
            SimpleEnvironmentVertexType::Default
        }
    }
}

impl SimpleEnvironmentVertexType {
    fn size(&self) -> usize {
        match self {
            SimpleEnvironmentVertexType::Default => 36,
            SimpleEnvironmentVertexType::Position => 12,
            SimpleEnvironmentVertexType::Uv2 => 44,
            SimpleEnvironmentVertexType::Color2 => 40,
        }
    }
}

impl SimpleEnvironmentMesh {
    fn read<R: Read + Seek>(reader: &mut BinaryReader<R>,
                            version: Version,
                            materials: &Vec<SimpleEnvironmentMaterial>,
                            vertex_buffers: &Vec<SimpleEnvironmentVertexBuffer>,
                            index_buffers: &Vec<Vec<u16>>)
        -> io::Result<Self>
    {
        let quality = reader.read_i32()?;
        let flags = if version.major == 9 && version.minor == 1 { reader.read_u32()? } else { 0 };
        let bounding_sphere = Sphere::read(reader)?;
        let bounding_box = Box3D::read(reader)?;

        let material = reader.read_u32()? as usize;
        let material = materials.get(material)
            .ok_or(io::Error::new(io::ErrorKind::NotFound, "Invalid Material ID in Mesh"))?;

        let complex_geometry = SimpleEnvironmentMeshGeometry::read(reader, version,
                                                                  material,
                                                                  SimpleEnvironmentMeshGeometryType::Complex,
                                                                  vertex_buffers, index_buffers)?;
        let simple_geometry = SimpleEnvironmentMeshGeometry::read(reader, version,
                                                                  material,
                                                                  SimpleEnvironmentMeshGeometryType::Simple,
                                                                  vertex_buffers, index_buffers)?;

        Ok(SimpleEnvironmentMesh {
            quality,
            flags,
            material: material.name.clone(),
            simple_geometry,
            complex_geometry
        })
    }
}

impl SimpleEnvironmentMeshGeometry {
    fn read<R: Read + Seek>(reader: &mut BinaryReader<R>,
                            version: Version,
                            material: &SimpleEnvironmentMaterial,
                            geometry_type: SimpleEnvironmentMeshGeometryType,
                            vertex_buffers: &Vec<SimpleEnvironmentVertexBuffer>,
                            index_buffers: &Vec<Vec<u16>>)
        -> io::Result<Self>
    {
        let vertex_buffer = reader.read_u32()? as usize;
        let first_vertex = reader.read_u32()? as usize;
        let vertex_count = reader.read_u32()? as usize;
        let return_offset = reader.position();

        let vertex_type = match geometry_type {
            SimpleEnvironmentMeshGeometryType::Simple => SimpleEnvironmentVertexType::Position,
            SimpleEnvironmentMeshGeometryType::Complex => SimpleEnvironmentVertex::type_from_material(material)
        };
        let vertex_size = vertex_type.size();
        let mut vertices: Vec<SimpleEnvironmentVertex> = Vec::with_capacity(vertex_count);

        // Seek to Vertex Buffer
        reader.seek(SeekFrom::Start(vertex_buffers[vertex_buffer].offset + (first_vertex * vertex_size) as u64))?;
        for vertex in 0..vertex_count {
            vertices.push(SimpleEnvironmentVertex::read(reader, vertex_type)?);
        }
        reader.seek(SeekFrom::Start(return_offset));

        let index_buffer = reader.read_u32()? as usize;
        let first_index = reader.read_u32()? as usize;
        let index_count = reader.read_u32()? as usize;
        let mut indices: Vec<u16> = Vec::with_capacity(index_count);
        let index_buffer = &index_buffers[index_buffer];
        for index in first_index..index_count + first_index {
            indices.push(index_buffer[index]);
        }

        // Normalize indices
        let min_index = *indices.iter().min()
            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Mesh Geometry has no indices"))?;
        for index in &mut indices {
            *index -= min_index;
        }

        Ok(SimpleEnvironmentMeshGeometry {
            vertex_type,
            vertices,
            indices
        })
    }
}
