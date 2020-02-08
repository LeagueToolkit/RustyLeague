pub mod io;
pub mod structures;

#[cfg(test)]
mod tests
{
    use crate::io::world_geometry::WorldGeometry;
    use crate::io::release_manifest::ReleaseManifest;
    use crate::io::simple_skin::SimpleSkin;
    use std::fs::File;
    use std::path::Path;
    use std::io::{Write, Read};

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

        let s = format!("{:#?}", simple_skin);

        {
            let mut file = File::create("kek.txt").unwrap();
            file.write(&s.as_bytes());
        }

        assert!(simple_skin.is_ok());

        let write_result = simple_skin.unwrap().write_to_file(&Path::new("test_files/aatrox_write.skn"));

        assert!(write_result.is_ok());
    }
}