use crate::{cs2::bvh::read_bvh, os::process::Process, parser::bvh::Bvh};

pub mod bvh;

pub fn read_map(process: &Process, vphys_world: usize) -> Option<Bvh> {
    let triangles = read_bvh(process, vphys_world)?;
    let mut bvh = Bvh::new();
    bvh.set(triangles);
    bvh.build();
    Some(bvh)
}
