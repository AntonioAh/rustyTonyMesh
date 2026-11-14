use crate::io::{get_file_type, get_path, FileType, IoError};
use crate::{FaceMatrix, Index, Mesh, Real, VertexMatrix};
use std::fs;
use std::path::Path;

fn read_off(path: &Path) -> Result<Mesh, IoError> {
    let text = fs::read_to_string(path)?;
    let bad = || IoError::WrongFileFormat;

    // Meaningful lines only: strip `#` comments (full-line and inline), trim, drop blanks.
    let mut lines = text
        .lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty());

    if lines.next() != Some("OFF") {
        return Err(bad());
    }

    // "num_vertices num_faces [num_edges]"
    let counts = lines.next().ok_or_else(bad)?;
    let mut it = counts.split_whitespace();
    let (Some(nv), Some(nf)) = (it.next(), it.next()) else {
        return Err(bad());
    };
    let nv: usize = nv.parse().map_err(|_| bad())?;
    let nf: usize = nf.parse().map_err(|_| bad())?;

    // Capacity is capped so a bogus header cannot trigger a huge allocation.
    let mut vertices: Vec<Real> = Vec::with_capacity(nv.min(1 << 20) * 3);
    for _ in 0..nv {
        let line = lines.next().ok_or_else(bad)?;
        let mut it = line.split_whitespace();
        let (Some(x), Some(y), Some(z)) = (it.next(), it.next(), it.next()) else {
            return Err(bad());
        };
        for tok in [x, y, z] {
            vertices.push(tok.parse::<Real>().map_err(|_| bad())?);
        }
    }

    // Face line: "3 a b c" (triangles only; extra tokens such as colors are ignored)
    let mut faces: Vec<Index> = Vec::with_capacity(nf.min(1 << 20) * 3);
    for _ in 0..nf {
        let line = lines.next().ok_or_else(bad)?;
        let mut it = line.split_whitespace();
        let n: usize = it.next().ok_or_else(bad)?.parse().map_err(|_| bad())?;
        if n != 3 {
            return Err(IoError::NotTriangleMesh);
        }
        for _ in 0..3 {
            let idx: Index = it.next().ok_or_else(bad)?.parse().map_err(|_| bad())?;
            if idx < 0 || idx as usize >= nv {
                return Err(bad());
            }
            faces.push(idx);
        }
    }

    Ok(Mesh::new(
        VertexMatrix::from_row_slice(&vertices),
        FaceMatrix::from_row_slice(&faces),
    ))
}

fn read_obj(_path: &Path) -> Result<Mesh, IoError> {
    Err(IoError::YetToImplement)
}

pub fn read_mesh(file_name: &str) -> Result<Mesh, IoError> {
    let path = get_path(file_name)?;
    let file_type = get_file_type(&path)?;

    match file_type {
        FileType::Off => read_off(&path),
        FileType::Obj => read_obj(&path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Write `content` to a uniquely named file in the system temp directory.
    fn write_tmp(name: &str, content: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("tony_mesh_{}_{}", std::process::id(), name));
        fs::write(&p, content).unwrap();
        p
    }

    #[test]
    fn reads_valid_triangle_with_comments_and_blanks() {
        let p = write_tmp(
            "valid.off",
            "OFF\n# a triangle\n\n3 1 0   # counts\n0 0 0\n  # indented comment\n1 0 0\n0 1 0\n3 0 1 2\n",
        );
        let mesh = read_off(&p).unwrap();
        assert_eq!(mesh.vertices.nrows(), 3);
        assert_eq!(mesh.faces.nrows(), 1);
        assert_eq!(mesh.vertices[(1, 0)], 1.0);
        assert_eq!(mesh.faces[(0, 2)], 2);
    }

    #[test]
    fn rejects_empty_file() {
        let p = write_tmp("empty.off", "");
        assert!(matches!(read_off(&p), Err(IoError::WrongFileFormat)));
    }

    #[test]
    fn rejects_wrong_header() {
        let p = write_tmp("header.off", "OBJ\n0 0 0\n");
        assert!(matches!(read_off(&p), Err(IoError::WrongFileFormat)));
    }

    #[test]
    fn rejects_truncated_vertex_list() {
        let p = write_tmp("trunc.off", "OFF\n3 1 0\n0 0 0\n1 0 0\n");
        assert!(matches!(read_off(&p), Err(IoError::WrongFileFormat)));
    }

    #[test]
    fn rejects_non_numeric_coordinate() {
        let p = write_tmp("nan.off", "OFF\n1 0 0\n0 x 0\n");
        assert!(matches!(read_off(&p), Err(IoError::WrongFileFormat)));
    }

    #[test]
    fn rejects_quad_faces() {
        let p = write_tmp("quad.off", "OFF\n4 1 0\n0 0 0\n1 0 0\n1 1 0\n0 1 0\n4 0 1 2 3\n");
        assert!(matches!(read_off(&p), Err(IoError::NotTriangleMesh)));
    }

    #[test]
    fn rejects_out_of_range_face_index() {
        let p = write_tmp("range.off", "OFF\n3 1 0\n0 0 0\n1 0 0\n0 1 0\n3 0 1 3\n");
        assert!(matches!(read_off(&p), Err(IoError::WrongFileFormat)));
    }

    #[test]
    fn rejects_negative_face_index() {
        let p = write_tmp("neg.off", "OFF\n3 1 0\n0 0 0\n1 0 0\n0 1 0\n3 0 1 -1\n");
        assert!(matches!(read_off(&p), Err(IoError::WrongFileFormat)));
    }
}
