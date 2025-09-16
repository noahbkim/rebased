use crate::TreeIndex;
use crate::TreeView;

#[derive(Clone, Debug)]
pub struct TreeIter<'a, T>(Vec<&'a T>);

impl<'a, T: TreeView<T>> TreeIter<'a, T> {
    pub fn new(stack: Vec<&'a T>) -> Self {
        Self(stack)
    }

    pub fn new_from<R: TreeView<T> + ?Sized>(root: &'a R) -> Self {
        Self::new(root.iter_children().rev().collect())
    }
}

impl<'a, T: TreeView<T>> Iterator for TreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop().map(|item| {
            self.0.extend(item.iter_children().rev());
            item
        })
    }
}

#[derive(Clone, Debug)]
pub struct TreeIterWithDepth<'a, T>(Vec<(usize, &'a T)>);

impl<'a, T: TreeView<T>> TreeIterWithDepth<'a, T> {
    pub fn new(stack: Vec<(usize, &'a T)>) -> Self {
        Self(stack)
    }

    pub fn new_from<R: TreeView<T> + ?Sized>(root: &'a R) -> Self {
        Self::new(root.iter_children().rev().map(|child| (0, child)).collect())
    }
}

impl<'a, T: TreeView<T>> Iterator for TreeIterWithDepth<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop().map(|(depth, item)| {
            self.0
                .extend(item.iter_children().rev().map(|child| (depth + 1, child)));
            (depth, item)
        })
    }
}

#[derive(Clone, Debug)]
pub struct TreeIterWithIndex<'a, T>(Vec<(TreeIndex, &'a T)>);

impl<'a, T: TreeView<T>> TreeIterWithIndex<'a, T> {
    pub fn new(stack: Vec<(TreeIndex, &'a T)>) -> Self {
        Self(stack)
    }

    pub fn new_from<R: TreeView<T> + ?Sized>(root: &'a R) -> Self {
        Self::new(
            root.iter_children()
                .enumerate()
                .rev()
                .map(|(i, child)| (TreeIndex::new_at(i), child))
                .collect(),
        )
    }
}

impl<'a, T: TreeView<T>> Iterator for TreeIterWithIndex<'a, T> {
    type Item = (TreeIndex, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop().map(|(index, item)| {
            self.0.extend(
                item.iter_children()
                    .enumerate()
                    .rev()
                    .map(|(i, child)| (index.pushed(i), child)),
            );
            (index, item)
        })
    }
}
