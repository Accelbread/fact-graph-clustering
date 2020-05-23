//! Adjacency matrix based graph implementation.

use crate::graph::{IndexMap,lower_triangular::LowerTriangular};
use serde::{Deserialize, Serialize};

/// Graph implementation based off of an adjacency matrix graph implementation.
/// Represents edges as a lower triangular matrix encoded as a jagged array, allowing for adding
/// nodes without existing rows.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AMGraph<E> {
    map: IndexMap,
    edges: LowerTriangular<Option<E>>,
}

impl<E> AMGraph<E> {
    /// Creates an empty `AMGraph` that allocates storage for all the verticies
    /// in `map`.
    pub fn new(map: IndexMap) -> Self {
        let vert_len = map.len();
        let edge_len = (vert_len * (vert_len + 1)) / 2;
        AMGraph {
            edges: LowerTriangular((0..edge_len).map(|_| None).collect()),
            map,
        }
    }

    /// Returns the number of verticies in the graph.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns an iterator over the verticies in the graph.
    pub fn vertices(&self) -> <&IndexMap as IntoIterator>::IntoIter {
        self.map.into_iter()
    }

    /// Returns an iterator over the edges of the graph.
    ///
    /// The return type is of the format (row, column, edge).
    pub fn edges(&self) -> Edges<E> {
        Edges {
            graph: self,
            row: 0,
            col: 0,
        }
    }

    /// Returns `true` if the graph contains no verticies.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns a reference to the edge between the given verticies.
    ///
    /// Return value will be `Err` if the verticies are not in the graph, otherwise the value will
    /// be `Ok`.
    pub fn get(&self, v1: &str, v2: &str) -> Result<&Option<E>, ()> {
        let v1 = self.map.get(v1).ok_or(())?;
        let v2 = self.map.get(v2).ok_or(())?;
        Ok(&self.edges[(v1, v2)])
    }

    /// Returns a mutable reference to the edge between the given verticies.
    ///
    /// Return value will be `Err` if the verticies are not in the graph, otherwise the value will
    /// be `Ok`.
    pub fn get_mut(&mut self, v1: &str, v2: &str) -> Result<&mut Option<E>, ()> {
        let v1 = self.map.get(v1).ok_or(())?;
        let v2 = self.map.get(v2).ok_or(())?;
        Ok(&mut self.edges[(v1, v2)])
    }

    /// Returns `true` if the graph contains the given vertex.
    pub fn contains_vertex(&self, v: &str) -> bool {
        self.map.get(v).is_some()
    }
}

/// An iterator over the edges of an `AMGraph`.
#[derive(Clone, Debug)]
pub struct Edges<'a, E> {
    graph: &'a AMGraph<E>,
    row: usize,
    col: usize,
}

impl<'a, E> Iterator for Edges<'a, E> {
    type Item = (String, String, &'a E);

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.graph.len();
        while self.row < len {
            let res = &self.graph.edges[(self.row, self.col)];
            let (row, col) = (self.row, self.col);
            self.col += 1;
            if self.col > self.row {
                self.row += 1;
                self.col = 0;
            }
            if let Some(e) = res {
                return Some((
                    self.graph.map.get(row).unwrap(),
                    self.graph.map.get(col).unwrap(),
                    e,
                ));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.graph.len();
        if self.row >= len {
            (0, Some(0))
        } else {
            let rem_cur_row = self.row - self.col + 1;
            let rem_rest_rows = (self.row + 1..len).map(|x| x + 1).sum::<usize>();
            (0, Some(rem_cur_row + rem_rest_rows))
        }
    }
}
