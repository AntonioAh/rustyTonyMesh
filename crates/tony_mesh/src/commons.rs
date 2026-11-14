use nalgebra::{Dyn, OMatrix, U3};

pub type Real = f32;
pub type Index = i32;
pub type VertexMatrix = OMatrix<Real, Dyn, U3>;
pub type FaceMatrix = OMatrix<Index, Dyn, U3>;
pub type NormalMatrix = OMatrix<Real, Dyn, U3>;
