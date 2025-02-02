use super::super::bag;
use super::super::bag::Bag;
use std::iter;

#[derive(Clone, Debug)]
pub struct Graph {
    v: usize,
    e: usize,
    adj: Vec<Bag<usize>>,
}

impl Graph {
    pub fn new(v: usize) -> Graph {
        Graph {
            v,
            e: 0,
            adj: iter::repeat(Bag::<usize>::new()).take(v).collect(),
        }
    }

    fn validate_vertex(&self, v: usize) {
        assert!(v < self.v, "vertex is not between 0 and {}", self.v - 1)
    }

    pub fn vertices(&self) -> usize {
        self.v
    }

    pub fn edges(&self) -> usize {
        self.e
    }

    pub fn add_edge(&mut self, v: usize, w: usize) {
        self.validate_vertex(v);
        self.validate_vertex(w);

        self.e += 1;
        self.adj[v].add(w);
        self.adj[w].add(v);
    }

    pub fn degree(&self, v: usize) -> usize {
        self.validate_vertex(v);
        self.adj[v].len()
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::new();

        dot.push_str("graph G {\n");
        for i in 0..self.v {
            dot.push_str(&format!("  {};\n", i));
        }

        //let mut edges = Vec::new();
        for (v, adj) in self.adj.iter().enumerate() {
            for w in adj.iter() {
                // if let iter::MinMaxResult::MinMax(mi, ma) = vec![v, *w].into_iter().min_max() {
                //     if !edges.contains(&(mi, ma)) {
                //         edges.push((mi, ma))
                //     }
                // }
                dot.push_str(&format!("  {} -- {};\n", v, w));
            }
        }
        // for &(v, w) in edges.iter() {
        //     dot.push_str(&format!("  {} -- {};\n", v, w));
        // }
        dot.push_str("}\n");
        dot
    }

    pub fn adj(&self, v: usize) -> bag::Iter<usize> {
        self.adj[v].iter()
    }
}

#[test]
fn test_graph() {
    let mut g = Graph::new(10);
    g.add_edge(0, 3);
    g.add_edge(0, 5);
    g.add_edge(4, 5);
    g.add_edge(2, 9);
    g.add_edge(2, 8);
    g.add_edge(3, 7);

    g.add_edge(1, 6);
    g.add_edge(6, 9);
    g.add_edge(5, 8);

    println!("got => \n{}", g.to_dot());

    assert_eq!(10, g.vertices());
    assert_eq!(9, g.edges());
    assert_eq!(3, g.degree(5));

    for w in g.adj(5) {
        assert!(vec![8, 4, 0].contains(w));
    }
}
