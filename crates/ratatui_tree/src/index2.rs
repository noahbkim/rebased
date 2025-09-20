// Invariant: indices must have at least one element.

use std::borrow::Borrow;
use std::ops::Deref;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct TreeIndex([usize]);

impl TreeIndex {
    fn new_unchecked(indices: &[usize]) -> &Self {
        unsafe { &*(indices as *const [usize] as *const TreeIndex) }
    }

    pub fn new(indices: &[usize]) -> Option<&Self> {
        if indices.is_empty() {
            None
        } else {
            Some(Self::new_unchecked(indices))
        }
    }

    pub fn first(&self) -> usize {
        self.0[0]
    }

    pub fn last(&self) -> usize {
        self.0[self.0.len() - 1]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, usize> {
        self.0.iter()
    }

    pub fn iter_rest(&self) -> std::slice::Iter<'_, usize> {
        self.0[1..].iter()
    }

    pub fn parent(&self) -> Option<&Self> {
        self.0
            .len()
            .checked_sub(1)
            .map(|i| Self::new_unchecked(&self.0[..i]))
    }

    pub fn rest(&self) -> Option<&Self> {
        self.0
            .len()
            .gt(&1)
            .then(|| Self::new_unchecked(&self.0[1..]))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_root(&self) -> bool {
        self.0.len() == 1
    }

    pub fn as_slice(&self) -> &[usize] {
        &self.0
    }
}

impl<'a, const N: usize> From<&'a [usize; N]> for &'a TreeIndex {
    fn from(indices: &'a [usize; N]) -> Self {
        if N > 0 {
            TreeIndex::new_unchecked(indices)
        } else {
            panic!("tree index must contain at least one element");
        }
    }
}

impl ToOwned for TreeIndex {
    type Owned = TreeIndexBuf;

    fn to_owned(&self) -> Self::Owned {
        TreeIndexBuf::new_unchecked(self.0.to_owned())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct TreeIndexBuf(Vec<usize>);

impl TreeIndexBuf {
    fn new_unchecked(indices: Vec<usize>) -> Self {
        Self(indices)
    }

    pub fn first_mut(&mut self) -> &mut usize {
        self.0.first_mut().unwrap()
    }

    pub fn last_mut(&mut self) -> &mut usize {
        self.0.last_mut().unwrap()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, usize> {
        self.0.iter_mut()
    }

    pub fn iter_rest_mut(&mut self) -> std::slice::IterMut<'_, usize> {
        self.0[1..].iter_mut()
    }

    pub fn as_mut_slice(&mut self) -> &mut [usize] {
        &mut self.0
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
}

impl Borrow<TreeIndex> for TreeIndexBuf {
    fn borrow(&self) -> &TreeIndex {
        TreeIndex::new_unchecked(&self.0)
    }
}

impl Default for TreeIndexBuf {
    fn default() -> Self {
        Self(vec![0])
    }
}

impl Deref for TreeIndexBuf {
    type Target = TreeIndex;

    fn deref(&self) -> &Self::Target {
        TreeIndex::new_unchecked(&self.0)
    }
}
