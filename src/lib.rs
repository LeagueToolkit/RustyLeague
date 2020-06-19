pub mod io;
pub mod structures;
pub mod utilities;

#[macro_use] extern crate bitflags;
#[macro_use] extern crate num_derive;

#[cfg(test)]
mod tests
{
    use crate::io::world_geometry::WorldGeometry;
    use crate::io::release_manifest::ReleaseManifest;
    use crate::io::simple_skin::SimpleSkin;
    use std::fs::File;
    use std::path::Path;
    use std::io::{Write, Read};
    use crate::io::static_object::StaticObject;
    use crate::io::bin::{BinReader, BinWriter};
    use std::io;

    #[test]
    fn test_wgeo()
    {
        let mut world_geometry = WorldGeometry::read_from_file(Path::new("test_files/room_map11.wgeo"));

        assert!(world_geometry.is_ok());

        let mut world_geometry = world_geometry.unwrap();
        let mut models = world_geometry.models();

        assert_eq!(models.len(), 367);
    }

    #[test]
    fn test_release_manifest()
    {
        let mut release_manifest = ReleaseManifest::read_from_file(Path::new("test_files/C944A5BD0686C600.manifest"));
    }

    #[test]
    fn test_simple_skin()
    {
        let mut simple_skin = SimpleSkin::read_from_file(Path::new("test_files/aatrox.skn"));
        assert!(simple_skin.is_ok());

        let write_result = simple_skin.unwrap().write_to_file(&Path::new("test_files/aatrox_write.skn"));
        assert!(write_result.is_ok());
    }

    #[test]
    fn test_static_object()
    {
        let mut static_object = StaticObject::read_scb_from_file(Path::new("test_files/aatrox_base_w_ground_ring.scb"));

        {
            let mut file = File::create("kek.txt").unwrap();
            file.write(&format!("{:#?}", static_object).as_bytes());
        }

        assert!(static_object.is_ok())
    }

    #[test]
    fn test_bin() -> io::Result<()> {
        let mut bin = BinReader::read_tree_file(Path::new("test_files/skin0.bin"));

        assert!(bin.is_ok());

        {
            let mut file = File::create("kek.txt")?;
            file.write(&format!("{:#?}", bin).as_bytes());
        }

        if let Ok(bin) = bin {
            BinWriter::write_tree_file(&bin, &Path::new("test_files/skin0_write.bin"))?;

            let mut bin = BinReader::read_tree_file(Path::new("test_files/skin0_write.bin"))?;
            //assert!(bin.is_ok());
        }


        Ok(())
    }
}