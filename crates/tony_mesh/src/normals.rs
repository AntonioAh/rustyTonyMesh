use crate::acceleration::AccelerationStructure;
use crate::{FaceMatrix, Index, NormalMatrix, Real, VertexMatrix, acceleration, connectivity};
use nalgebra::{Matrix3, RowVector3, SymmetricEigen};
use rayon::prelude::*;

pub fn face_normals(vertices: &VertexMatrix, faces: &FaceMatrix) -> NormalMatrix {
    let num_faces = faces.nrows();
    let mut normals = NormalMatrix::zeros(num_faces);

    for (i, face) in faces.row_iter().enumerate() {
        let a = vertices.row(face[0] as usize);
        let b = vertices.row(face[1] as usize);
        let c = vertices.row(face[2] as usize);

        let n = (b - a)
            .cross(&(c - a))
            .try_normalize(Real::EPSILON)
            .unwrap_or_else(RowVector3::zeros);
        normals.row_mut(i).copy_from(&n);
    }

    normals
}
/// Vertex normals as the normalized plain average of the adjacent face normals.
/// Isolated vertices and vertices whose face normals cancel out get a zero normal.
pub fn avg_face_normals(vertices: &VertexMatrix, faces: &FaceMatrix) -> NormalMatrix {
    let num_vertices = vertices.nrows();
    let face_normals = face_normals(vertices, faces);
    let vertex_face = connectivity::vertex_face_list(num_vertices, faces);
    let mut result = NormalMatrix::zeros(num_vertices);

    for i in 0..num_vertices {
        let mut sum = RowVector3::<Real>::zeros();
        for &face in &vertex_face[i] {
            sum += face_normals.row(face as usize);
        }

        let n = sum
            .try_normalize(Real::EPSILON)
            .unwrap_or_else(RowVector3::zeros);
        result.row_mut(i).copy_from(&n);
    }
    result
}

/// Vertex normals as the normalized average of adjacent face normals, each weighted
/// by the face's interior angle at the vertex (independent of triangulation density).
pub fn avg_weighted_face_normals(vertices: &VertexMatrix, faces: &FaceMatrix) -> NormalMatrix {
    let num_vertices = vertices.nrows();
    let mut result = NormalMatrix::zeros(num_vertices);
    let face_normals = face_normals(vertices, faces);
    let vertex_face = connectivity::vertex_face_list(num_vertices, faces);

    for i in 0..num_vertices {
        let v = i as Index;
        let mut sum = RowVector3::<Real>::zeros();

        for &face in &vertex_face[i] {
            let f = faces.row(face as usize);

            // the other two corners, keeping cyclic order v -> p -> q
            let (p, q) = match v {
                _ if f[0] == v => (f[1], f[2]),
                _ if f[1] == v => (f[2], f[0]),
                _ if f[2] == v => (f[0], f[1]),
                _ => {
                    unreachable!("vertex_face_list returned a face that doesn't contain the vertex")
                }
            };

            let pv = vertices.row(i);
            let e1 = vertices.row(p as usize) - pv;
            let e2 = vertices.row(q as usize) - pv;

            // interior angle at v; atan2 is stable near 0 and pi and never NaN
            let alpha = e1.cross(&e2).norm().atan2(e1.dot(&e2));

            sum += face_normals.row(face as usize) * alpha;
        }
        let n = sum
            .try_normalize(Real::EPSILON)
            .unwrap_or_else(RowVector3::zeros);
        result.row_mut(i).copy_from(&n);
    }
    result
}

/// PCA normal of vertex `i`: the eigenvector of the smallest eigenvalue of the covariance of
/// the point and its `k` nearest neighbours. Zero when fewer than 3 points are available.
/// The sign is arbitrary (PCA gives a direction, not an orientation).
fn pca_normal(
    vertices: &VertexMatrix,
    tree: &acceleration::KdTree,
    i: usize,
    k: usize,
) -> RowVector3<Real> {
    let Some(mut idx) = tree.knn_exclude(i as Index, k) else {
        return RowVector3::zeros();
    };
    idx.push(i as Index);
    if idx.len() < 3 {
        return RowVector3::zeros();
    }

    // centroid of the neighbourhood
    let mut c = RowVector3::<Real>::zeros();
    for &j in &idx {
        c += vertices.row(j as usize);
    }
    c /= idx.len() as Real;

    // covariance (up to a scale) = sum of d^T * d with d = p - c
    let mut cov = Matrix3::<Real>::zeros();
    for &j in &idx {
        let d = vertices.row(j as usize) - c;
        cov += d.transpose() * d;
    }

    // symmetric matrix: real eigenvalues, not sorted by nalgebra
    let eig = SymmetricEigen::new(cov);
    eig.eigenvectors.column(eig.eigenvalues.imin()).transpose()
}

/// Per-vertex normals of a point cloud from PCA on the `k_neighbors` nearest neighbours,
/// computed in parallel.
pub fn point_cloud(vertices: &VertexMatrix, k_neighbors: usize) -> NormalMatrix {
    let tree = acceleration::KdTree::new(vertices);
    let num_vertices = vertices.nrows();

    // parallel map over the vertex indices; collect keeps the order
    let normals: Vec<RowVector3<Real>> = (0..num_vertices)
        .into_par_iter()
        .map(|i| pca_normal(vertices, &tree, i, k_neighbors))
        .collect();

    let mut result = NormalMatrix::zeros(num_vertices);
    for (i, n) in normals.iter().enumerate() {
        result.row_mut(i).copy_from(n);
    }
    result
}
