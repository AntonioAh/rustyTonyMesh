use crate::commons::{FaceMatrix, Index, VertexMatrix};
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: VertexMatrix,
    pub faces: FaceMatrix,
    // pub normals: NormalMatrix,
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
            second: f.max(s)
        }
    }
}

impl Mesh {
    pub fn new(vertices: VertexMatrix, faces: FaceMatrix) -> Self {
        Self { vertices, faces }
    }
    pub fn vertex_count(&self) -> Index {
        self.vertices.nrows() as Index
    }

    pub fn faces_count(&self) -> Index {
        self.faces.nrows() as Index
    }

    pub fn vertex_vertex(&self, vert: Index) -> Vec<Index> {
        let mut result: Vec<Index> = Vec::new();

        for row in self.faces.row_iter() {
            if row[0] == vert {
                result.extend([row[1], row[2]]);
            } else if row[1] == vert {
                result.extend([row[0], row[2]]);
            } else if row[2] == vert {
                result.extend([row[0], row[1]]);
            }
        }

        result.sort();
        result.dedup();
        result
    }

    pub fn vertex_degree(&self, vert: Index) -> Index {
        self.vertex_vertex(vert).len() as Index
    }

    pub fn vertex_face(&self, vert: Index) -> Vec<Index> {
        let mut result: Vec<Index> = Vec::new();
        for (i, row) in self.faces.row_iter().enumerate() {
            if row[0] == vert || row[1] == vert || row[2] == vert {
                result.push(i as Index);
            }
        }

        result
    }

    pub fn vertex_vertex_lists(&self) -> Vec<Vec<Index>> {
        let mut result: Vec<Vec<Index>> = vec![Vec::new(); self.vertices.nrows()];

        for row in self.faces.row_iter() {
            result[row[0] as usize].extend([row[1], row[2]]);
            result[row[1] as usize].extend([row[0], row[2]]);
            result[row[2] as usize].extend([row[0], row[1]]);
        }

        result.par_iter_mut().for_each(|v| {
            v.sort_unstable();
            v.dedup();
        });
        result
    }

    pub fn edges(&self) -> Vec<Edge>{
        let mut result: Vec<Edge> = Vec::with_capacity(self.faces.nrows() * 3);

        for row in self.faces.row_iter(){
            result.push(Edge::new(row[0], row[1]));
            result.push(Edge::new(row[1], row[2]));
            result.push(Edge::new(row[2], row[0]));
        }

        result.sort();
        result.dedup();
        return result;
    }
}
