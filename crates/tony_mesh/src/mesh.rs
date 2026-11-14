use crate::commons::{FaceMatrix, Index, VertexMatrix};
use crate::mesh::MeshError::NonManifold;
use rayon::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: VertexMatrix,
    pub faces: FaceMatrix,
    // pub normals: NormalMatrix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshError{
    NonManifold
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

    pub fn edge_manifold_flags(&self) -> Vec<bool>{
        let edges = self.edges();
        let mut count: Vec<usize> = vec![0usize; edges.len()];

        for face in self.faces.row_iter(){
            for k in 0..3{
                let curr = Edge::new(face[k], face[(k+1)%3]);
                let idx = edges.binary_search_by(|e| e.cmp(&curr)).unwrap();

                count[idx] += 1;
            }
        }

        count.iter().map(|&c| c <= 2).collect()
    }

    pub fn is_edge_manifold(&self) -> bool{
        self.edge_manifold_flags().iter().all(|&ok| ok)
    }

    /// One flag per vertex: `true` when the faces around it form a single fan
    /// (one path for a boundary vertex, one cycle for an interior vertex).
    /// Vertices not used by any face are reported as manifold.
    pub fn vertex_manifold_flags(&self) -> Vec<bool> {
        // For every face (v, a, b) the "link" of v gets the edge (a, b).
        let mut links: Vec<Vec<(Index, Index)>> = vec![Vec::new(); self.vertices.nrows()];
        for face in self.faces.row_iter() {
            for k in 0..3 {
                links[face[k] as usize].push((face[(k + 1) % 3], face[(k + 2) % 3]));
            }
        }

        links
            .par_iter()
            .map(|link| {
                let m = link.len();
                if m == 0 {
                    return true;                                    // isolated vertex
                }
                if link.iter().any(|&(a, b)| a == b) {
                    return false;                                   // degenerate face
                }

                // Every neighbour may appear at most twice, otherwise the edge (v, neighbour)
                // belongs to 3 or more faces.
                let mut ends: Vec<Index> = link.iter().flat_map(|&(a, b)| [a, b]).collect();
                ends.sort_unstable();
                if ends.windows(3).any(|w| w[0] == w[2]) {
                    return false;
                }

                // The link must be connected: walk across link edges that share an endpoint.
                let mut visited = vec![false; m];
                visited[0] = true;
                let mut seen = 1;
                let mut stack = vec![0usize];
                while let Some(e) = stack.pop() {
                    let (a, b) = link[e];
                    for j in 0..m {
                        if !visited[j] {
                            let (c, d) = link[j];
                            if a == c || a == d || b == c || b == d {
                                visited[j] = true;
                                seen += 1;
                                stack.push(j);
                            }
                        }
                    }
                }
                seen == m
            })
            .collect()
    }

    pub fn is_vertex_manifold(&self) -> bool {
        self.vertex_manifold_flags().iter().all(|&ok| ok)
    }

    pub fn is_manifold(&self) -> bool {
        self.is_edge_manifold() && self.is_vertex_manifold()
    }

    pub fn edge_faces(&self) -> Result<Vec<(Edge, [Index; 2])>, MeshError> {
        let edges = self.edges();

        let mut result: Vec<(Edge, [Index; 2])> = Vec::with_capacity(edges.len());

        for edge in &edges{
            result.push((*edge, [-1, -1]));
        }

        for (i, face) in self.faces.row_iter().enumerate(){
            for k in 0..3 {
                let curr = Edge::new(face[k], face[(k+1)%3]);

                let idx = edges.binary_search_by(|e| e.cmp(&curr)).unwrap();
                if result[idx].1[0] == -1{
                    result[idx].1[0] = i as Index;
                }
                else if result[idx].1[1] == -1{
                    result[idx].1[1] = i as Index;
                }
                else{
                    return Err(NonManifold);
                }
            }
        }
        Ok(result)
    }
}
