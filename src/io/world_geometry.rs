use std::string::String;
use crate::io::binary_reader::BinaryReader;
use crate::structures::sphere::Sphere;
use crate::structures::box3d::Box3D;
use crate::structures::vector3::Vector3;
use crate::structures::vector2::Vector2;
use crate::io::binary_writer::BinaryWriter;
use std::fs::read;

pub struct WorldGeometry
{
    models: Vec<WorldGeometryModel>,
    bucket_grid: WorldGeometryBucketGrid
}

pub struct WorldGeometryModel
{
    texture: String,
    material: String,
    bounding_sphere: Sphere,
    bounding_box: Box3D,
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
    bounds: Box3D,
    vertices: Vec<Vector3>,
    indices: Vec<u16>,
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

    pub fn write(&mut self, file_location: &str)
    {
        let mut writer = BinaryWriter::from_location(file_location);

        writer.write_string("WGEO".to_string());
        writer.write_u32(5); // Version
        writer.write_u32(self.models.len() as u32); // Model Count

        let face_count =
            {
                let mut face_count = 0;

                for (i, model) in self.models.iter().enumerate()
                {
                    face_count += model.indices.len() as u32 / 3;
                }

                face_count
            };
        writer.write_u32(face_count);

        for (i, model) in self.models.iter().enumerate()
        {
            model.write(&mut writer);
        }

        self.bucket_grid.write(&mut writer);
    }

    pub fn models(&self) -> &Vec<WorldGeometryModel>
    {
        return &self.models;
    }
    pub fn bucket_grid(&self) -> &WorldGeometryBucketGrid
    {
        return &self.bucket_grid;
    }
}

impl WorldGeometryModel
{
    pub fn read(reader: &mut BinaryReader) -> Self
    {
        let texture = reader.read_padded_string(260);
        let material = reader.read_padded_string(64);
        let bounding_sphere = Sphere::read(reader);
        let bounding_box = Box3D::read(reader);
        let vertex_count = reader.read_u32();
        let index_count = reader.read_u32();

        let mut vertices: Vec<WorldGeometryVertex> = Vec::with_capacity(vertex_count as usize);
        for i in 0..vertex_count
        {
            vertices.push(WorldGeometryVertex::read(reader));
        }

        let mut indices: Vec<u32> = Vec::with_capacity(index_count as usize);
        if index_count <= 65536
        {
            for i in 0..index_count
            {
                indices.push(reader.read_u16() as u32);
            }
        }
        else
        {
            for i in 0..index_count
            {
                indices.push(reader.read_u32());
            }
        }

        WorldGeometryModel
        {
            texture,
            material,
            bounding_sphere,
            bounding_box,
            vertices,
            indices
        }
    }

    pub fn write(&self, writer: &mut BinaryWriter)
    {
        writer.write_padded_string(self.texture.clone(), 260);
        writer.write_padded_string(self.material.clone(), 64);

        self.bounding_sphere.write(writer);
        self.bounding_box.write(writer);

        writer.write(self.vertices.len() as u32);
        writer.write(self.indices.len() as u32);

        for vertex in &self.vertices
        {
            vertex.write(writer);
        }

        if self.indices.len() <= 65536
        {
            for index in &self.indices
            {
                writer.write(*index as u16);
            }
        }
        else
        {
            for index in &self.indices
            {
                writer.write(*index as u32);
            }
        }

    }
}

impl WorldGeometryVertex
{
    pub fn read(reader: &mut BinaryReader) -> Self
    {
        WorldGeometryVertex
        {
            position: Vector3::read(reader),
            uv: Vector2::read(reader)
        }
    }

    pub fn write(&self, writer: &mut BinaryWriter)
    {
        self.position.write(writer);
        self.uv.write(writer);
    }
}

impl WorldGeometryBucketGrid
{
    pub fn empty() -> Self
    {
        WorldGeometryBucketGrid
        {
            bounds: Box3D::new(Vector3::zero(), Vector3::zero()),
            vertices: Vec::new(),
            indices: Vec::new(),
            buckets: Vec::new()
        }
    }
    pub fn read(reader: &mut BinaryReader) -> Self
    {
        let min_x = reader.read_f32();
        let min_z = reader.read_f32();
        let max_x = reader.read_f32();
        let max_z = reader.read_f32();
        let max_stick_out_x = reader.read_f32();
        let max_stick_out_z = reader.read_f32();
        let bucket_size_x = reader.read_f32();
        let bucket_size_z = reader.read_f32();

        let buckets_per_side = reader.read_u16();
        let unknown = reader.read_u16();
        let vertex_count = reader.read_u32();
        let index_count = reader.read_u32();
        let mut vertices: Vec<Vector3> = Vec::with_capacity(vertex_count as usize);
        let mut indices: Vec<u16> = Vec::with_capacity(index_count as usize);
        let mut buckets: Vec<Vec<WorldGeometryBucket>> = Vec::with_capacity(buckets_per_side as usize);

        for i in 0..vertex_count
        {
            vertices.push(Vector3::read(reader));
        }
        for i in 0..index_count
        {
            indices.push(reader.read_u16());
        }
        for i in 0..buckets_per_side
        {
            let mut bucket_row: Vec<WorldGeometryBucket> = Vec::with_capacity(buckets_per_side as usize);

            for j in 0..buckets_per_side
            {
                bucket_row.push(WorldGeometryBucket::read(reader));
            }

            buckets.push(bucket_row);
        }

        WorldGeometryBucketGrid
        {
            bounds:
            {
                let min_vector = Vector3::new(min_x, std::f32::NEG_INFINITY, min_z);
                let max_vector = Vector3::new(max_x, std::f32::INFINITY, max_z);
                Box3D::new(min_vector, max_vector)
            },
            vertices,
            indices,
            buckets
        }
    }

    pub fn write(&self, writer: &mut BinaryWriter)
    {
        let bounds = self.bounds();
        let (max_stick_out_x, max_stick_out_z) = self.max_stick_out();
        let (bucket_size_x, bucket_size_z) = self.bucket_size();
        let buckets_per_side = self.buckets_per_side();

        writer.write(bounds.min.x);
        writer.write(bounds.min.z);
        writer.write(bounds.max.x);
        writer.write(bounds.max.x);
        writer.write(max_stick_out_x);
        writer.write(max_stick_out_z);
        writer.write(bucket_size_x);
        writer.write(bucket_size_z);
        writer.write(buckets_per_side);
        writer.write(0 as u16);
        writer.write(self.vertices.len() as u32);
        writer.write(self.indices.len() as u32);

        for vertex in &self.vertices
        {
            vertex.write(writer);
        }
        for index in &self.indices
        {
            writer.write(*index);
        }
        for bucketRow in &self.buckets
        {
            for bucket in bucketRow
            {
                bucket.write(writer);
            }
        }
    }

    pub fn bounds(&self) -> Box3D
    {
        if self.bounds.equals(Box3D::ZERO)
        {
            let mut min = Vector3::new(self.vertices[0].x, std::f32::NEG_INFINITY, self.vertices[0].z);
            let mut max = Vector3::new(self.vertices[0].x, std::f32::INFINITY, self.vertices[0].z);

            for vertex in &self.vertices
            {
                if min.x > vertex.x { min.x = vertex.x };
                if min.z > vertex.z { min.z = vertex.z };
                if max.x < vertex.x { max.x = vertex.x };
                if max.z < vertex.z { max.z = vertex.z };
            }

            return Box3D::new(min, max);
        }
        else
        {
            return self.bounds;
        }
    }
    pub fn max_stick_out(&self) -> (f32, f32)
    {
        let mut max_stick_out_x: f32 = 0.0;
        let mut max_stick_out_z: f32 = 0.0;

        for bucketRow in &self.buckets
        {
            for bucket in bucketRow
            {
                let (bucket_max_stick_out_x, bucket_max_stick_out_z) = bucket.max_stick_out();

                if max_stick_out_x < bucket_max_stick_out_x { max_stick_out_x = bucket_max_stick_out_x }
                if max_stick_out_z < bucket_max_stick_out_z { max_stick_out_z = bucket_max_stick_out_z }
            }
        }

        return (max_stick_out_x, max_stick_out_z);
    }
    pub fn bucket_size(&self) -> (f32, f32)
    {
        let bounds = self.bounds();
        let length_x = f32::abs(bounds.min.x) + f32::abs(bounds.max.x);
        let length_z = f32::abs(bounds.min.z) + f32::abs(bounds.max.z);
        let buckets_per_side = self.buckets_per_side() as f32;

        ( length_x / buckets_per_side, length_z / buckets_per_side )
    }
    pub fn buckets_per_side(&self) -> u16
    {
        return self.buckets.len() as u16;
    }
    pub fn vertices(&self) -> &Vec<Vector3>
    {
        return &self.vertices;
    }
    pub fn indices(&self) -> &Vec<u16>
    {
        return &self.indices;
    }
}

impl WorldGeometryBucket
{
    pub fn read(reader: &mut BinaryReader) -> Self
    {
        WorldGeometryBucket
        {
            max_stick_out_x: reader.read_f32(),
            max_stick_out_z: reader.read_f32(),
            start_index: reader.read_u32(),
            base_vertex: reader.read_u32(),
            inside_face_count: reader.read_u16(),
            sticking_out_face_count: reader.read_u16()
        }
    }

    pub fn write(&self, writer: &mut BinaryWriter)
    {
        writer.write(self.max_stick_out_x);
        writer.write(self.max_stick_out_z);
        writer.write(self.start_index);
        writer.write(self.base_vertex);
        writer.write(self.inside_face_count);
        writer.write(self.sticking_out_face_count);
    }

    pub fn max_stick_out(&self) -> (f32, f32)
    {
        return (self.max_stick_out_x, self.max_stick_out_z);
    }
}