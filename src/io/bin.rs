use std::io;
use crate::io::binary_reader::BinaryReader;
use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;
use std::fs::File;
use crate::structures::vector2::Vector2;
use crate::structures::vector4::Vector4;
use crate::structures::vector3::Vector3;
use palette::LinSrgba;
use std::collections::HashMap;
use crate::structures::color::ColorRgba;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use std::hash::{Hash, Hasher};
use crate::utilities::hashing::StringHasher;
use crate::io::binary_writer::BinaryWriter;
use std::mem;

#[derive(Debug)]
pub struct BinTree {
    dependencies: Vec<String>,
    entries: Vec<BinEntry>
}

#[derive(Debug)]
pub struct BinEntry {
    class: u32,
    path: u32,
    values: Vec<BinValue>
}

#[derive(PartialEq, Debug)]
pub enum BinValue {
    None         { name: u32 },
    Boolean      { name: u32, value: bool },
    SByte        { name: u32, value: i8 },
    Byte         { name: u32, value: u8 },
    Int16        { name: u32, value: i16 },
    UInt16       { name: u32, value: u16 },
    Int32        { name: u32, value: i32 },
    UInt32       { name: u32, value: u32 },
    Int64        { name: u32, value: i64 },
    UInt64       { name: u32, value: u64 },
    Float        { name: u32, value: f32 },
    Vector2      { name: u32, value: Vector2 },
    Vector3      { name: u32, value: Vector3 },
    Vector4      { name: u32, value: Vector4 },
    Matrix44     { name: u32, value: [[f32; 4]; 4] },
    Color        { name: u32, value: LinSrgba },
    String       { name: u32, value: String },
    Hash         { name: u32, value: u32 },
    Container    { name: u32, value: BinContainer },
    Container2   { name: u32, value: BinContainer },
    Structure    { name: u32, value: BinStructure },
    Embedded     { name: u32, value: BinStructure },
    Link         { name: u32, value: u32 },
    Optional     { name: u32, value_type: BinValueType, value: Option<Box<BinValue>> },
    Map          { name: u32, value: BinMap },
    FlagsBoolean { name: u32, value: bool }
}

#[derive(FromPrimitive, ToPrimitive, PartialEq, Copy, Clone, Debug)]
pub enum BinValueType {
    None         = 0,
    Boolean      = 1,
    SByte        = 2,
    Byte         = 3,
    Int16        = 4,
    UInt16       = 5,
    Int32        = 6,
    UInt32       = 7,
    Int64        = 8,
    UInt64       = 9,
    Float        = 10,
    Vector2      = 11,
    Vector3      = 12,
    Vector4      = 13,
    Matrix44     = 14,
    Color        = 15,
    String       = 16,
    Hash         = 17,
    Container    = 18,
    Container2   = 19,
    Structure    = 20,
    Embedded     = 21,
    Link         = 22,
    Optional     = 23,
    Map          = 24,
    FlagsBoolean = 25
}

#[derive(PartialEq, Debug)]
pub struct BinStructure {
    name: u32,
    fields: Vec<BinValue>
}

#[derive(PartialEq, Debug)]
pub struct BinContainer {
    value_type: BinValueType,
    values: Vec<BinValue>
}

#[derive(PartialEq, Debug)]
pub struct BinMap {
    key_type: BinValueType,
    value_type: BinValueType,
    map: HashMap<BinValue, BinValue>
}

pub struct BinReader;
pub struct BinWriter;

impl BinReader {
    pub fn read_tree_file(path: &Path) -> io::Result<BinTree> {
        BinReader::read_tree(&mut BinaryReader::from_file(File::open(path)?))
    }
    pub fn read_tree_buffer(buffer: Cursor<Vec<u8>>) -> io::Result<BinTree> {
        BinReader::read_tree(&mut BinaryReader::from_buffer(buffer))
    }
    pub fn read_tree<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<BinTree> {
        let magic = reader.read_string(4)?;
        if magic.as_str() != "PROP" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }

        let version = reader.read_u32()?;
        if version != 1 && version != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported version"));
        }

        let mut dependencies: Vec<String> = Vec::default();
        if version >= 2 {
            let dependency_count = reader.read_u32()?;
            for _ in 0..dependency_count {
                let length = reader.read_u16()? as usize;
                dependencies.push(reader.read_string(length)?);
            }
        }

        let mut entry_count = reader.read_u32()? as usize;
        let mut entries: Vec<BinEntry> = Vec::with_capacity(entry_count);
        let mut entry_classes: Vec<u32> = Vec::with_capacity(entry_count);

        for _ in 0..entry_count {
            entry_classes.push(reader.read_u32()?);
        }

        for i in 0..entry_count {
            entries.push(BinEntry::read(entry_classes[i], reader)?);
        }

        Ok(BinTree{
            dependencies,
            entries
        })
    }
}

impl BinWriter {
    pub fn write_tree_file(tree: &BinTree, path: &Path) -> io::Result<()> {
        BinWriter::write_tree(tree, &mut BinaryWriter::from_location(path))
    }
    pub fn write_tree_buffer(tree: &BinTree, buffer: Cursor<Vec<u8>>) -> io::Result<()> {
        BinWriter::write_tree(tree, &mut BinaryWriter::from_buffer(buffer))
    }
    pub fn write_tree<W: Write + Seek>(tree: &BinTree, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_string("PROP")?; // Magic
        writer.write_u32(2)?; // Version

        writer.write_u32(tree.dependencies().len() as u32)?;
        for dependency in tree.dependencies() {
            writer.write_u16(dependency.len() as u16)?;
            writer.write_string(dependency)?;
        }

        writer.write_u32(tree.entries().len() as u32)?;
        for entry in tree.entries() {
            writer.write_u32(entry.class())?;
        }
        for entry in tree.entries() {
            entry.write(writer)?;
        }

        Ok(())
    }

}

impl BinTree {
    pub fn dependencies(&self) -> &Vec<String> { &self.dependencies }
    pub fn entries(&self) -> &Vec<BinEntry> { &self.entries }
}

impl BinEntry {
    pub fn read<R: Read + Seek>(class: u32, reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let size = reader.read_u32()?;
        let path = reader.read_u32()?;

        let value_count  = reader.read_u16()? as usize;
        let mut values: Vec<BinValue> = Vec::with_capacity(value_count);
        for i in 0..value_count {
            values.push(BinValue::read(reader)?);
        }

        Ok(BinEntry {
            class,
            path,
            values
        })
    }

    pub(crate) fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_u32(self.size() as u32)?;
        writer.write_u32(self.path)?;

        writer.write_u16(self.values.len() as u16)?;
        for value in &self.values {
            value.write(writer)?;
        }

        Ok(())
    }

    pub fn class(&self) -> u32 { self.class }
    pub fn path(&self) -> u32 { self.path }
    pub fn values(&self) -> &Vec<BinValue> { &self.values }

    pub(crate) fn size(&self) -> usize {
        let mut size = 6usize;
        for value in &self.values {
            size += value.size(false);
        }

        size
    }
}

impl BinValue {
    pub fn read<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let name = reader.read_u32()?;
        let value_type = BinValue::unpack_value_type(reader.read_u8()?);

        BinValue::read_value(name, value_type, reader)
    }
    fn read_value<R: Read + Seek>(name: u32, value_type: BinValueType, reader: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(match value_type {
            BinValueType::None         => BinValue::None         { name },
            BinValueType::Boolean      => BinValue::Boolean      { name, value: reader.read_u8()? != 0 },
            BinValueType::SByte        => BinValue::SByte        { name, value: reader.read_i8()?},
            BinValueType::Byte         => BinValue::Byte         { name, value: reader.read_u8()?},
            BinValueType::Int16        => BinValue::Int16        { name, value: reader.read_i16()?},
            BinValueType::UInt16       => BinValue::UInt16       { name, value: reader.read_u16()?},
            BinValueType::Int32        => BinValue::Int32        { name, value: reader.read_i32()?},
            BinValueType::UInt32       => BinValue::UInt32       { name, value: reader.read_u32()?},
            BinValueType::Int64        => BinValue::Int64        { name, value: reader.read_i64()?},
            BinValueType::UInt64       => BinValue::UInt64       { name, value: reader.read_u64()?},
            BinValueType::Float        => BinValue::Float        { name, value: reader.read_f32()?},
            BinValueType::Vector2      => BinValue::Vector2      { name, value: Vector2::read(reader)?},
            BinValueType::Vector3      => BinValue::Vector3      { name, value: Vector3::read(reader)?},
            BinValueType::Vector4      => BinValue::Vector4      { name, value: Vector4::read(reader)?},
            BinValueType::Matrix44     => BinValue::Matrix44     { name, value: {
                let mut matrix = [[0.0, 0.0, 0.0, 0.0]; 4];

                for row in 0..4usize {
                    for column in 0..4usize {
                        matrix[row][column] = reader.read_f32()?;
                    }
                }

                matrix
            }},
            BinValueType::Color        => BinValue::Color        { name, value: LinSrgba::read_rgba_u8(reader)?},
            BinValueType::String       => BinValue::String       { name, value: {
                let length = reader.read_u16()? as usize;
                let s = reader.read_string(length)?;
                s
            }},
            BinValueType::Hash         => BinValue::Hash         { name, value: reader.read_u32()?},
            BinValueType::Container    => BinValue::Container    { name, value: BinContainer::read(reader)?},
            BinValueType::Container2   => BinValue::Container2   { name, value: BinContainer::read(reader)?},
            BinValueType::Structure    => BinValue::Structure    { name, value: BinStructure::read(reader)?},
            BinValueType::Embedded     => BinValue::Embedded     { name, value: BinStructure::read(reader)?},
            BinValueType::Link         => BinValue::Link         { name, value: reader.read_u32()?},
            BinValueType::Optional     => {
                let optional = BinValue::read_optional(reader)?;

                BinValue::Optional { name, value_type: optional.0, value: optional.1 }
            },
            BinValueType::Map          => BinValue::Map          { name, value: BinMap::read(reader)?},
            BinValueType::FlagsBoolean => BinValue::FlagsBoolean { name, value: reader.read_u8()? != 0},
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Value type"))
        })
    }
    fn read_optional<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<(BinValueType, Option<Box<BinValue>>)> {
        let value_type = BinValue::unpack_value_type(reader.read_u8()?);
        let is_some = reader.read_u8()? != 0;

        if is_some {
            Ok((value_type, Some(Box::new(BinValue::read_value(0, value_type, reader)?))))
        } else {
            Ok((value_type, None))
        }
    }

    pub(crate) fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>, ) -> io::Result<()> {
        match self {
            BinValue::None         { name     } => { writer.write_u32(*name)?; },
            BinValue::Boolean      { name, .. } => { writer.write_u32(*name)?; },
            BinValue::SByte        { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Byte         { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Int16        { name, .. } => { writer.write_u32(*name)?; },
            BinValue::UInt16       { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Int32        { name, .. } => { writer.write_u32(*name)?; },
            BinValue::UInt32       { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Int64        { name, .. } => { writer.write_u32(*name)?; },
            BinValue::UInt64       { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Float        { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Vector2      { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Vector3      { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Vector4      { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Matrix44     { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Color        { name, .. } => { writer.write_u32(*name)?; },
            BinValue::String       { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Hash         { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Container    { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Container2   { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Structure    { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Embedded     { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Link         { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Optional     { name, .. } => { writer.write_u32(*name)?; },
            BinValue::Map          { name, .. } => { writer.write_u32(*name)?; },
            BinValue::FlagsBoolean { name, .. } => { writer.write_u32(*name)?; }
        };

        writer.write_u8(BinValue::pack_value_type(self.value_type()))?;
        self.write_value(writer)?;

        Ok(())
    }
    pub(crate) fn write_value<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        match self {
            BinValue::None         { name } => { },
            BinValue::Boolean      { name, value } => { writer.write_u8(*value as u8)?; },
            BinValue::SByte        { name, value } => { writer.write_i8(*value)?; },
            BinValue::Byte         { name, value } => { writer.write_u8(*value)?; },
            BinValue::Int16        { name, value } => { writer.write_i16(*value)?; },
            BinValue::UInt16       { name, value } => { writer.write_u16(*value)?; },
            BinValue::Int32        { name , value} => { writer.write_i32(*value)?; },
            BinValue::UInt32       { name, value } => { writer.write_u32(*value)?; },
            BinValue::Int64        { name , value} => { writer.write_i64(*value)?; },
            BinValue::UInt64       { name, value } => { writer.write_u64(*value)?; },
            BinValue::Float        { name , value} => { writer.write_f32(*value)?; },
            BinValue::Vector2      { name , value} => { value.write(writer)?; },
            BinValue::Vector3      { name , value} => { value.write(writer)?; },
            BinValue::Vector4      { name , value} => { value.write(writer)?; },
            BinValue::Matrix44     { name, value } => {
                for row in value.iter() {
                    for entry in row.iter() {
                        writer.write_f32(*entry)?;
                    }
                }
            },
            BinValue::Color        { name, value } => { value.write_rgba_u8(writer)?; },
            BinValue::String       { name, value } => {
                writer.write_u16(value.len() as u16)?;
                writer.write_string(value)?;
            },
            BinValue::Hash         { name, value } => { writer.write_u32(*value)?; },
            BinValue::Container    { name, value } => { value.write(writer)?; },
            BinValue::Container2   { name, value } => { value.write(writer)?; },
            BinValue::Structure    { name, value } => { value.write(writer)?; },
            BinValue::Embedded     { name, value } => { value.write(writer)?; },
            BinValue::Link         { name, value } => { writer.write_u32(*value)?; },
            BinValue::Optional     { name, value_type, value } => {
                writer.write_u8(BinValue::pack_value_type(*value_type))?;
                writer.write_u8(value.is_some() as u8)?;

                if let Some(option) = value {
                    option.write_value(writer)?;
                }
            },
            BinValue::Map          { name, value } => { value.write(writer)?; },
            BinValue::FlagsBoolean { name, value } => { writer.write_u8(*value as u8)?; }
        };

        Ok(())
    }

    fn unpack_value_type(mut value_type: u8) -> BinValueType {
        if value_type & 128 == 128
        {
            value_type -= 128;
            value_type += 18;
        }

        return BinValueType::from_u8(value_type).expect("Invalid Value type");
    }
    fn pack_value_type(value_type: BinValueType) -> u8 {
        let mut value_type = value_type.to_u8().expect("Invalid Value Type");

        if value_type >= 18
        {
            value_type = (value_type - 18) + 128;
        }

        value_type
    }

    pub fn value_type(&self) -> BinValueType {
        match self {
            BinValue::None         { .. } => { BinValueType::None         },
            BinValue::Boolean      { .. } => { BinValueType::Boolean      },
            BinValue::SByte        { .. } => { BinValueType::SByte        },
            BinValue::Byte         { .. } => { BinValueType::Byte         },
            BinValue::Int16        { .. } => { BinValueType::Int16        },
            BinValue::UInt16       { .. } => { BinValueType::UInt16       },
            BinValue::Int32        { .. } => { BinValueType::Int32        },
            BinValue::UInt32       { .. } => { BinValueType::UInt32       },
            BinValue::Int64        { .. } => { BinValueType::Int64        },
            BinValue::UInt64       { .. } => { BinValueType::UInt64       },
            BinValue::Float        { .. } => { BinValueType::Float        },
            BinValue::Vector2      { .. } => { BinValueType::Vector2      },
            BinValue::Vector3      { .. } => { BinValueType::Vector3      },
            BinValue::Vector4      { .. } => { BinValueType::Vector4      },
            BinValue::Matrix44     { .. } => { BinValueType::Matrix44     },
            BinValue::Color        { .. } => { BinValueType::Color        },
            BinValue::String       { .. } => { BinValueType::String       },
            BinValue::Hash         { .. } => { BinValueType::Hash         },
            BinValue::Container    { .. } => { BinValueType::Container    },
            BinValue::Container2   { .. } => { BinValueType::Container2   },
            BinValue::Structure    { .. } => { BinValueType::Structure    },
            BinValue::Embedded     { .. } => { BinValueType::Embedded     },
            BinValue::Link         { .. } => { BinValueType::Link         },
            BinValue::Optional     { .. } => { BinValueType::Optional     },
            BinValue::Map          { .. } => { BinValueType::Map          },
            BinValue::FlagsBoolean { .. } => { BinValueType::FlagsBoolean }
        }
    }

    pub(crate) fn size(&self, is_simple: bool) -> usize {
        let type_size = if is_simple { 0 } else { 5usize };
        let value_size = match self {
            BinValue::None         { name } => { 0 },
            BinValue::Boolean      { name, value } => { mem::size_of::<u8>() },
            BinValue::SByte        { name, value } => { mem::size_of::<i8>() },
            BinValue::Byte         { name, value } => { mem::size_of::<u8>() },
            BinValue::Int16        { name, value } => { mem::size_of::<i16>() },
            BinValue::UInt16       { name, value } => { mem::size_of::<u16>() },
            BinValue::Int32        { name , value} => { mem::size_of::<i32>() },
            BinValue::UInt32       { name, value } => { mem::size_of::<u32>() },
            BinValue::Int64        { name , value} => { mem::size_of::<i64>() },
            BinValue::UInt64       { name, value } => { mem::size_of::<u64>() },
            BinValue::Float        { name , value} => { mem::size_of::<f32>() },
            BinValue::Vector2      { name , value} => { mem::size_of::<Vector2>() },
            BinValue::Vector3      { name , value} => { mem::size_of::<Vector3>() },
            BinValue::Vector4      { name , value} => { mem::size_of::<Vector4>() },
            BinValue::Matrix44     { name, value } => { mem::size_of::<[[f32; 4]; 4]>() },
            BinValue::Color        { name, value } => { 4 },
            BinValue::String       { name, value } => { value.len() + 2 },
            BinValue::Hash         { name, value } => { mem::size_of::<u32>() },
            BinValue::Container    { name, value } => { value.size() },
            BinValue::Container2   { name, value } => { value.size() },
            BinValue::Structure    { name, value } => { value.size() },
            BinValue::Embedded     { name, value } => { value.size() },
            BinValue::Link         { name, value } => { mem::size_of::<u32>() },
            BinValue::Optional     { name, value_type, value } => {
                2 + if let Some(option) = value {
                    option.size(true)
                } else { 0 }
            },
            BinValue::Map          { name, value } => { value.size() },
            BinValue::FlagsBoolean { name, value } => { mem::size_of::<u8>() }
        };

        type_size + value_size
    }
}

impl BinStructure {
    pub fn read<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let name= reader.read_u32()?;
        if name == 0 {
            Ok(BinStructure {
                name,
                fields: Vec::default()
            })
        } else {
            let size = reader.read_u32()?;

            let field_count = reader.read_u16()? as usize;
            let mut fields: Vec<BinValue> = Vec::with_capacity(field_count);
            for _ in 0..field_count {
                fields.push(BinValue::read(reader)?);
            }

            Ok(BinStructure {
                name,
                fields
            })
        }
    }

    pub(crate) fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_u32(self.name)?;
        if self.name != 0 {
            writer.write_u32(self.content_size() as u32)?;

            writer.write_u16(self.fields.len() as u16)?;
            for field in &self.fields {
                field.write(writer)?;
            }
        }

        Ok(())
    }

    pub(crate) fn size(&self) -> usize {
        if self.name == 0 { 4 }
        else { 4 + 4 + self.content_size() }
    }

    pub(crate) fn content_size(&self) -> usize {
        let mut size = 2usize;
        for field in &self.fields {
            size += field.size(false);
        }

        size
    }
}

impl BinContainer {
    pub fn read<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let value_type = BinValue::unpack_value_type(reader.read_u8()?);
        let size = reader.read_u32()?;

        let value_count = reader.read_u32()? as usize;
        let mut values: Vec<BinValue> = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            values.push(BinValue::read_value(0, value_type, reader)?);
        }

        Ok(BinContainer {
            value_type,
            values
        })
    }

    pub(crate) fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_u8(BinValue::pack_value_type(self.value_type))?;
        writer.write_u32(self.content_size() as u32)?;

        writer.write_u32(self.values.len() as u32)?;
        for value in &self.values {
            value.write_value(writer)?;
        }

        Ok(())
    }

    pub(crate) fn size(&self) -> usize {
        1 + 4 + self.content_size()
    }

    pub(crate) fn content_size(&self) -> usize {
        let mut size = 4usize;
        for value in &self.values {
            size += value.size(true);
        }

        size
    }
}

impl BinMap {
    pub fn read<R: Read + Seek>(reader: &mut BinaryReader<R>) -> io::Result<Self> {
        let key_type = BinValue::unpack_value_type(reader.read_u8()?);
        let value_type = BinValue::unpack_value_type(reader.read_u8()?);
        let size = reader.read_u32()?;

        let entry_count = reader.read_u32()? as usize;
        let mut map: HashMap<BinValue, BinValue> = HashMap::with_capacity(entry_count);
        for _ in 0..entry_count {
            map.insert(BinValue::read_value(0, key_type, reader)?, BinValue::read_value(0, value_type, reader)?);
        }

        Ok(BinMap {
            key_type,
            value_type,
            map
        })
    }

    pub(crate) fn write<W: Write + Seek>(&self, writer: &mut BinaryWriter<W>) -> io::Result<()> {
        writer.write_u8(BinValue::pack_value_type(self.key_type))?;
        writer.write_u8(BinValue::pack_value_type(self.value_type))?;
        writer.write_u32(self.content_size() as u32)?;

        writer.write_u32(self.map.len() as u32)?;
        for (key, value) in &self.map {
            key.write_value(writer)?;
            value.write_value(writer)?;
        }

        Ok(())
    }

    pub(crate) fn size(&self) -> usize {
        1 + 1 + 4 + self.content_size()
    }

    pub(crate) fn content_size(&self) -> usize {
        let mut size = 4usize;
        for (key, value) in &self.map {
            size += key.size(true) + value.size(true);
        }

        size
    }
}

impl Hash for BinValue {
    fn hash<H: Hasher>(&self, state: &mut H) where H: StringHasher {
         match self {
             BinValue::Boolean { name, value } => { state.write_u32(*name); state.write_u8(*value as u8); },
             BinValue::SByte { name, value } => { state.write_u32(*name); state.write_i8(*value as i8); },
             BinValue::Byte { name, value } => { state.write_u32(*name); state.write_u8(*value as u8); },
             BinValue::Int16 { name, value } => { state.write_u32(*name); state.write_i16(*value as i16); },
             BinValue::UInt16 { name, value } => { state.write_u32(*name); state.write_u16(*value as u16); },
             BinValue::Int32 { name, value } => { state.write_u32(*name); state.write_i32(*value as i32); },
             BinValue::UInt32 { name, value } => { state.write_u32(*name); state.write_u32(*value as u32); },
             BinValue::Int64 { name, value } => { state.write_u32(*name); state.write_i64(*value as i64); },
             BinValue::UInt64 { name, value } => { state.write_u32(*name); state.write_u64(*value as u64); },
             BinValue::String { name, value} => { state.write_u32(*name); state.write_string_lc(value); },
             BinValue::Hash { name, value } => { state.write_u32(*name); state.write_u32(*value as u32); },
             _ => state.write_u8(0) // Hack
         }
    }
}

impl Eq for BinValue { }