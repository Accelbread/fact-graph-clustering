//! Operations for indexing an array as a lower triangular matrix

use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut, Index, IndexMut};

newtype_deref! {
    /// Type representing a lower triangualar matrix using a `Vec`.
    /// Provides 2 dimensional indexing for the `Vec`
    #[derive(Default, Clone, Debug, Serialize, Deserialize)]
    pub(crate) struct LowerTriangular<T>(pub(crate) Vec<T>);
}

impl<T> Index<(usize, usize)> for LowerTriangular<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let row = std::cmp::max(index.0, index.1);
        let col = std::cmp::min(index.0, index.1);
        &self.0[(row * (row + 1)) / 2 + col]
    }
}

impl<T> IndexMut<(usize, usize)> for LowerTriangular<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let row = std::cmp::max(index.0, index.1);
        let col = std::cmp::min(index.0, index.1);
        &mut self.0[(row * (row + 1)) / 2 + col]
    }
}
