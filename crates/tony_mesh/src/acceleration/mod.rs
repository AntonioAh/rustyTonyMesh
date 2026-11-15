use crate::{Index, Real};

pub trait AccelerationStructure {
    fn nearest(&self, q: &[Real; 3]) -> Option<Index>;
    fn knn(&self, q: &[Real; 3], k: usize) -> Option<Vec<Index>>;
    fn knn_exclude(&self, idx: Index, k: usize) -> Option<Vec<Index>>;
    fn within_radius(&self, q: &[Real; 3], radius: Real) -> Option<Vec<Index>>;
}

mod kdtree;
pub use kdtree::KdTree;
