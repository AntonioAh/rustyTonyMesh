use crate::commons::{FaceMatrix, Index, VertexMatrix};
use crate::{NormalMatrix, connectivity, normals};

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: VertexMatrix,
    pub faces: FaceMatrix,
    pub normals: NormalMatrix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshError {
    NonManifold,
}
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Edge {
    pub first: Index,
    pub second: Index,
}

impl Edge {
    pub fn new(f: Index, s: Index) -> Self {
        Self {
            first: f.min(s),
            second: f.max(s),
        }
    }
}

impl Mesh {
    pub fn new(vertices: VertexMatrix, faces: FaceMatrix) -> Self {
        let norm = normals::avg_weighted_face_normals(&vertices, &faces);
        Self {
            vertices,
            faces,
            normals: norm,
        }
    }
    pub fn vertex_count(&self) -> Index {
        self.vertices.nrows() as Index
    }

    pub fn faces_count(&self) -> Index {
        self.faces.nrows() as Index
    }

    pub fn vertex_vertex(&self, vert: Index) -> Vec<Index> {
        connectivity::vertex_vertex(&self.faces, vert)
    }

    pub fn vertex_degree(&self, vert: Index) -> Index {
        connectivity::vertex_degree(&self.faces, vert)
    }

    pub fn vertex_face(&self, vert: Index) -> Vec<Index> {
        connectivity::vertex_face(&self.faces, vert)
    }

    pub fn vertex_face_list(&self) -> Vec<Vec<Index>> {
        connectivity::vertex_face_list(self.vertices.nrows(), &self.faces)
    }

    pub fn vertex_vertex_lists(&self) -> Vec<Vec<Index>> {
        connectivity::vertex_vertex_lists(self.vertices.nrows(), &self.faces)
    }

    pub fn edges(&self) -> Vec<Edge> {
        connectivity::edges(&self.faces)
    }

    pub fn edge_manifold_flags(&self) -> Vec<bool> {
        connectivity::edge_manifold_flags(&self.faces)
    }

    pub fn is_edge_manifold(&self) -> bool {
        connectivity::is_edge_manifold(&self.faces)
    }

    pub fn vertex_manifold_flags(&self) -> Vec<bool> {
        connectivity::vertex_manifold_flags(self.vertices.nrows(), &self.faces)
    }

    pub fn is_vertex_manifold(&self) -> bool {
        connectivity::is_vertex_manifold(self.vertices.nrows(), &self.faces)
    }

    pub fn is_manifold(&self) -> bool {
        connectivity::is_manifold(self.vertices.nrows(), &self.faces)
    }

    pub fn edge_faces(&self) -> Result<Vec<(Edge, [Index; 2])>, MeshError> {
        connectivity::edge_faces(&self.faces)
    }
    pub fn face_normals(&self) -> NormalMatrix {
        normals::face_normals(&self.vertices, &self.faces)
    }
}
