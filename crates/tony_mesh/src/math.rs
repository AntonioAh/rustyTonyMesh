use crate::{Point, Real};

pub fn distance_squared(first: &Point, second: &Point) -> Real {
    let a = first[0] - second[0];
    let b = first[1] - second[1];
    let c = first[2] - second[2];

    a * a + b * b + c * c
}

pub fn distance(first: &Point, second: &Point) -> Real {
    distance_squared(first, second).sqrt()
}
