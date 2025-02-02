use self::Color::*;
use std::cmp::Ordering;
use std::fmt;
use std::iter;
use std::mem;

fn max<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if a >= b {
        a
    } else {
        b
    }
}

// fn min<T: PartialOrd + Copy>(a: T, b: T) -> T {
//     if a >= b {
//         b
//     } else {
//         a
//     }
// }

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Red,
    Black,
}

pub type NodeCell<K, V> = Option<Box<Node<K, V>>>;

pub struct Node<K, V> {
    pub key: K,
    pub val: V,
    pub left: NodeCell<K, V>,
    pub right: NodeCell<K, V>,
    pub color: Color,
}

impl<K, V> Node<K, V> {
    #[inline]
    pub fn new(key: K, val: V, color: Color) -> Node<K, V> {
        Node {
            key,
            val,
            left: None,
            right: None,
            color,
        }
    }

    #[inline]
    fn is_red(&self) -> bool {
        self.color == Red
    }

    fn depth(&self) -> usize {
        let lsz = self.left.as_ref().map_or(0, |n| n.depth());
        let rsz = self.right.as_ref().map_or(0, |n| n.depth());
        max(lsz, rsz) + 1
    }

    fn size(&self) -> usize {
        1 + self.left.as_ref().map_or(0, |n| n.size()) + self.right.as_ref().map_or(0, |n| n.size())
    }

    /// Left rotation. Orient a (temporarily) right-leaning red link to lean left.
    fn rotate_left(&mut self) {
        assert!(is_red(&self.right));
        let mut x = self.right.take();
        self.right = x.as_mut().unwrap().left.take();
        x.as_mut().unwrap().color = self.color;
        self.color = Red;
        let old_self = mem::replace(self, *x.unwrap());
        self.left = Some(Box::new(old_self));
    }

    /// Right rotation. Orient a left-leaning red link to (temporarily) lean right
    fn rotate_right(&mut self) {
        assert!(is_red(&self.left));
        let mut x = self.left.take();
        self.left = x.as_mut().unwrap().right.take();
        x.as_mut().unwrap().color = self.color;
        self.color = Red;
        let old_self = mem::replace(self, *x.unwrap());
        self.right = Some(Box::new(old_self));
    }

    /// Color flip. Recolor to split a (temporary) 4-node.
    fn flip_color(&mut self) {
        assert!(!self.is_red());
        assert!(is_red(&self.left));
        assert!(is_red(&self.right));
        self.color = Red;
        if let Some(n) = self.left.as_mut() {
            n.color = Black;
        }
        if let Some(n) = self.right.as_mut() {
            n.color = Black;
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> Node<K, V> {
    fn dump(&self, depth: usize, f: &mut fmt::Formatter, symbol: char) {
        if depth == 0 {
            writeln!(f, "\n{:?}[{:?}]", self.key, self.val).unwrap();
        } else if self.is_red() {
            writeln!(
                f,
                "{}{}=={:?}[{:?}]",
                iter::repeat("|  ").take(depth - 1).collect::<Vec<&str>>().concat(),
                symbol,
                self.key,
                self.val
            )
            .unwrap();
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

fn is_red<K, V>(x: &NodeCell<K, V>) -> bool {
    if x.as_ref().is_none() {
        false
    } else {
        x.as_ref().unwrap().color == Red
    }
}

fn put<K: PartialOrd, V>(mut x: NodeCell<K, V>, key: K, val: V) -> NodeCell<K, V> {
    if x.is_none() {
        return Some(Box::new(Node::new(key, val, Red)));
    }
    let cmp = key.partial_cmp(&x.as_ref().unwrap().key).unwrap();
    match cmp {
        Ordering::Less => {
            let left = x.as_mut().unwrap().left.take();
            x.as_mut().unwrap().left = put(left, key, val)
        }
        Ordering::Greater => {
            let right = x.as_mut().unwrap().right.take();
            x.as_mut().unwrap().right = put(right, key, val)
        }
        Ordering::Equal => x.as_mut().unwrap().val = val,
    }

    if is_red(&x.as_ref().unwrap().right) && !is_red(&x.as_ref().unwrap().left) {
        x.as_mut().unwrap().rotate_left();
    }
    if is_red(&x.as_ref().unwrap().left) && is_red(&x.as_ref().unwrap().left.as_ref().unwrap().left) {
        x.as_mut().unwrap().rotate_right();
    }
    if is_red(&x.as_ref().unwrap().left) && is_red(&x.as_ref().unwrap().right) {
        x.as_mut().unwrap().flip_color();
    }
    x
}

fn delete<K: PartialOrd, V>(mut x: NodeCell<K, V>, key: &K) -> NodeCell<K, V> {
    x.as_ref()?;

    match key.partial_cmp(&x.as_ref().unwrap().key).unwrap() {
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

pub struct RedBlackBST<K, V> {
    pub root: NodeCell<K, V>,
}

impl<K: PartialOrd, V> RedBlackBST<K, V> {
    pub fn depth(&self) -> usize {
        match self.root {
            None => 0,
            Some(ref x) => x.depth(),
        }
    }
}

impl<K: PartialOrd, V> Default for RedBlackBST<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialOrd, V> RedBlackBST<K, V> {
    pub fn new() -> RedBlackBST<K, V> {
        RedBlackBST { root: None }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut x = self.root.as_ref();
        while x.is_some() {
            match key.partial_cmp(&x.unwrap().key).unwrap() {
                Ordering::Less => {
                    x = x.unwrap().left.as_ref();
                }
                Ordering::Greater => {
                    x = x.unwrap().right.as_ref();
                }
                Ordering::Equal => return Some(&x.unwrap().val),
            }
        }
        None
    }

    pub fn put(&mut self, key: K, val: V) {
        self.root = put(self.root.take(), key, val);
        // FIXME: too bad
        self.root.as_mut().unwrap().color = Black;
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

fn floor<'a, K: PartialOrd, V>(x: Option<&'a Node<K, V>>, key: &K) -> Option<&'a Node<K, V>> {
    x?;

    match key.partial_cmp(&x.unwrap().key).unwrap() {
        Ordering::Equal => {
            return Some(x.unwrap());
        }
        Ordering::Less => {
            return floor(x.unwrap().left.as_deref(), key);
        }
        _ => (),
    }

    let t = floor(x.unwrap().right.as_deref(), key);
    if t.is_some() {
        t
    } else {
        Some(x.unwrap())
    }
}

fn ceiling<'a, K: PartialOrd, V>(x: Option<&'a Node<K, V>>, key: &K) -> Option<&'a Node<K, V>> {
    x?;

    match key.partial_cmp(&x.unwrap().key).unwrap() {
        Ordering::Equal => {
            return Some(x.unwrap());
        }
        Ordering::Greater => {
            return ceiling(x.unwrap().right.as_deref(), key);
        }
        _ => (),
    }

    let t = ceiling(x.unwrap().left.as_deref(), key);
    if t.is_some() {
        t
    } else {
        Some(x.unwrap())
    }
}

// delete_min helper
// returns: top, deleted
fn delete_min<K: PartialOrd, V>(mut x: NodeCell<K, V>) -> (NodeCell<K, V>, NodeCell<K, V>) {
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

// delete_max helper
// returns: top, deleted
fn delete_max<K: PartialOrd, V>(mut x: NodeCell<K, V>) -> (NodeCell<K, V>, NodeCell<K, V>) {
    if x.is_none() {
        return (None, None);
    }
    match x.as_mut().unwrap().right.take() {
        None => (x.as_mut().unwrap().left.take(), x),
        right @ Some(_) => {
            let (t, deleted) = delete_max(right);
            x.as_mut().unwrap().right = t;
            (x, deleted)
        }
    }
}

fn find_max<K: PartialOrd, V>(x: Option<&Node<K, V>>) -> Option<&Node<K, V>> {
    x?;
    match x.as_ref().unwrap().right.as_deref() {
        None => x,
        right @ Some(_) => find_max(right),
    }
}

fn find_min<K: PartialOrd, V>(x: Option<&Node<K, V>>) -> Option<&Node<K, V>> {
    x?;
    match x.as_ref().unwrap().left.as_deref() {
        None => x,
        left @ Some(_) => find_min(left),
    }
}

impl<K: PartialOrd, V> RedBlackBST<K, V> {
    /// smallest key
    pub fn min(&self) -> Option<&K> {
        find_min(self.root.as_deref()).map(|n| &n.key)
    }

    /// largest key
    pub fn max(&self) -> Option<&K> {
        find_max(self.root.as_deref()).map(|n| &n.key)
    }

    /// largest key less than or equal to key
    pub fn floor(&self, key: &K) -> Option<&K> {
        let x = floor(self.root.as_deref(), key);
        if let Some(x) = x {
            Some(&x.key)
        } else {
            None
        }
    }

    /// smallest key greater than or equal to key
    pub fn ceiling(&self, key: &K) -> Option<&K> {
        let x = ceiling(self.root.as_deref(), key);
        if let Some(x) = x {
            Some(&x.key)
        } else {
            None
        }
    }

    /// number of keys less than key
    pub fn rank(&self, key: &K) -> usize {
        fn rank_helper<K: PartialOrd, V>(x: Option<&Node<K, V>>, key: &K) -> usize {
            if x.is_none() {
                return 0;
            }

            match key.partial_cmp(&x.unwrap().key).unwrap() {
                Ordering::Less => rank_helper(x.unwrap().left.as_deref(), key),
                Ordering::Greater => {
                    1 + x.as_ref().unwrap().left.as_ref().map_or(0, |n| n.size())
                        + rank_helper(x.unwrap().right.as_deref(), key)
                }
                Ordering::Equal => x.as_ref().unwrap().left.as_ref().map_or(0, |n| n.size()),
            }
        }

        rank_helper(self.root.as_deref(), key)
    }

    /// key of rank k
    pub fn select(&self, k: usize) -> Option<&K> {
        self.keys().find(|&key| self.rank(key) == k)
    }

    /// delete smallest key
    pub fn delete_min(&mut self) {
        self.root = delete_min(self.root.take()).0;
    }

    /// delete largest key
    pub fn delete_max(&mut self) {
        self.root = delete_max(self.root.take()).0;
    }
}

impl<K: PartialOrd, V> RedBlackBST<K, V> {
    pub fn keys(&self) -> ::std::vec::IntoIter<&K> {
        fn inorder<'a, K, V>(x: Option<&'a Node<K, V>>, queue: &mut Vec<&'a K>) {
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

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for RedBlackBST<K, V> {
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
fn test_red_black_tree_shape() {
    let mut t = RedBlackBST::<i32, ()>::new();
    assert_eq!(0, t.depth());
    for c in 0..255 {
        t.put(c, ());
    }
    // println!("{:?}", t);
    assert_eq!(255, t.size());
    // max for n=255
    assert!(t.depth() <= 8);
}

#[test]
fn test_red_black_tree() {
    use std::iter::FromIterator;

    let mut t = RedBlackBST::<char, usize>::new();
    for (i, c) in "SEARCHEXAMP".chars().enumerate() {
        t.put(c, i);
    }

    // println!("{:?}", t);
    assert_eq!(t.get(&'E'), Some(&6));
    assert_eq!(t.floor(&'O'), Some(&'M'));
    assert_eq!(t.ceiling(&'Q'), Some(&'R'));
    assert_eq!(t.size(), 9);
    assert_eq!(t.rank(&'E'), 2);
    assert_eq!(t.select(2), Some(&'E'));
    assert_eq!(t.rank(&'M'), 4);
    assert_eq!(t.select(4), Some(&'M'));
    assert_eq!(t.max(), Some(&'X'));
    assert_eq!(t.min(), Some(&'A'));
    // inorder visite
    assert_eq!(String::from_iter(t.keys().copied()), "ACEHMPRSX");
}
