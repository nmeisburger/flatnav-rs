mod distance;
mod index;
mod priority_queue;
mod storage;

pub use distance::EuclideanDistance;

pub type Index = index::Index<u32, f32, distance::EuclideanDistance>;
