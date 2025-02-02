use std::borrow::Borrow;
use std::cmp::Ordering;
use std::f64;
use std::fmt;
use std::iter;
use std::vec::IntoIter;

use super::primitive::{Point2D, RectHV};
use super::Queue;

/// A generic multidimension point.
pub trait Point: Copy {
    // const DIMENSION: usize = 2;
    fn get(&self, d: usize) -> f64;

    #[inline]
    fn dimension() -> usize {
        2
    }
}

impl Point for Point2D {
    #[inline]
    fn get(&self, d: usize) -> f64 {
        if d == 0 {
            self.x
        } else if d == 1 {
            self.y
        } else {
            panic!("dimension not supported")
        }
    }
}

pub type NodeCell<K, V> = Option<Box<Node<K, V>>>;

pub struct Node<K: Point, V> {
    pub key: K,
    pub val: V,
    pub left: NodeCell<K, V>,
    pub right: NodeCell<K, V>,
    pub depth: usize,
}

impl<K: Point, V> Node<K, V> {
    pub fn new(key: K, val: V, depth: usize) -> Node<K, V> {
        Node {
            key,
            val,
            left: None,
            right: None,
            // depth use (depth % k)-th dimension
            depth,
        }
    }

    fn size(&self) -> usize {
        let mut ret = 1;
        if self.left.is_some() {
            ret += self.left.as_ref().unwrap().size()
        }
        if self.right.is_some() {
            ret += self.right.as_ref().unwrap().size()
        }
        ret
    }

    #[inline]
    fn comparator_for_current_dim(&self) -> f64 {
        // let dim = self.depth % <K as Point>::dimension();
        self.key.get(self.depth % <K as Point>::dimension())
    }
}

impl<K: Point + fmt::Debug, V: fmt::Debug> Node<K, V> {
    fn dump(&self, depth: usize, f: &mut fmt::Formatter, symbol: char) {
        if depth == 0 {
            writeln!(f, "\n{:?}[{:?}]", self.key, self.val).unwrap();
        } else {
            writeln!(
                f,
                "{}{}--{:?}[{:?}]",
                iter::repeat("|  ").take(depth - 1).collect::<Vec<&str>>().concat(),
                symbol,
                self.key,
                self.val
            )
            .unwrap();
        }
        if self.left.is_some() {
            self.left.as_ref().unwrap().dump(depth + 1, f, '+');
        }
        if self.right.is_some() {
            self.right.as_ref().unwrap().dump(depth + 1, f, '`');
        }
    }
}

fn put<K: Point, V>(x: NodeCell<K, V>, key: K, val: V, depth: usize) -> NodeCell<K, V> {
    let mut x = x;
    if x.is_none() {
        return Some(Box::new(Node::new(key, val, depth)));
    }
    let depth = x.as_ref().unwrap().depth;
    let dimension = <K as Point>::dimension();
    let current_dim = x.as_ref().unwrap().depth % dimension;
    let mut dim = current_dim;

    loop {
        let cmp = key.get(dim).partial_cmp(&x.as_ref().unwrap().key.get(dim)).unwrap();
        match cmp {
            Ordering::Less => {
                let left = x.as_mut().unwrap().left.take();
                x.as_mut().unwrap().left = put(left, key, val, depth + 1);
                break;
            }
            Ordering::Greater => {
                let right = x.as_mut().unwrap().right.take();
                x.as_mut().unwrap().right = put(right, key, val, depth + 1);
                break;
            }
            // when current dimension is equal, compare next non-equal dimension
            Ordering::Equal => {
                dim = (dim + 1) % dimension;
                if dim == current_dim {
                    x.as_mut().unwrap().val = val;
                    break;
                }
            }
        }
    }
    x
}

fn delete_min<K: Point, V>(x: NodeCell<K, V>) -> (NodeCell<K, V>, NodeCell<K, V>) {
    let mut x = x;
    if x.is_none() {
        return (None, None);
    }
    match x.as_mut().unwrap().left.take() {
        None => (x.as_mut().unwrap().right.take(), x),
        left @ Some(_) => {
            let (t, deleted) = delete_min(left);
            x.as_mut().unwrap().left = t;
            (x, deleted)
        }
    }
}

fn delete<K: Point, V>(x: NodeCell<K, V>, key: &K) -> NodeCell<K, V> {
    x.as_ref()?;

    let mut x = x;
    let dim = x.as_ref().unwrap().depth % <K as Point>::dimension();

    match key.get(dim).partial_cmp(&x.as_ref().unwrap().key.get(dim)).unwrap() {
        Ordering::Less => {
            let left = x.as_mut().unwrap().left.take();
            x.as_mut().unwrap().left = delete(left, key);
            x
        }
        Ordering::Greater => {
            let right = x.as_mut().unwrap().right.take();
            x.as_mut().unwrap().right = delete(right, key);
            x
        }
        Ordering::Equal => {
            if x.as_ref().unwrap().right.is_none() {
                return x.as_mut().unwrap().left.take();
            }
            if x.as_ref().unwrap().left.is_none() {
                return x.as_mut().unwrap().right.take();
            }

            // Save top
            let mut t = x;

            // split right into right without min, and the min
            let (right, right_min) = delete_min(t.as_mut().unwrap().right.take());
            x = right_min;
            x.as_mut().unwrap().right = right;
            x.as_mut().unwrap().left = t.as_mut().unwrap().left.take();
            x
        }
    }
}

pub struct KdTree<K: Point, V> {
    pub root: NodeCell<K, V>,
}

impl<K: Point, V> Default for KdTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Point, V> KdTree<K, V> {
    pub fn new() -> KdTree<K, V> {
        assert!(K::dimension() >= 2);
        KdTree { root: None }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut x = self.root.as_ref();
        let dimension = <K as Point>::dimension();
        let current_dim = x.as_ref().unwrap().depth % dimension;
        while x.is_some() {
            let mut dim = current_dim;
            loop {
                match key.get(dim).partial_cmp(&x.unwrap().key.get(dim)).unwrap() {
                    Ordering::Less => {
                        x = x.unwrap().left.as_ref();
                        break;
                    }
                    Ordering::Greater => {
                        x = x.unwrap().right.as_ref();
                        break;
                    }
                    Ordering::Equal => {
                        dim = (dim + 1) % dimension;
                        if dim == current_dim {
                            return Some(&x.unwrap().val);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn put(&mut self, key: K, val: V) {
        self.root = put(self.root.take(), key, val, 0);
    }

    pub fn delete(&mut self, key: &K) {
        self.root = delete(self.root.take(), key);
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// number of key-value pairs in the table
    pub fn size(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.root.as_ref().unwrap().size()
        }
    }
}

impl<K: Point, V> KdTree<K, V> {
    pub fn keys(&self) -> ::std::vec::IntoIter<&K> {
        fn inorder<'a, K: Point, V>(x: Option<&'a Node<K, V>>, queue: &mut Vec<&'a K>) {
            if x.is_none() {
                return;
            }
            inorder(x.unwrap().left.as_deref(), queue);
            queue.push(&x.unwrap().key);
            inorder(x.unwrap().right.as_deref(), queue);
        }

        let mut queue = Vec::new();
        inorder(self.root.as_deref(), &mut queue);
        queue.into_iter()
    }
}

impl KdTree<Point2D, ()> {
    // add the point to the KdTree
    pub fn insert(&mut self, p: Point2D) {
        self.put(p, ());
    }

    /// find all Point2D keys that lie in a 2d range
    pub fn range_search<T: Borrow<RectHV>>(&self, rect: T) -> IntoIter<&Point2D> {
        let mut result = Vec::new();
        let rect = rect.borrow();
        // use stack approach
        let mut stack = Vec::new();
        stack.push(self.root.as_ref());
        while !stack.is_empty() {
            let x = stack.pop().unwrap();

            if x.is_none() {
                continue;
            }

            let dim = x.as_ref().unwrap().depth % 2;

            // Check if point in node lies in given rectangle
            if rect.contains(x.as_ref().unwrap().key) {
                result.push(&x.as_ref().unwrap().key)
            }
            // Recursively search left/bottom (if any could fall in rectangle)
            // Recursively search right/top (if any could fall in rectangle)
            if dim == 0 {
                if rect.xmin < x.as_ref().unwrap().comparator_for_current_dim() {
                    stack.push(x.unwrap().left.as_ref())
                }
                if rect.xmax > x.as_ref().unwrap().comparator_for_current_dim() {
                    stack.push(x.unwrap().right.as_ref())
                }
            } else {
                // dim == 1: y
                if rect.ymin < x.as_ref().unwrap().comparator_for_current_dim() {
                    stack.push(x.unwrap().left.as_ref())
                }
                if rect.ymax > x.as_ref().unwrap().comparator_for_current_dim() {
                    stack.push(x.unwrap().right.as_ref())
                }
            }
        }
        result.into_iter()
    }

    /// number of keys that lie in a 2d range
    pub fn range_count<T: Borrow<RectHV>>(&self, rect: T) -> usize {
        self.range_search(rect).count()
    }

    // TODO: refactor to a generic solution
    pub fn nearest<T: Borrow<Point2D>>(&self, p: T) -> Option<&Point2D> {
        let mut result = None;
        let mut min_distance = f64::MAX;
        let p = p.borrow();

        // use FIFO queue
        let mut queue = Queue::new();
        queue.enqueue(self.root.as_ref());
        while !queue.is_empty() {
            let x = queue.dequeue().unwrap();

            if x.is_none() {
                continue;
            }

            let dim = x.as_ref().unwrap().depth % 2;

            // Check distance from point in node to query point
            let dist = x.as_ref().unwrap().key.distance_to(p);
            if dist < min_distance {
                result = Some(&x.as_ref().unwrap().key);
                min_distance = dist;
            }

            // Recursively search left/bottom (if it could contain a closer point)
            // Recursively search right/top (if it could contain a closer point)
            // FIXME: duplicated code
            if dim == 0 {
                // p in left
                if p.x < x.unwrap().key.x {
                    queue.enqueue(x.unwrap().left.as_ref());
                    if x.unwrap().right.is_some() {
                        let perpendicular_len = (p.y - x.unwrap().right.as_ref().unwrap().key.y).abs();
                        if perpendicular_len < min_distance {
                            queue.enqueue(x.unwrap().right.as_ref());
                        }
                    }
                } else {
                    // p in right
                    queue.enqueue(x.unwrap().right.as_ref());
                    if x.unwrap().left.is_some() {
                        let perpendicular_len = (p.y - x.unwrap().left.as_ref().unwrap().key.y).abs();
                        if perpendicular_len < min_distance {
                            queue.enqueue(x.unwrap().left.as_ref());
                        }
                    }
                }
            } else if p.y < x.unwrap().key.y {
                queue.enqueue(x.unwrap().left.as_ref());
                if x.unwrap().right.is_some() {
                    let perpendicular_len = (p.x - x.unwrap().right.as_ref().unwrap().key.x).abs();
                    if perpendicular_len < min_distance {
                        queue.enqueue(x.unwrap().right.as_ref());
                    }
                }
            } else {
                queue.enqueue(x.unwrap().right.as_ref());
                if x.unwrap().left.is_some() {
                    let perpendicular_len = (p.x - x.unwrap().left.as_ref().unwrap().key.x).abs();
                    if perpendicular_len < min_distance {
                        queue.enqueue(x.unwrap().left.as_ref());
                    }
                }
            }
        }
        result
    }
}

impl<K: Point + fmt::Debug, V: fmt::Debug> fmt::Debug for KdTree<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.root.is_none() {
            write!(f, "<empty tree>")
        } else {
            self.root.as_ref().unwrap().dump(0, f, ' ');
            Ok(())
        }
    }
}

#[test]
fn test_kd_tree_with_point_2d() {
    let mut t = KdTree::<Point2D, ()>::new();

    assert!(t.nearest(Point2D::new(0.9, 0.8)).is_none());

    t.put(Point2D::new(0.7, 0.2), ());
    t.put(Point2D::new(0.5, 0.4), ());
    t.put(Point2D::new(0.2, 0.3), ());
    t.put(Point2D::new(0.4, 0.7), ());
    t.put(Point2D::new(0.9, 0.6), ());

    // println!("got => {:?}", t);

    assert_eq!(5, t.range_search(RectHV::new(0.1, 0.1, 0.9, 0.9)).count());
    assert_eq!(1, t.range_search(RectHV::new(0.1, 0.1, 0.4, 0.4)).count());

    assert_eq!(&Point2D::new(0.2, 0.3), t.nearest(Point2D::new(0.1, 0.1)).unwrap());
    assert_eq!(&Point2D::new(0.9, 0.6), t.nearest(Point2D::new(0.9, 0.8)).unwrap());
}

#[test]
fn test_kd_tree_with_point_2d_duplicated() {
    let mut t = KdTree::<Point2D, ()>::new();

    t.put(Point2D::new(0.7, 0.2), ());
    t.put(Point2D::new(0.5, 0.4), ());
    t.put(Point2D::new(0.2, 0.3), ());
    t.put(Point2D::new(0.2, 0.7), ());
    t.put(Point2D::new(0.4, 0.7), ());
    t.put(Point2D::new(0.4, 0.2), ());
    t.put(Point2D::new(0.9, 0.6), ());
    t.put(Point2D::new(0.7, 0.4), ());
    // same node, this is replace behavior, no new node
    t.put(Point2D::new(0.9, 0.6), ());

    assert_eq!(8, t.size());
    assert!(t.contains(&Point2D::new(0.7, 0.2)));
    assert!(t.contains(&Point2D::new(0.7, 0.4)));
    assert!(!t.contains(&Point2D::new(0.7, 0.3)));
    assert!(!t.contains(&Point2D::new(0.4, 0.3)));
    assert_eq!(8, t.range_search(RectHV::new(0.1, 0.1, 0.9, 0.9)).count());
    assert_eq!(2, t.range_search(RectHV::new(0.1, 0.1, 0.4, 0.4)).count());

    assert_eq!(t.nearest(&Point2D::new(0.7, 0.39)).unwrap(), &Point2D::new(0.7, 0.4));
}

// A B E C D H F G
#[test]
fn test_kd_tree_quiz_777404() {
    let mut t = KdTree::<Point2D, char>::new();

    t.put(Point2D::new(0.50, 0.23), 'A');
    t.put(Point2D::new(0.25, 0.75), 'B');
    t.put(Point2D::new(0.17, 0.72), 'C');
    t.put(Point2D::new(0.01, 0.82), 'D');
    t.put(Point2D::new(0.71, 0.86), 'E');
    t.put(Point2D::new(0.98, 0.94), 'F');
    t.put(Point2D::new(0.08, 0.66), 'G');
    t.put(Point2D::new(0.57, 0.20), 'H');

    println!("tree : {:?}", t);
}
