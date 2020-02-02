pub mod io;
pub mod structures;

#[cfg(test)]
mod tests
{
    use crate::io::world_geometry::WorldGeometry;

    #[test]
    fn test_wgeo()
    {
        let mut world_geometry = WorldGeometry::read("test_files/room_map11.wgeo");
        let mut models = world_geometry.models();

        assert_eq!(models.len(), 367);

        world_geometry.write("test_files/room_map11.write.wgeo");
    }
}