use crate::{
    cs2::{CS2, bvh::read_bvh},
    parser::bvh::Bvh,
};

pub mod bvh;

pub fn read_map(cs2: &CS2) -> Option<Bvh> {
    let triangles = read_bvh(cs2)?;
    let mut bvh = Bvh::new();
    bvh.set(triangles);
    bvh.build();
    Some(bvh)
}
