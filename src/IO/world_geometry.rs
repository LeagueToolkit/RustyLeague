use std::string::String;
use crate::io::binary_reader::BinaryReader;
use crate::structures::r3d_sphere::R3DSphere;
use crate::structures::r3d_box::R3DBox;
use crate::structures::vector3::Vector3;
use crate::structures::vector2::Vector2;

pub struct WorldGeometry
{
    version: u32,
    models: Vec<WorldGeometryModel>,
    bucket_grid: WorldGeometryBucketGrid
}

pub struct WorldGeometryModel
{
    texture: String,
    material: String,
    bounding_sphere: R3DSphere,
    bounding_box: R3DBox,
    vertices: Vec<WorldGeometryVertex>,
    indices: Vec<u32>
}

pub struct WorldGeometryVertex
{
    position: Vector3,
    uv: Vector2
}

pub struct WorldGeometryBucketGrid
{
    bounds: R3DBox,
    center: Vector3,
    buckets: Vec<Vec<WorldGeometryBucket>>
}

pub struct WorldGeometryBucket
{
    max_stick_out_x: f32,
    max_stick_out_z: f32,
    start_index: u32,
    base_vertex: u32,
    inside_face_count: u16,
    sticking_out_face_count: u16
}

impl WorldGeometry
{
    pub fn read(file_location: &str) -> Self
    {
        let mut file= BinaryReader::from_location(file_location);

        let magic: String = file.read_string(4);
        if &magic != "WGEO"
        {
            panic!("Not a WGEO file")
        }

        let version: u32 = file.read_u32();
        if version != 5 && version != 4
        {
            panic!("Unsupported version");
        }

        WorldGeometry
        {
            version,
            models:
            {
                let model_count: u32 = file.read_u32();
                let face_count: u32 = file.read_u32();
                let mut models: Vec<WorldGeometryModel> = Vec::with_capacity(model_count as usize);

                for i in 0..model_count
                {
                    models.push(WorldGeometryModel::read(&mut file));
                }

                models
            },
            bucket_grid:
            {
                if version == 5
                {
                    WorldGeometryBucketGrid::read(&mut file)
                }
                else
                {
                    WorldGeometryBucketGrid::empty()
                }
            }
        }
    }

    pub fn write(&mut self)
    {

    }
}

impl WorldGeometryModel
{
    pub fn read(file: &mut BinaryReader) -> Self
    {
        let texture =
            {
                let string = file.read_string(260);
                string[0..string.find('\0').unwrap()].to_string()
            };

        let material =
            {
                let string = file.read_string(64);
                string[0..string.find('\0').unwrap()].to_string()
            };

        let bounding_sphere = R3DSphere::read(file);
        let bounding_box = R3DBox::read(file);
        let vertex_count = file.read_u32();
        let index_count = file.read_u32();

        let mut vertices: Vec<WorldGeometryVertex> = Vec::with_capacity(vertex_count as usize);
        for i in 0..vertex_count
        {
            vertices.push(WorldGeometryVertex::read(file));
        }

        let mut indices: Vec<u32> = Vec::with_capacity(index_count as usize);
        if index_count <= 65536
        {
            for i in 0..index_count
            {
                indices.push(file.read_u16() as u32);
            }
        }
        else
        {
            for i in 0..index_count
            {
                indices.push(file.read_u32());
            }
        }

        WorldGeometryModel
        {
            texture,
            material,
            bounding_sphere: bounding_sphere,
            bounding_box: bounding_box,
            vertices,
            indices
        }
    }

}

impl WorldGeometryVertex
{
    pub fn read(file: &mut BinaryReader) -> Self
    {
        WorldGeometryVertex
        {
            position: Vector3::read(file),
            uv: Vector2::read(file)
        }
    }
}

impl WorldGeometryBucketGrid
{
    pub fn empty() -> Self
    {
        WorldGeometryBucketGrid
        {
            bounds: R3DBox::new(Vector3::empty(), Vector3::empty()),
            center: Vector3::empty(),
            buckets: Vec::new()
        }
    }

    pub fn read(file: &mut BinaryReader) -> Self
    {
        let min_x = file.read_f32();
        let min_z = file.read_f32();
        let max_x = file.read_f32();
        let max_z = file.read_f32();
        let center_x = file.read_f32();
        let center_z = file.read_f32();
        let min_y = file.read_f32();
        let max_y = file.read_f32();

        let buckets_per_side = file.read_u32();
        let vertex_count = file.read_u32();
        let index_count = file.read_u32();
        let mut buckets: Vec<Vec<WorldGeometryBucket>> = Vec::with_capacity(buckets_per_side as usize);

        for i in 0..buckets_per_side
        {
            let mut bucket_row: Vec<WorldGeometryBucket> = Vec::with_capacity(buckets_per_side as usize);

            for j in 0..buckets_per_side
            {
                bucket_row.push(WorldGeometryBucket::read(file));
            }

            buckets.push(bucket_row);
        }

        WorldGeometryBucketGrid
        {
            bounds:
            {
                let min_vector = Vector3::new(min_x, min_y, min_z);
                let max_vector = Vector3::new(max_x, max_y, max_z);
                R3DBox::new(min_vector, max_vector)
            },
            center: Vector3::new(center_x, (min_y + max_y) / 2.0, center_z),
            buckets
        }
    }

}

impl WorldGeometryBucket
{
    pub fn read(file: &mut BinaryReader) -> Self
    {
        WorldGeometryBucket
        {
            max_stick_out_x: file.read_f32(),
            max_stick_out_z: file.read_f32(),
            start_index: file.read_u32(),
            base_vertex: file.read_u32(),
            inside_face_count: file.read_u16(),
            sticking_out_face_count: file.read_u16()
        }
    }
}


