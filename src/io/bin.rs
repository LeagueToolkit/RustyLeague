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
use std::hash::{Hash, Hasher};
use std::collections::hash_map::Entry::Occupied;
use crate::utilities::hashing::StringHasher;

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
                println!("{}", length);
                let s = reader.read_string(length)?;
                println!("{}", s);
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

    fn unpack_value_type(mut value_type: u8) -> BinValueType {
        if value_type & 128 == 128
        {
            value_type -= 128;
            value_type += 18;
        }

        return BinValueType::from_u8(value_type).expect("Invalid Value type");
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