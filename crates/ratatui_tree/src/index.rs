#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct TreeIndex(Vec<usize>);

impl Default for TreeIndex {
    fn default() -> Self {
        Self(vec![0])
    }
}

impl TreeIndex {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn new(index: usize) -> Self {
        Self(vec![index])
    }

    pub fn new_unchecked(indices: Vec<usize>) -> Self {
        Self(indices)
    }

    pub fn first(&self) -> usize {
        *self.first_ref()
    }

    pub fn first_ref(&self) -> &usize {
        self.0.first().unwrap()
    }

    pub fn first_mut(&mut self) -> &mut usize {
        self.0.first_mut().unwrap()
    }

    pub fn last(&self) -> usize {
        *self.last_ref()
    }

    pub fn last_ref(&self) -> &usize {
        self.0.last().unwrap()
    }

    pub fn last_mut(&mut self) -> &mut usize {
        self.0.last_mut().unwrap()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, usize> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, usize> {
        self.0.iter_mut()
    }

    pub fn iter_rest(&self) -> std::slice::Iter<'_, usize> {
        self.0[1..].iter()
    }

    pub fn iter_rest_mut(&mut self) -> std::slice::IterMut<'_, usize> {
        self.0[1..].iter_mut()
    }

    pub fn push(&mut self, index: usize) {
        self.0.push(index);
    }

    pub fn pushed(&self, index: usize) -> Self {
        let mut result = self.clone();
        result.push(index);
        result
    }

    pub fn pop(&mut self) -> Option<usize> {
        if self.0.len() > 1 {
            self.0.pop()
        } else {
            None
        }
    }

    pub fn popped(&self) -> Self {
        let mut result = self.clone();
        result.pop();
        result
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_root(&self) -> bool {
        self.0.len() == 1
    }

    pub fn splice(&mut self, place: usize, index: usize) {
        self.0.resize(place, 0);
        self.0.push(index);
    }

    pub fn spliced(&self, place: usize, index: usize) -> Self {
        let mut result = self.clone();
        result.splice(place, index);
        result
    }

    pub fn floor(&mut self, place: usize) {
        self.0.resize(place.saturating_add(1), 0);
    }

    pub fn floored(&self, place: usize) -> Self {
        let mut result = self.clone();
        result.floor(place);
        result
    }

    pub fn as_slice(&self) -> &[usize] {
        self.0.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [usize] {
        self.0.as_mut_slice()
    }
}

// Specifically does not permit an empty index.
#[macro_export]
macro_rules! tree_index {
    ($($index:expr),+) => { ::ratatui_tree::TreeIndex::new_unchecked(vec![$($index),+]) };
}
