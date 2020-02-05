use std::io::{Read, Seek, Cursor, SeekFrom, Write};
use crate::io::binary_reader::BinaryReader;
use std::borrow::Borrow;
use std::fs::{read, File};
use std::path::Path;

extern crate zstd;

#[derive(Debug)]
pub struct ReleaseManifest
{
    release_id: u64,
    bundles: Vec<ReleaseManifestBundle>,
    languages: Vec<ReleaseManifestLanguage>,
    files: Vec<ReleaseManifestFile>,
    directories: Vec<ReleaseManifestDirectory>,
}

#[derive(Debug)]
struct ReleaseManifestBody
{
    bundles: Vec<ReleaseManifestBundle>,
    languages: Vec<ReleaseManifestLanguage>,
    files: Vec<ReleaseManifestFile>,
    directories: Vec<ReleaseManifestDirectory>
}

#[derive(Debug)]
pub struct ReleaseManifestBundle
{
    id: u64,
    chunks: Vec<ReleaseManifestBundleChunk>
}

#[derive(Debug)]
pub struct ReleaseManifestBundleChunk
{
    compressed_size: u32,
    uncompressed_size: u32,
    id: u64
}

#[derive(Debug)]
pub struct ReleaseManifestLanguage
{
    id: u32,
    name: String
}

#[derive(Debug)]
pub struct ReleaseManifestFile
{
    name: String,
    link: String,
    id: u64,
    directory_id: u64,
    size: u32,
    language_ids: Vec<u32>,
    chunk_ids: Vec<u64>
}

#[derive(Debug)]
pub struct ReleaseManifestDirectory
{
    name: String,
    id: u64,
    parent_id: u64
}

impl ReleaseManifest
{
    pub fn read_from_file(file_location: &str) -> Self
    {
        ReleaseManifest::read(&mut BinaryReader::from_location(file_location))
    }
    pub fn read_from_buffer(buffer: Cursor<Vec<u8>>) -> Self
    {
        ReleaseManifest::read(&mut BinaryReader::from_buffer(buffer))
    }
    pub fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> Self
    {
        let magic = reader.read_string(4);
        if &magic != "RMAN"
        {
            panic!("Not a valid Release Manifest file");
        }

        let major = reader.read_u8();
        let minor = reader.read_u8();
        if major != 2 || minor != 0
        {
            panic!(format!("Unsupported version: {}.{}", major, minor));
        }

        let unknown = reader.read_u8();
        let signature_type = reader.read_u8();
        let content_offset = reader.read_u32();
        let compressed_content_size = reader.read_u32();
        let release_id = reader.read_u64();
        let uncompressed_content_size = reader.read_u32();

        reader.seek(SeekFrom::Start(content_offset as u64));
        let mut compressed_manifest_body = Cursor::new(reader.read_bytes(compressed_content_size as usize));
        let mut uncompressed_manifest_body = Cursor::new(Vec::with_capacity(uncompressed_content_size as usize));

        zstd::stream::copy_decode(&mut compressed_manifest_body, &mut uncompressed_manifest_body);

        let signature: Vec<u8> = reader.read_bytes(256);
        let body = ReleaseManifest::read_body(&mut uncompressed_manifest_body);

        ReleaseManifest
        {
            release_id,
            bundles: body.bundles,
            languages: body.languages,
            files: body.files,
            directories: body.directories
        }
    }
    fn read_body(body_buffer: &mut Cursor<Vec<u8>>) -> ReleaseManifestBody
    {
        let mut reader = BinaryReader::from_buffer(body_buffer.to_owned());
        reader.seek(SeekFrom::Start(0));

        let header_offset = reader.read_u32() as u64;

        reader.seek(SeekFrom::Start(header_offset));
        let offset_table_offset = reader.read_u32() as u64;
        let bundles_offset = reader.position() + reader.read_u32() as u64;
        let languages_offset = reader.position() + reader.read_u32() as u64;
        let files_offset = reader.position() + reader.read_u32() as u64;
        let directories_offset = reader.position() + reader.read_u32() as u64;
        let key_header_offset = reader.position() + reader.read_u32() as u64;
        let unknown_offset = reader.position() + reader.read_u32() as u64;

        {
            let mut file = File::create(Path::new("xd")).unwrap();
            file.write( &u32::to_le_bytes(header_offset as u32));
            file.write( &u32::to_le_bytes(offset_table_offset as u32));
            file.write( &u32::to_le_bytes(bundles_offset as u32));
            file.write( &u32::to_le_bytes(languages_offset as u32));
            file.write( &u32::to_le_bytes(files_offset as u32));
            file.write( &u32::to_le_bytes(directories_offset as u32));
        }

        ReleaseManifestBody
        {
            bundles: ReleaseManifest::read_body_bundles(bundles_offset, &mut reader),
            languages: ReleaseManifest::read_body_languages(languages_offset, &mut reader),
            files: ReleaseManifest::read_body_files(files_offset, &mut reader),
            directories: ReleaseManifest::read_body_directories(directories_offset, &mut reader)
        }
    }
    fn read_body_bundles<T: Read + Seek>(offset: u64, reader: &mut BinaryReader<T>) -> Vec<ReleaseManifestBundle>
    {
        reader.seek(SeekFrom::Start(offset));

        let bundle_count = reader.read_u32();
        let mut bundles: Vec<ReleaseManifestBundle> = Vec::with_capacity(bundle_count as usize);
        for i in 0..bundle_count
        {
            let bundle_offset = reader.read_u32() as u64;
            let return_offset = reader.position();

            reader.seek(SeekFrom::Start(bundle_offset + return_offset - 4));
            bundles.push(ReleaseManifestBundle::read(reader));
            reader.seek(SeekFrom::Start(return_offset));
        }

        return bundles;
    }
    fn read_body_languages<T: Read + Seek>(offset: u64, reader: &mut BinaryReader<T>) -> Vec<ReleaseManifestLanguage>
    {
        reader.seek(SeekFrom::Start(offset));

        let language_count = reader.read_u32();
        let mut languages: Vec<ReleaseManifestLanguage> = Vec::with_capacity(language_count as usize);
        for i in 0..language_count
        {
            let language_offset = reader.read_u32() as u64;
            let return_offset = reader.position();

            reader.seek(SeekFrom::Start(language_offset + return_offset - 4));
            languages.push(ReleaseManifestLanguage::read(reader));
            reader.seek(SeekFrom::Start(return_offset));
        }

        return languages;
    }
    fn read_body_files<T: Read + Seek>(offset: u64, reader: &mut BinaryReader<T>) -> Vec<ReleaseManifestFile>
    {
        reader.seek(SeekFrom::Start(offset));

        let file_count = reader.read_u32();
        let mut files: Vec<ReleaseManifestFile> = Vec::with_capacity(file_count as usize);
        for i in 0..file_count
        {
            let file_offset = reader.read_u32() as u64;
            let return_offset = reader.position();

            reader.seek(SeekFrom::Start(file_offset + return_offset - 4));
            files.push(ReleaseManifestFile::read(reader));
            reader.seek(SeekFrom::Start(return_offset));
        }

        return files;
    }
    fn read_body_directories<T: Read + Seek>(offset: u64, reader: &mut BinaryReader<T>) -> Vec<ReleaseManifestDirectory>
    {
        reader.seek(SeekFrom::Start(offset));

        let directory_count = reader.read_u32();
        let mut directories: Vec<ReleaseManifestDirectory> = Vec::with_capacity(directory_count as usize);
        for i in 0..directory_count
        {
            let directory_offset = reader.read_u32() as u64;
            let return_offset = reader.position();

            reader.seek(SeekFrom::Start(directory_offset + return_offset - 4));
            directories.push(ReleaseManifestDirectory::read(reader));
            reader.seek(SeekFrom::Start(return_offset));
        }

        return directories;
    }

    pub fn release_id(&self) -> u64 { self.release_id }
    pub fn bundles(&self) -> &Vec<ReleaseManifestBundle> { &self.bundles }
    pub fn languages(&self) -> &Vec<ReleaseManifestLanguage> { &self.languages }
    pub fn files(&self) -> &Vec<ReleaseManifestFile> { &self.files }
    pub fn directories(&self) -> &Vec<ReleaseManifestDirectory> { &self.directories }
}

impl ReleaseManifestBundle
{
    fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> Self
    {
        reader.read_u32(); //read offset table offset
        let header_size = reader.read_u32();
        let id = reader.read_u64();



        //Skip rest of header
        reader.seek(SeekFrom::Current((header_size - 12) as i64));

        let chunk_count = reader.read_u32();
        let mut chunks: Vec<ReleaseManifestBundleChunk> = Vec::with_capacity(chunk_count as usize);
        for i in 0..chunk_count
        {
            let chunk_offset = reader.read_u32() as u64;
            let return_offset = reader.position();

            reader.seek(SeekFrom::Start(chunk_offset + return_offset - 4));
            chunks.push(ReleaseManifestBundleChunk::read(reader));
            reader.seek(SeekFrom::Start(return_offset));
        }

        ReleaseManifestBundle
        {
            id,
            chunks
        }
    }

    pub fn id(&self) -> u64 { self.id }
    pub fn chunks(&self) -> &Vec<ReleaseManifestBundleChunk> { &self.chunks }
}

impl ReleaseManifestBundleChunk
{
    fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> Self
    {
        reader.read_u32(); // offset table offset

        ReleaseManifestBundleChunk
        {
            compressed_size: reader.read_u32(),
            uncompressed_size: reader.read_u32(),
            id: reader.read_u64()
        }
    }

    pub fn id(&self) -> u64 { self.id }
    pub fn compressed_size(&self) -> u32 { self.compressed_size }
    pub fn uncompressed_size(&self) -> u32 { self.uncompressed_size }
}

impl ReleaseManifestLanguage
{
    fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> Self
    {
        reader.read_u32(); //offset table offset
        let id = reader.read_u32();

        let name_offset = reader.read_u32() as u64;
        let return_offset = reader.position();

        reader.seek(SeekFrom::Start(name_offset + return_offset - 4));
        let name = reader.read_sized_string();
        reader.seek(SeekFrom::Start(return_offset));

        ReleaseManifestLanguage
        {
            id,
            name
        }
    }

    pub fn id(&self) -> u32 { self.id }
    pub fn name(&self) -> String { self.name.clone() }
}

impl ReleaseManifestFile
{
    fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> Self
    {
        reader.read_u32(); //offset table offset

        let file_offset = reader.position();
        let mut flags = reader.read_u32();
        let file_type = flags >> 24;
        let mut name_offset = 0u64;

        if flags == 0x00010200 || file_type != 0
        {
            name_offset = reader.read_u32() as u64;
        }
        else
        {
            name_offset = (flags - 4) as u64;
        }

        let structure_size = reader.read_u32();
        let link_offset = reader.read_u32() as u64;
        let id = reader.read_u64();
        let directory_id: u64 = if structure_size > 28 { reader.read_u64() } else { 0 };
        let size = reader.read_u32();
        let permissions = reader.read_u32(); //?
        let language_ids = if structure_size > 36
        {
            let mut language_ids: Vec<u32> = Vec::with_capacity(64);
            let language_mask = reader.read_u64();
            for i in 0..64
            {
                if (language_mask & (1u64 << i)) != 0
                {
                    language_ids.push(i as u32);
                }
            }

            language_ids
        }
        else { Vec::default() };

        let unknown1 = reader.read_u32();
        let permissions = reader.read_u32(); //??

        let chunk_count = reader.read_u32();
        let mut chunk_ids: Vec<u64> = Vec::with_capacity(chunk_count as usize);
        for i in 0..chunk_count
        {
            chunk_ids.push(reader.read_u64());
        }

        let return_offset = reader.position();

        reader.seek(SeekFrom::Start(file_offset + name_offset + 4));
        let name = reader.read_sized_string();

        reader.seek(SeekFrom::Start(file_offset + link_offset + 12));
        let link = reader.read_sized_string();

        ReleaseManifestFile
        {
            name,
            link,
            id,
            directory_id,
            size: size,
            language_ids,
            chunk_ids
        }
    }

    pub fn name(&self) -> String { self.name.clone() }
    pub fn link(&self) -> String { self.link.clone() }
    pub fn id(&self) -> u64 { self.id }
    pub fn directory_id(&self) -> u64 { self.directory_id }
    pub fn size(&self) -> u32 { self.size }
    pub fn language_ids(&self) -> &Vec<u32> { &self.language_ids }
    pub fn chunk_ids(&self) -> &Vec<u64> { &self.chunk_ids }
}

impl ReleaseManifestDirectory
{
    fn read<T: Read + Seek>(reader: &mut BinaryReader<T>) -> Self
    {
        let offset_table_offset = reader.read_u32() as u64;
        let directory_offset = reader.position();

        reader.seek(SeekFrom::Start(directory_offset - offset_table_offset));

        let id_offset = reader.read_u16() as u64;
        let parent_id_offset = reader.read_u16() as u64;

        reader.seek(SeekFrom::Start(directory_offset));

        let name_offset = reader.read_u32() as u64;
        let id = if directory_offset > 0 { reader.read_u64() } else { 0 };
        let parent_id = if parent_id_offset > 0 { reader.read_u64() } else { 0 };

        reader.seek(SeekFrom::Start(directory_offset + name_offset));
        let name = reader.read_sized_string();

        ReleaseManifestDirectory
        {
            name,
            id,
            parent_id
        }
    }

    pub fn name(&self) -> String { self.name.clone() }
    pub fn id(&self) -> u64 { self.id }
    pub fn parent_id(&self) -> u64 { self.parent_id }
}