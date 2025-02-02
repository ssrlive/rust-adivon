use super::rbtree::RedBlackBST;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::borrow::Borrow;
use std::f64;
use std::fmt;
use std::vec::IntoIter;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Point2D {
        Point2D { x, y }
    }

    pub fn distance_to<T: Borrow<Point2D>>(&self, that: T) -> f64 {
        self.distance_squared_to(that).sqrt()
    }

    pub fn distance_squared_to<T: Borrow<Point2D>>(&self, that: T) -> f64 {
        (self.x - that.borrow().x).powi(2) + (self.y - that.borrow().y).powi(2)
    }
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Distribution<Point2D> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point2D {
        Point2D {
            x: rng.gen(),
            y: rng.gen(),
        }
    }
}

#[test]
fn test_point2d() {
    let p1 = Point2D::new(0.0, 0.0);
    let p2 = Point2D::new(1.0, 1.0);

    // maybe bad :(
    assert_eq!(p1.distance_to(p2), (2.0f64).sqrt());
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
/// Implementation of 2D axis-aligned rectangle
pub struct RectHV {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl RectHV {
    pub fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> RectHV {
        RectHV { xmin, ymin, xmax, ymax }
    }

    pub fn width(&self) -> f64 {
        self.xmax - self.xmin
    }

    pub fn height(&self) -> f64 {
        self.ymax - self.ymin
    }

    pub fn contains<T: Borrow<Point2D>>(&self, p: T) -> bool {
        let p = p.borrow();
        p.x >= self.xmin && p.y >= self.ymin && p.x <= self.xmax && p.y <= self.ymax
    }

    /// does this axis-aligned rectangle intersect that one?
    pub fn intersects<T: Borrow<RectHV>>(&self, that: T) -> bool {
        let that = that.borrow();
        self.xmax >= that.xmin && self.ymax >= that.ymin && that.xmax >= self.xmin && that.ymax >= self.ymin
    }

    /// distance from p to closest point on this axis-aligned rectangle
    pub fn distance_to<T: Borrow<Point2D>>(&self, p: T) -> f64 {
        self.distance_squared_to(p).sqrt()
    }

    /// distance squared from p to closest point on this axis-aligned rectangle
    pub fn distance_squared_to<T: Borrow<Point2D>>(&self, p: T) -> f64 {
        let p = p.borrow();
        let mut dx = 0.0;
        let mut dy = 0.0;
        if p.x < self.xmin {
            dx = p.x - self.xmin;
        } else if p.x > self.xmax {
            dx = p.x - self.xmax;
        }
        if p.y < self.ymin {
            dy = p.y - self.ymin;
        } else if p.y > self.ymax {
            dy = p.y - self.ymax;
        }
        dx.powi(2) + dy.powi(2)
    }
}

impl fmt::Display for RectHV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}] x [{}, {}]", self.xmin, self.xmax, self.ymin, self.ymax)
    }
}

#[test]
fn test_rect() {
    let r1 = RectHV::new(0.0, 0.0, 1.1, 1.1);
    let r2 = RectHV::new(1.2, 2.0, 3.1, 4.1);
    assert!(!r1.intersects(r2));
}

/// Represents a set of points in the unit square
/// implemented using `RedBlackBST`
pub struct PointSet {
    pset: RedBlackBST<Point2D, ()>,
}

impl Default for PointSet {
    fn default() -> Self {
        Self::new()
    }
}

impl PointSet {
    pub fn new() -> PointSet {
        PointSet {
            pset: RedBlackBST::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.pset.size()
    }

    pub fn insert(&mut self, p: Point2D) {
        if !self.pset.contains(&p) {
            self.pset.put(p, ())
        }
    }

    pub fn contains<T: Borrow<Point2D>>(&self, p: T) -> bool {
        self.pset.contains(p.borrow())
    }

    pub fn range_search<T: Borrow<RectHV>>(&self, rect: T) -> IntoIter<&Point2D> {
        let mut result = Vec::new();
        for p in self.pset.keys() {
            if rect.borrow().contains(p) {
                result.push(p);
            }
        }
        result.into_iter()
    }

    pub fn range_count<T: Borrow<RectHV>>(&self, rect: T) -> usize {
        self.range_search(rect).count()
    }

    pub fn nearest<T: Borrow<Point2D>>(&self, p: T) -> Option<&Point2D> {
        let mut min_distance = f64::MAX;
        let mut result = None;
        for q in self.pset.keys() {
            let dist = p.borrow().distance_to(q);
            if dist < min_distance {
                result = Some(q);
                min_distance = dist;
            }
        }
        result
    }
}

#[test]
fn test_point_set() {
    use rand::thread_rng;

    let mut rng = thread_rng();
    let mut ps = PointSet::new();
    for _ in 0..100 {
        ps.insert(rng.gen())
    }
    assert_eq!(ps.size(), 100);

    assert!(ps.nearest(Point2D::new(0.5, 0.5)).is_some());
    assert!(ps.range_search(RectHV::new(0.1, 0.1, 0.9, 0.9)).count() > 0);
}
