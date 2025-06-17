mod distance;
mod index;
mod priority_queue;
pub mod reordering;
mod storage;

pub use distance::EuclideanDistance;

pub use reordering::Reordering;

pub type IndexEuclideanF32 = index::Index<u32, f32, distance::EuclideanDistance>;
