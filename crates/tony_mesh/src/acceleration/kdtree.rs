use crate::acceleration::AccelerationStructure;
use crate::{Index, Point, Real, VertexMatrix, math};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::ops::Range;

pub struct KdTree {
    points: Vec<Point>,
    order: Vec<Index>,
}

impl KdTree {
    pub fn new(vertices: &VertexMatrix) -> Self {
        let points: Vec<Point> = vertices.row_iter().map(|r| [r[0], r[1], r[2]]).collect();
        let mut order: Vec<Index> = (0..points.len() as Index).collect();

        Self::build(&points, &mut order, 0);
        Self { points, order }
    }

    fn build(points: &[Point], order: &mut [Index], depth: usize) {
        if order.len() <= 1 {
            return;
        }

        let axis = depth % 3;
        let mid = order.len() / 2;
        order.select_nth_unstable_by(mid, |&a, &b| {
            points[a as usize][axis].total_cmp(&points[b as usize][axis])
        });

        let (left, right) = order.split_at_mut(mid);
        Self::build(points, left, depth + 1);
        Self::build(points, &mut right[1..], depth + 1);
    }

    fn nearest_rec(
        &self,
        range: Range<usize>,
        depth: usize,
        q: &Point,
        best: &mut Option<(Index, Real)>,
    ) {
        if range.is_empty() {
            return;
        }

        let mid = range.start + (range.end - range.start) / 2;
        let idx = self.order[mid];
        let point = &self.points[idx as usize];

        let sq_distance = math::distance_squared(point, q);
        if best.map_or(true, |(_, d)| sq_distance < d) {
            *best = Some((idx, sq_distance));
        }

        let diff = q[depth % 3] - point[depth % 3];
        let (near, far) = if diff < 0.0 {
            (range.start..mid, mid + 1..range.end)
        } else {
            (mid + 1..range.end, range.start..mid)
        };
        self.nearest_rec(near, depth + 1, q, best);
        if diff * diff < best.unwrap().1 {
            self.nearest_rec(far, depth + 1, q, best);
        }
    }

    fn knn_rec(
        &self,
        range: Range<usize>,
        depth: usize,
        q: &Point,
        k: usize,
        exclude: Option<Index>,
        heap: &mut BinaryHeap<Candidate>,
    ) {
        if range.is_empty() {
            return;
        }

        let mid = range.start + (range.end - range.start) / 2;
        let idx = self.order[mid as usize];
        let point = self.points[idx as usize];

        if Some(idx) != exclude {
            let sq_distance = math::distance_squared(&point, q);
            if heap.len() < k {
                heap.push(Candidate(sq_distance, idx));
            } else if sq_distance < heap.peek().unwrap().0 {
                heap.pop();
                heap.push(Candidate(sq_distance, idx));
            }
        }

        let diff = q[depth % 3] - point[depth % 3];
        let (near, far) = if diff < 0.0 {
            (range.start..mid, mid + 1..range.end)
        } else {
            (mid + 1..range.end, range.start..mid)
        };

        self.knn_rec(near, depth + 1, q, k, exclude, heap);
        if heap.len() < k || diff * diff < heap.peek().unwrap().0 {
            self.knn_rec(far, depth + 1, q, k, exclude, heap);
        }
    }
}

#[derive(PartialEq)]
struct Candidate(Real, Index);
impl Eq for Candidate {}
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0).then(self.1.cmp(&other.1))
    }
}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AccelerationStructure for KdTree {
    fn nearest(&self, q: &[Real; 3]) -> Option<Index> {
        if self.points.len() < 1 {
            return None;
        }
        let mut best = None;
        self.nearest_rec(0..self.order.len(), 0, q, &mut best);
        Some(best.unwrap().0)
    }

    fn knn(&self, q: &[Real; 3], k: usize) -> Option<Vec<Index>> {
        if self.points.is_empty() {
            return None;
        }

        let mut heap = BinaryHeap::with_capacity(k + 1);
        self.knn_rec(0..self.points.len(), 0, q, k, None, &mut heap);

        if heap.is_empty() {
            return None;
        }

        Some(
            heap.into_sorted_vec()
                .into_iter()
                .rev()
                .map(|c| c.1)
                .collect(),
        )
    }

    fn knn_exclude(&self, idx: Index, k: usize) -> Option<Vec<Index>> {
        if self.points.is_empty() {
            return None;
        }
        let q: Point = self.points[idx as usize];

        let mut heap = BinaryHeap::with_capacity(k + 1);
        self.knn_rec(0..self.points.len(), 0, &q, k, Some(idx), &mut heap);

        if heap.is_empty() {
            return None;
        }

        Some(
            heap.into_sorted_vec()
                .into_iter()
                .rev()
                .map(|c| c.1)
                .collect(),
        )
    }

    fn within_radius(&self, q: &[Real; 3], radius: Real) -> Option<Vec<Index>> {
        todo!()
    }
}
