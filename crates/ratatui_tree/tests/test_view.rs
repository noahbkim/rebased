use ratatui_tree::tree_index as I;
use ratatui_tree::TreeView;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Tree(&'static str, pub Vec<Tree>);

impl TreeView<Tree> for Tree {
    type ChildIter<'a> = std::slice::Iter<'a, Tree>;

    fn iter_children(&self) -> Self::ChildIter<'_> {
        self.1.iter()
    }
}

#[test]
fn test_iter_children() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.iter_descendants().count(), 0);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(
        tree.iter_children().map(|t| t.0).collect::<Vec<_>>(),
        vec!["a", "x"]
    );
}

#[test]
fn test_iter_descendants() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.iter_descendants().count(), 0);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(
        tree.iter_descendants().map(|t| t.0).collect::<Vec<_>>(),
        vec!["a", "b", "c", "x", "y", "z"]
    );
}

#[test]
fn test_iter_descendants_with_depth() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.iter_descendants_with_depth().count(), 0);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(
        tree.iter_descendants_with_depth()
            .map(|(d, t)| (d, t.0))
            .collect::<Vec<_>>(),
        vec![(0, "a"), (1, "b"), (1, "c"), (0, "x"), (1, "y"), (1, "z")]
    );
}

#[test]
fn test_iter_descendants_with_index() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.iter_descendants_with_index().count(), 0);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(
        tree.iter_descendants_with_index()
            .map(|(d, t)| (d, t.0))
            .collect::<Vec<_>>(),
        vec![
            (I![0], "a"),
            (I![0, 0], "b"),
            (I![0, 1], "c"),
            (I![1], "x"),
            (I![1, 0], "y"),
            (I![1, 1], "z")
        ]
    );
}

#[test]
fn test_get_child() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.get_child(0), None);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(tree.get_child(0).map(|t| t.0), Some("a"));
    assert_eq!(tree.get_child(1).map(|t| t.0), Some("x"));
    assert_eq!(tree.get_child(2).map(|t| t.0), None);
}

#[test]
fn test_get_descendant() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.get_descendant(&I![0]), None);
    assert_eq!(tree.get_descendant(&I![0, 0]), None);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(tree.get_descendant(&I![0]).map(|t| t.0), Some("a"));
    assert_eq!(tree.get_descendant(&I![0, 0]).map(|t| t.0), Some("b"));
    assert_eq!(tree.get_descendant(&I![0, 0, 0]).map(|t| t.0), None);
    assert_eq!(tree.get_descendant(&I![0, 1]).map(|t| t.0), Some("c"));
    assert_eq!(tree.get_descendant(&I![0, 1, 0]).map(|t| t.0), None);
    assert_eq!(tree.get_descendant(&I![0, 2]).map(|t| t.0), None);
    assert_eq!(tree.get_descendant(&I![1]).map(|t| t.0), Some("x"));
    assert_eq!(tree.get_descendant(&I![1, 0]).map(|t| t.0), Some("y"));
    assert_eq!(tree.get_descendant(&I![1, 0, 0]).map(|t| t.0), None);
    assert_eq!(tree.get_descendant(&I![1, 1]).map(|t| t.0), Some("z"));
    assert_eq!(tree.get_descendant(&I![1, 1, 0]).map(|t| t.0), None);
    assert_eq!(tree.get_descendant(&I![1, 2]).map(|t| t.0), None);
    assert_eq!(tree.get_descendant(&I![2]).map(|t| t.0), None);
}

#[test]
fn test_get_descendant_infix() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.get_descendant_infix(0), None);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(tree.get_descendant_infix(0).map(|t| t.0), Some("a"));
    assert_eq!(tree.get_descendant_infix(1).map(|t| t.0), Some("b"));
    assert_eq!(tree.get_descendant_infix(2).map(|t| t.0), Some("c"));
    assert_eq!(tree.get_descendant_infix(3).map(|t| t.0), Some("x"));
    assert_eq!(tree.get_descendant_infix(4).map(|t| t.0), Some("y"));
    assert_eq!(tree.get_descendant_infix(5).map(|t| t.0), Some("z"));
    assert_eq!(tree.get_descendant_infix(6).map(|t| t.0), None);
}

#[test]
fn test_find_index_of_offset() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.find_index_of_offset(0), None);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(tree.find_index_of_offset(0).map(|r| r.0), Some(I![0]));
    assert_eq!(tree.find_index_of_offset(1).map(|r| r.0), Some(I![0, 0]));
    assert_eq!(tree.find_index_of_offset(2).map(|r| r.0), Some(I![0, 1]));
    assert_eq!(tree.find_index_of_offset(3).map(|r| r.0), Some(I![1]));
    assert_eq!(tree.find_index_of_offset(4).map(|r| r.0), Some(I![1, 0]));
    assert_eq!(tree.find_index_of_offset(5).map(|r| r.0), Some(I![1, 1]));
    assert_eq!(tree.find_index_of_offset(6).map(|r| r.0), None);
}

#[test]
fn test_find_offset_of_index() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.find_offset_of_index(&I![0]), None);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(tree.find_offset_of_index(&I![0]).map(|r| r.0), Some(0));
    assert_eq!(tree.find_offset_of_index(&I![0, 0]).map(|r| r.0), Some(1));
    assert_eq!(tree.find_offset_of_index(&I![0, 0, 0]).map(|r| r.0), None);
    assert_eq!(tree.find_offset_of_index(&I![0, 1]).map(|r| r.0), Some(2));
    assert_eq!(tree.find_offset_of_index(&I![0, 1, 0]).map(|r| r.0), None);
    assert_eq!(tree.find_offset_of_index(&I![0, 2]).map(|r| r.0), None);
    assert_eq!(tree.find_offset_of_index(&I![1]).map(|r| r.0), Some(3));
    assert_eq!(tree.find_offset_of_index(&I![1, 0]).map(|r| r.0), Some(4));
    assert_eq!(tree.find_offset_of_index(&I![1, 0, 0]).map(|r| r.0), None);
    assert_eq!(tree.find_offset_of_index(&I![1, 1]).map(|r| r.0), Some(5));
    assert_eq!(tree.find_offset_of_index(&I![1, 1, 0]).map(|r| r.0), None);
    assert_eq!(tree.find_offset_of_index(&I![1, 2]).map(|r| r.0), None);
    assert_eq!(tree.find_offset_of_index(&I![2]).map(|r| r.0), None);
}

#[test]
fn test_find_nearest_to() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.find_nearest_to(&I![0]), None);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(tree.find_nearest_to(&I![0]).map(|r| r.0), Some(I![0]));
    assert_eq!(tree.find_nearest_to(&I![0, 0]).map(|r| r.0), Some(I![0, 0]));
    assert_eq!(
        tree.find_nearest_to(&I![0, 0, 0]).map(|r| r.0),
        Some(I![0, 0])
    );
    assert_eq!(tree.find_nearest_to(&I![0, 1]).map(|r| r.0), Some(I![0, 1]));
    assert_eq!(
        tree.find_nearest_to(&I![0, 1, 0]).map(|r| r.0),
        Some(I![0, 1])
    );
    assert_eq!(tree.find_nearest_to(&I![0, 2]).map(|r| r.0), Some(I![0, 1]));
    assert_eq!(tree.find_nearest_to(&I![1]).map(|r| r.0), Some(I![1]));
    assert_eq!(tree.find_nearest_to(&I![1, 0]).map(|r| r.0), Some(I![1, 0]));
    assert_eq!(
        tree.find_nearest_to(&I![1, 0, 0]).map(|r| r.0),
        Some(I![1, 0])
    );
    assert_eq!(tree.find_nearest_to(&I![1, 1]).map(|r| r.0), Some(I![1, 1]));
    assert_eq!(
        tree.find_nearest_to(&I![1, 1, 0]).map(|r| r.0),
        Some(I![1, 1])
    );
    assert_eq!(tree.find_nearest_to(&I![1, 2]).map(|r| r.0), Some(I![1, 1]));
    assert_eq!(tree.find_nearest_to(&I![2]).map(|r| r.0), Some(I![1]));
}

#[test]
fn find_next_relative_of() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.find_next_relative_of(&I![0]), None);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(
        tree.find_next_relative_of(&I![0]).map(|r| r.0),
        Some(I![0, 0])
    );
    assert_eq!(
        tree.find_next_relative_of(&I![0, 0]).map(|r| r.0),
        Some(I![0, 1])
    );
    assert_eq!(tree.find_next_relative_of(&I![0, 0, 0]).map(|r| r.0), None);
    assert_eq!(
        tree.find_next_relative_of(&I![0, 1]).map(|r| r.0),
        Some(I![1])
    );
    assert_eq!(tree.find_next_relative_of(&I![0, 1, 0]).map(|r| r.0), None);
    assert_eq!(tree.find_next_relative_of(&I![0, 2]).map(|r| r.0), None);
    assert_eq!(
        tree.find_next_relative_of(&I![1]).map(|r| r.0),
        Some(I![1, 0])
    );
    assert_eq!(
        tree.find_next_relative_of(&I![1, 0]).map(|r| r.0),
        Some(I![1, 1])
    );
    assert_eq!(tree.find_next_relative_of(&I![1, 0, 0]).map(|r| r.0), None);
    assert_eq!(tree.find_next_relative_of(&I![1, 1]).map(|r| r.0), None);
    assert_eq!(tree.find_next_relative_of(&I![1, 1, 0]).map(|r| r.0), None);
    assert_eq!(tree.find_next_relative_of(&I![1, 2]).map(|r| r.0), None);
    assert_eq!(tree.find_next_relative_of(&I![2]).map(|r| r.0), None);
}

#[test]
fn find_previous_relative_of() {
    let tree = Tree("root", vec![]);
    assert_eq!(tree.find_previous_relative_of(&I![0]), None);

    let tree = Tree(
        "root",
        vec![
            Tree("a", vec![Tree("b", vec![]), Tree("c", vec![])]),
            Tree("x", vec![Tree("y", vec![]), Tree("z", vec![])]),
        ],
    );

    assert_eq!(tree.find_previous_relative_of(&I![0]).map(|r| r.0), None);
    assert_eq!(
        tree.find_previous_relative_of(&I![0, 0]).map(|r| r.0),
        Some(I![0])
    );
    assert_eq!(
        tree.find_previous_relative_of(&I![0, 0, 0]).map(|r| r.0),
        None
    );
    assert_eq!(
        tree.find_previous_relative_of(&I![0, 1]).map(|r| r.0),
        Some(I![0, 0])
    );
    assert_eq!(
        tree.find_previous_relative_of(&I![0, 1, 0]).map(|r| r.0),
        None
    );
    assert_eq!(tree.find_previous_relative_of(&I![0, 2]).map(|r| r.0), None);
    assert_eq!(
        tree.find_previous_relative_of(&I![1]).map(|r| r.0),
        Some(I![0, 1])
    );
    assert_eq!(
        tree.find_previous_relative_of(&I![1, 0]).map(|r| r.0),
        Some(I![1])
    );
    assert_eq!(
        tree.find_previous_relative_of(&I![1, 0, 0]).map(|r| r.0),
        None
    );
    assert_eq!(
        tree.find_previous_relative_of(&I![1, 1]).map(|r| r.0),
        Some(I![1, 0])
    );
    assert_eq!(
        tree.find_previous_relative_of(&I![1, 1, 0]).map(|r| r.0),
        None
    );
    assert_eq!(tree.find_previous_relative_of(&I![1, 2]).map(|r| r.0), None);
    assert_eq!(tree.find_previous_relative_of(&I![2]).map(|r| r.0), None);
}
