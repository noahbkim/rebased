use crate::TreeIndex;
use crate::TreeIter;
use crate::TreeIterWithDepth;
use crate::TreeIterWithIndex;

pub trait TreeView<T: TreeView<T>> {
    type ChildIter<'a>: ExactSizeIterator<Item = &'a T> + DoubleEndedIterator<Item = &'a T>
    where
        Self: 'a,
        T: 'a;

    fn iter_children(&self) -> Self::ChildIter<'_>;

    fn iter_descendants(&self) -> TreeIter<'_, T> {
        TreeIter::new_from(self)
    }

    fn iter_descendants_with_depth(&self) -> TreeIterWithDepth<T> {
        TreeIterWithDepth::new_from(self)
    }

    fn iter_descendants_with_index(&self) -> TreeIterWithIndex<T> {
        TreeIterWithIndex::new_from(self)
    }

    fn first_child(&self) -> Option<&T> {
        self.iter_children().next()
    }

    fn last_child(&self) -> Option<&T> {
        self.iter_children().last()
    }

    fn get_child(&self, index: usize) -> Option<&T> {
        self.iter_children().nth(index)
    }

    fn get_descendant(&self, index: &TreeIndex) -> Option<&T> {
        let mut cursor = self.get_child(index.first())?;
        for i in index.iter_rest() {
            cursor = cursor.get_child(*i)?;
        }
        Some(cursor)
    }

    fn get_descendant_infix(&self, offset: usize) -> Option<&T> {
        self.iter_descendants().nth(offset) // Could be made more efficient.
    }

    fn len_children(&self) -> usize {
        self.iter_children().len()
    }

    fn len_descendants(&self) -> usize {
        self.iter_children()
            .map(|child| 1 + child.len_descendants())
            .sum::<usize>()
    }

    fn is_empty(&self) -> bool {
        self.iter_children().len() == 0
    }

    fn find_index_of_offset(&self, offset: usize) -> Option<(TreeIndex, &T)> {
        self.iter_descendants_with_index().nth(offset)
    }

    fn find_offset_of_index(&self, index: &TreeIndex) -> Option<(usize, &T)> {
        self.iter_descendants_with_index()
            .enumerate()
            .find_map(|(offset, (i, item))| match i.cmp(&index) {
                std::cmp::Ordering::Less => None,
                std::cmp::Ordering::Equal => return Some(Some((offset, item))),
                std::cmp::Ordering::Greater => return Some(None),
            })
            .flatten()
    }

    fn find_nearest_to(&self, origin: &TreeIndex) -> Option<(TreeIndex, &T)> {
        if self.is_empty() {
            return None;
        }

        let clamped = origin.first().min(self.len_children() - 1);
        let mut index = TreeIndex::new_at(clamped);
        let mut cursor = self.get_child(clamped).unwrap();
        for &i in origin.iter_rest() {
            if cursor.is_empty() {
                return Some((index, cursor));
            } else {
                let clamped = i.min(cursor.len_children() - 1);
                index.push(clamped);
                cursor = cursor.get_child(clamped).unwrap();
            }
        }

        Some((index, cursor))
    }

    fn find_first_child(&self) -> Option<(usize, &T)> {
        self.get_child(0).map(|child| (0, child))
    }

    fn find_first_descendant(&self) -> Option<(TreeIndex, &T)> {
        self.get_child(0).map(|child| (TreeIndex::new_at(0), child))
    }

    fn find_last_descendant_in(&self, mut index: TreeIndex) -> Option<(TreeIndex, &T)> {
        let mut cursor = self.get_descendant(&index)?;
        while !cursor.is_empty() {
            let i = cursor.len_children() - 1;
            index.push(i);
            cursor = cursor.get_child(i)?;
        }

        Some((index, cursor))
    }

    fn find_last_child(&self) -> Option<(usize, &T)> {
        let i = self.len_children() - 1;
        self.get_child(0).map(|child| (i, child))
    }

    fn find_last_descendant(&self) -> Option<(TreeIndex, &T)> {
        self.find_last_descendant_in(TreeIndex::new_at(self.len_children().saturating_sub(1)))
    }

    fn find_previous_child_to(&self, index: usize) -> Option<(usize, &T)> {
        if index > 0 {
            let previous_index = index - 1;
            self.get_child(previous_index)
                .map(|child| (previous_index, child))
        } else {
            None
        }
    }

    fn find_previous_sibling_of(&self, index: &TreeIndex) -> Option<(TreeIndex, &T)> {
        if index.is_root() {
            self.find_previous_child_to(index.first())
                .map(|(i, child)| (TreeIndex::new_at(i), child))
        } else {
            let parent_index = index.popped();
            self.get_descendant(&parent_index).and_then(|parent| {
                parent
                    .find_previous_child_to(index.last())
                    .map(|(i, child)| (parent_index.pushed(i), child))
            })
        }
    }

    fn find_previous_relative_of(&self, index: &TreeIndex) -> Option<(TreeIndex, &T)> {
        let mut cursor_index = TreeIndex::new_at(index.first());
        let mut cursor = self.get_child(index.first())?;
        let mut previous = self
            .find_previous_child_to(index.first())
            .and_then(|(x, _)| self.find_last_descendant_in(index.spliced(0, x)));

        for &i in index.iter().skip(1) {
            previous = match cursor.find_previous_child_to(i) {
                Some((j, _)) => self.find_last_descendant_in(cursor_index.pushed(j)),
                None => Some((cursor_index.clone(), cursor)),
            };

            cursor_index.push(i);
            cursor = cursor.get_child(i)?;
        }

        previous
    }

    fn find_next_child_to(&self, index: usize) -> Option<(usize, &T)> {
        let next_index = index + 1;
        if next_index < self.len_children() {
            self.get_child(next_index).map(|child| (next_index, child))
        } else {
            None
        }
    }

    fn find_next_sibling_of(&self, index: &TreeIndex) -> Option<(TreeIndex, &T)> {
        if index.is_root() {
            self.find_next_child_to(index.first())
                .map(|(i, child)| (TreeIndex::new_at(i), child))
        } else {
            let parent_index = index.popped();
            self.get_descendant(&parent_index).and_then(|parent| {
                parent
                    .find_next_child_to(index.last())
                    .map(|(i, child)| (parent_index.pushed(i), child))
            })
        }
    }

    fn find_next_relative_of(&self, index: &TreeIndex) -> Option<(TreeIndex, &T)> {
        let mut cursor = self.get_child(index.first())?;
        let mut next = self
            .find_next_child_to(index.first())
            .map(|(x, sibling)| (index.spliced(0, x), sibling));

        for (place, &i) in index.iter().enumerate().skip(1) {
            cursor
                .find_next_child_to(i)
                .map(|(x, sibling)| next.insert((index.spliced(place, x), sibling)));
            cursor = cursor.get_child(i)?;
        }

        cursor
            .find_first_child()
            .map(|(i, child)| (index.pushed(i), child))
            .or(next)
    }

    fn find_parent_of(&self, mut index: TreeIndex) -> Option<(TreeIndex, &T)> {
        if index.pop().is_some() {
            self.get_descendant(&index).map(|parent| (index, parent))
        } else {
            None
        }
    }
}
