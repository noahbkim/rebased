use crate::index2::{TreeIndex, TreeIndexBuf};

pub trait ChildrenView {
    type Child<'b>
    where
        Self: 'b;

    fn get(&self, index: usize) -> Option<Self::Child<'_>>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn first(&self) -> Option<Self::Child<'_>> {
        self.get(0)
    }

    fn last(&self) -> Option<Self::Child<'_>> {
        self.len().checked_sub(1).and_then(|i| self.get(i))
    }
}

pub trait ParentView {
    type Children<'b>: ChildrenView
    where
        Self: 'b;

    fn children(&self) -> Self::Children<'_>;

    // fn first_child(&self) -> Option<<Self::Children<'_> as ChildrenView>::Child<'_>> {
    //     self.children().first()
    // }
}
