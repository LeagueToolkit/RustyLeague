pub mod io;
pub mod structures;

#[cfg(test)]
mod tests
{
    use crate::io::world_geometry::WorldGeometry;
    use crate::io::release_manifest::ReleaseManifest;
    use std::fs::File;
    use std::path::Path;
    use std::io::{Write, Read};

    #[test]
    fn test_wgeo()
    {
        let mut world_geometry = WorldGeometry::read_from_file("test_files/room_map11.wgeo");
        let mut models = world_geometry.models();

        assert_eq!(models.len(), 367);

        world_geometry.write_to_file("test_files/room_map11.write.wgeo");
    }

    #[test]
    fn test_release_manifest()
    {
        let mut release_manifest = ReleaseManifest::read_from_file("test_files/C944A5BD0686C600.manifest");
        let mut file = File::create(Path::new("out.txt")).unwrap();
    }
}