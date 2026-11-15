use crate::mesh::MeshError::NonManifold;
use crate::{Edge, FaceMatrix, Index, MeshError};
use rayon::prelude::*;

/// Sorted, deduplicated indices of the vertices sharing a face with `vert`.
/// Scans every face, so use `vertex_vertex_lists` when you need all vertices.
pub fn vertex_vertex(faces: &FaceMatrix, vert: Index) -> Vec<Index> {
    let mut result: Vec<Index> = Vec::new();

    for row in faces.row_iter() {
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

/// Number of distinct neighbour vertices of `vert` (its valence).
pub fn vertex_degree(faces: &FaceMatrix, vert: Index) -> Index {
    vertex_vertex(faces, vert).len() as Index
}

/// Indices of the faces that contain `vert`.
/// Scans every face, so use `vertex_face_list` when you need all vertices.
pub fn vertex_face(faces: &FaceMatrix, vert: Index) -> Vec<Index> {
    let mut result: Vec<Index> = Vec::new();
    for (i, row) in faces.row_iter().enumerate() {
        if row[0] == vert || row[1] == vert || row[2] == vert {
            result.push(i as Index);
        }
    }

    result
}

/// For every vertex, the indices of the faces that contain it.
pub fn vertex_face_list(num_vertices: usize, faces: &FaceMatrix) -> Vec<Vec<Index>> {
    let mut result: Vec<Vec<Index>> = vec![Vec::new(); num_vertices];

    for (i, face) in faces.row_iter().enumerate() {
        result[face[0] as usize].push(i as Index);
        result[face[1] as usize].push(i as Index);
        result[face[2] as usize].push(i as Index);
    }
    result
}

/// For every vertex, its sorted, deduplicated neighbour vertices.
pub fn vertex_vertex_lists(num_vertices: usize, faces: &FaceMatrix) -> Vec<Vec<Index>> {
    let mut result: Vec<Vec<Index>> = vec![Vec::new(); num_vertices];

    for row in faces.row_iter() {
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

/// Unique undirected edges of the mesh, sorted (each edge is stored with first < second).
pub fn edges(faces: &FaceMatrix) -> Vec<Edge> {
    let mut result: Vec<Edge> = Vec::with_capacity(faces.nrows() * 3);

    for row in faces.row_iter() {
        result.push(Edge::new(row[0], row[1]));
        result.push(Edge::new(row[1], row[2]));
        result.push(Edge::new(row[2], row[0]));
    }

    result.sort();
    result.dedup();
    return result;
}

/// One flag per edge of `edges`: `true` when the edge is shared by at most two faces.
pub fn edge_manifold_flags(faces: &FaceMatrix) -> Vec<bool> {
    let edges = edges(faces);
    let mut count: Vec<usize> = vec![0usize; edges.len()];

    for face in faces.row_iter() {
        for k in 0..3 {
            let curr = Edge::new(face[k], face[(k + 1) % 3]);
            let idx = edges.binary_search_by(|e| e.cmp(&curr)).unwrap();

            count[idx] += 1;
        }
    }

    count.iter().map(|&c| c <= 2).collect()
}

/// `true` when every edge belongs to at most two faces.
pub fn is_edge_manifold(faces: &FaceMatrix) -> bool {
    edge_manifold_flags(faces).iter().all(|&ok| ok)
}

/// One flag per vertex: `true` when the faces around it form a single fan
/// (one path for a boundary vertex, one cycle for an interior vertex).
/// Vertices not used by any face are reported as manifold.
pub fn vertex_manifold_flags(num_vertices: usize, faces: &FaceMatrix) -> Vec<bool> {
    // For every face (v, a, b) the "link" of v gets the edge (a, b).
    let mut links: Vec<Vec<(Index, Index)>> = vec![Vec::new(); num_vertices];
    for face in faces.row_iter() {
        for k in 0..3 {
            links[face[k] as usize].push((face[(k + 1) % 3], face[(k + 2) % 3]));
        }
    }

    links
        .par_iter()
        .map(|link| {
            let m = link.len();
            if m == 0 {
                return true; // isolated vertex
            }
            if link.iter().any(|&(a, b)| a == b) {
                return false; // degenerate face
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

/// `true` when every vertex passes the single-fan test of `vertex_manifold_flags`.
pub fn is_vertex_manifold(num_vertices: usize, faces: &FaceMatrix) -> bool {
    vertex_manifold_flags(num_vertices, faces)
        .iter()
        .all(|&ok| ok)
}

/// `true` when the mesh is both edge-manifold and vertex-manifold.
pub fn is_manifold(num_vertices: usize, faces: &FaceMatrix) -> bool {
    is_edge_manifold(faces) && is_vertex_manifold(num_vertices, faces)
}

/// For every edge of `edges`, the (up to two) faces that share it, `-1` marking a missing face.
/// Fails with `NonManifold` if an edge belongs to more than two faces.
pub fn edge_faces(faces: &FaceMatrix) -> Result<Vec<(Edge, [Index; 2])>, MeshError> {
    let edges = edges(faces);

    let mut result: Vec<(Edge, [Index; 2])> = Vec::with_capacity(edges.len());

    for edge in &edges {
        result.push((*edge, [-1, -1]));
    }

    for (i, face) in faces.row_iter().enumerate() {
        for k in 0..3 {
            let curr = Edge::new(face[k], face[(k + 1) % 3]);

            let idx = edges.binary_search_by(|e| e.cmp(&curr)).unwrap();
            if result[idx].1[0] == -1 {
                result[idx].1[0] = i as Index;
            } else if result[idx].1[1] == -1 {
                result[idx].1[1] = i as Index;
            } else {
                return Err(NonManifold);
            }
        }
    }
    Ok(result)
}
