use anyhow::Context;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use git2::{Commit, Delta, Deltas, Diff, DiffDelta, DiffFlags, FileMode, Oid};
use git2::{DiffFile, Repository};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Style, Stylize, Widget};
use ratatui::text::{Line, Text, ToLine, ToSpan};
use ratatui::widgets::{Block, BorderType, Paragraph, StatefulWidgetRef, WidgetRef};
use ratatui::{DefaultTerminal, Frame};
use ratatui_tree::{tree_index, Tree, TreeIndex, TreeItem, TreeState, TreeView};
use std::collections::VecDeque;
use std::ops::{AddAssign, Deref, Index, SubAssign};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::RwLock;

// MARK: Extra

struct StackCommitDeltaFile {
    mode: FileMode,
    path: Option<PathBuf>,
}

impl<'a> From<DiffFile<'a>> for StackCommitDeltaFile {
    fn from(file: DiffFile<'a>) -> Self {
        Self {
            mode: file.mode(),
            path: file.path().map(PathBuf::from),
        }
    }
}

// MARK: Delta

struct DeltaNode {
    status: Delta,
    new_file: StackCommitDeltaFile,
    old_file: StackCommitDeltaFile,
    flags: DiffFlags,
}

impl From<DiffDelta<'_>> for DeltaNode {
    fn from(delta: DiffDelta<'_>) -> Self {
        Self {
            status: delta.status(),
            new_file: delta.new_file().into(),
            old_file: delta.old_file().into(),
            flags: delta.flags(),
        }
    }
}

impl<'a> Into<TreeItem<'a>> for &'a DeltaNode {
    fn into(self) -> TreeItem<'a> {
        let path = self
            .new_file
            .path
            .as_deref()
            .and_then(Path::to_str)
            .unwrap_or("?");
        TreeItem::new_empty(Line::from(path))
    }
}

// MARK: Commit

struct CommitNode<'repo> {
    commit: Commit<'repo>,
    diff: Option<Diff<'repo>>,
    deltas: Vec<Node<'repo>>,
    is_collapsed: bool,
}

impl<'repo> CommitNode<'repo> {
    pub fn get(&self, index: usize) -> Option<&DeltaNode> {
        self.deltas.get(index).map(Node::unwrap_delta_ref)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut DeltaNode> {
        self.deltas.get_mut(index).map(Node::unwrap_delta_mut)
    }

    pub fn push<T: Into<DeltaNode>>(&mut self, delta: T) {
        self.deltas.push(Node::Delta(delta.into()));
    }

    pub fn clear(&mut self) {
        self.deltas.clear();
    }

    pub fn len(&self) -> usize {
        self.deltas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }
}

impl<'repo> From<Commit<'repo>> for CommitNode<'repo> {
    fn from(commit: Commit<'repo>) -> Self {
        Self {
            commit,
            diff: None,
            deltas: Vec::new(),
            is_collapsed: true,
        }
    }
}

impl<'a, 'repo> Into<TreeItem<'a>> for &'a CommitNode<'repo> {
    fn into(self) -> TreeItem<'a> {
        let icon: &'static str = if self.is_collapsed { " + " } else { " - " };
        let mut hash = self.commit.id().to_string();
        hash.truncate(8);
        let message = self
            .commit
            .message()
            .and_then(|message| message.lines().next())
            .unwrap_or("");
        let label = Line::from(vec![icon.into(), hash.into(), " ".into(), message.into()]);
        if self.is_collapsed {
            TreeItem::new_empty(label)
        } else {
            TreeItem::new(label, self.deltas.iter())
        }
    }
}

impl<'repo> Index<usize> for CommitNode<'repo> {
    type Output = DeltaNode;

    fn index(&self, index: usize) -> &Self::Output {
        self.deltas.index(index).unwrap_delta_ref()
    }
}

// MARK: Node

enum Node<'repo> {
    Delta(DeltaNode),
    Commit(CommitNode<'repo>),
}

impl<'repo> Node<'repo> {
    pub fn unwrap_delta_ref(&self) -> &DeltaNode {
        match self {
            Node::Delta(delta) => delta,
            _ => panic!("node contains a commit"),
        }
    }

    pub fn unwrap_delta_mut(&mut self) -> &mut DeltaNode {
        match self {
            Node::Delta(delta) => delta,
            _ => panic!("node contains a delta"),
        }
    }

    pub fn unwrap_commit_ref(&self) -> &CommitNode<'repo> {
        match self {
            Node::Commit(commit) => commit,
            _ => panic!("node contains a delta"),
        }
    }

    pub fn unwrap_commit_mut(&mut self) -> &mut CommitNode<'repo> {
        match self {
            Node::Commit(commit) => commit,
            _ => panic!("node contains a delta"),
        }
    }
}

impl<'repo> From<DiffDelta<'_>> for Node<'repo> {
    fn from(delta: DiffDelta<'_>) -> Self {
        Node::Delta(delta.into())
    }
}

impl<'repo> From<Commit<'repo>> for Node<'repo> {
    fn from(commit: Commit<'repo>) -> Self {
        Node::Commit(commit.into())
    }
}

impl<'a, 'repo> Into<TreeItem<'a>> for &'a Node<'repo> {
    fn into(self) -> TreeItem<'a> {
        match self {
            Node::Delta(delta) => delta.into(),
            Node::Commit(commit) => commit.into(),
        }
    }
}

impl<'repo> TreeView<Node<'repo>> for Node<'repo> {
    type ChildIter<'a>
        = std::slice::Iter<'a, Node<'repo>>
    where
        Self: 'a;

    fn iter_children(&self) -> Self::ChildIter<'_> {
        match self {
            Node::Delta(_) => std::slice::Iter::default(),
            Node::Commit(commit) => commit.deltas.iter(),
        }
    }
}

// MARK: Stack

struct StackTree<'repo> {
    commits: Vec<Node<'repo>>,
}

impl<'repo> StackTree<'repo> {
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&CommitNode<'repo>> {
        self.commits.get(index).map(Node::unwrap_commit_ref)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut CommitNode<'repo>> {
        self.commits.get_mut(index).map(Node::unwrap_commit_mut)
    }

    pub fn push<T: Into<CommitNode<'repo>>>(&mut self, commit: T) {
        self.commits.push(Node::Commit(commit.into()));
    }

    pub fn clear(&mut self) {
        self.commits.clear();
    }

    pub fn len(&self) -> usize {
        self.commits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commits.is_empty()
    }
}

impl<'repo> Index<usize> for StackTree<'repo> {
    type Output = CommitNode<'repo>;

    fn index(&self, index: usize) -> &Self::Output {
        self.commits.index(index).unwrap_commit_ref()
    }
}

impl<'repo> TreeView<Node<'repo>> for StackTree<'repo> {
    type ChildIter<'a>
        = std::slice::Iter<'a, Node<'repo>>
    where
        Self: 'a;

    fn iter_children(&self) -> Self::ChildIter<'_> {
        self.commits.iter()
    }
}

// MARK: Model

struct Model<'repo> {
    repo: &'repo Repository,
    stack: StackTree<'repo>,
    tree: TreeState,
    preview: Paragraph<'static>,
}

impl<'repo> Model<'repo> {
    pub fn new(repo: &'repo Repository) -> Self {
        Self {
            repo,
            stack: StackTree::new(),
            tree: TreeState::new(),
            preview: Paragraph::new(""),
        }
    }

    fn load_commits_since_merge_base_with(&mut self, base: &str) -> anyhow::Result<()> {
        let base_id = self
            .repo
            .resolve_reference_from_short_name(base)
            .with_context(|| format!("failed to resolve base commit {}", base))?
            .target()
            .ok_or_else(|| anyhow::format_err!("no target OID for reference {}", base))?;
        let head_id = self
            .repo
            .head()
            .context("failed to resolve HEAD")?
            .target()
            .ok_or_else(|| anyhow::format_err!("no target OID for HEAD"))?;
        let merge_base_id = self
            .repo
            .merge_base(head_id, base_id)
            .with_context(|| format!("failed to resolve merge base between {} and HEAD", base))?;

        let mut revwalk = self
            .repo
            .revwalk()
            .context("failed to construct a revision walk")?;
        revwalk
            .push_head()
            .context("failed to push HEAD onto the revision walk")?;
        revwalk
            .set_sorting(git2::Sort::TOPOLOGICAL)
            .context("failed to sort the revision walk topologically")?;

        self.stack.clear();
        for result in revwalk {
            let id = result.context("failed to retrieve commit from revwalk")?;
            if id == merge_base_id {
                break;
            }
            let commit = self
                .repo
                .find_commit(id)
                .with_context(|| format!("failed to find commit {}", id))?;
            self.stack.push(commit);
        }

        self.stack.commits.reverse();
        if self.stack.is_empty() {
            self.tree.select(None)
        } else {
            self.tree.select(Some(TreeIndex::new(0)));
        }

        Ok(())
    }

    pub fn toggle_deltas(&mut self, commit_index: usize) -> anyhow::Result<()> {
        let Some(commit_node) = self.stack.get_mut(commit_index) else {
            // state.status = format!("invalid commit index {}", index);
            // TODO
            return Ok(());
        };

        if commit_node.diff.is_none() {
            if commit_node.commit.parent_count() != 1 {
                return Err(anyhow::format_err!(
                    "commit {} has {} parents",
                    commit_node.commit.id(),
                    commit_node.commit.parent_count()
                ));
            };

            let diff = self.repo.diff_tree_to_tree(
                Some(
                    &commit_node
                        .commit
                        .parent(0)
                        .context("failed to retrieve commit parent")?
                        .tree()
                        .context("failed to retrieve commit tree")?,
                ),
                Some(
                    &commit_node
                        .commit
                        .tree()
                        .context("failed to retrieve commit tree")?,
                ),
                None,
            )?;

            commit_node.deltas = diff.deltas().map(Node::from).collect();
            commit_node.diff = Some(diff);
        }

        commit_node.is_collapsed = !commit_node.is_collapsed;
        Ok(())
    }

    pub fn show_delta(&mut self, commit_index: usize, delta_index: usize) -> anyhow::Result<()> {
        let Some(stack_commit) = self.stack.get_mut(commit_index) else {
            // state.status = format!("invalid commit index {}", index);
            // TODO
            return Ok(());
        };

        let counter = RwLock::new(0usize);
        let lines = RwLock::new(Vec::new());
        if let Some(diff) = stack_commit.diff.as_ref() {
            diff.foreach(
                &mut |_delta, _x| {
                    counter.write().unwrap().add_assign(1);
                    true
                },
                Some(&mut |_delta, _binary| true),
                Some(&mut |delta, hunk| {
                    if *counter.read().unwrap() != delta_index + 1 {
                        return true;
                    }

                    let mut text = String::from_utf8_lossy(hunk.header()).to_string();
                    text.push_str("\n");
                    lines.write().unwrap().push(Line::from(text));
                    true
                }),
                Some(&mut |_delta, _hunk, line| {
                    if *counter.read().unwrap() != delta_index + 1 {
                        return true;
                    }

                    let mut style = Style::new();
                    let mut text = String::from_utf8_lossy(line.content()).to_string();
                    if line.old_lineno().is_some() && line.new_lineno().is_some() {
                        text.insert_str(0, "  ");
                    } else if line.new_lineno().is_some() {
                        text.insert_str(0, "+ ");
                        style = style.green();
                    } else if line.old_lineno().is_some() {
                        text.insert_str(0, "- ");
                        style = style.red();
                    }
                    lines.write().unwrap().push(Line::from(text).style(style));
                    true
                }),
            )?;
        }

        self.preview = Paragraph::new(lines.into_inner()?);
        Ok(())
    }
}

// MARK: Messaging

enum Message {
    Noop,
    Load(String),
    Terminal(Event),
    Exit,
}

// MARK: Controller

struct Controller<'repo> {
    model: Model<'repo>,
    queue: VecDeque<Message>,
}

impl<'repo> Controller<'repo> {
    fn new(model: Model<'repo>) -> Self {
        Self {
            model,
            queue: VecDeque::new(),
        }
    }

    fn message(&mut self, message: Message) {
        self.queue.push_back(message);
    }

    fn exit(&mut self) {
        self.queue.clear();
        self.message(Message::Exit);
    }

    fn select_up(&mut self) {
        self.model.tree.select(
            self.model
                .tree
                .selected()
                .as_ref()
                .and_then(|index| self.model.stack.find_previous_relative_of(index))
                .map(|(index, _)| index),
        );
    }

    fn select_down(&mut self) {
        self.model.tree.select(
            self.model
                .tree
                .selected()
                .as_ref()
                .and_then(|index| self.model.stack.find_next_relative_of(index))
                .map(|(index, _)| index),
        );
    }

    fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let message = self.queue.pop_front();

        match message.as_ref() {
            Some(Message::Terminal(Event::Key(key))) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if matches!(key.code, KeyCode::Char('c' | 'q' | 'x')) {
                        return Ok(self.exit());
                    }
                }
            }
            Some(Message::Load(base)) => self.model.load_commits_since_merge_base_with(base)?,
            _ => {}
        }

        let area = frame.area();
        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]);
        let [content_area, footer_area] = layout.areas(area);
        self.draw_content(content_area, frame.buffer_mut(), &message)?;
        self.draw_footer(footer_area, frame.buffer_mut(), &message)?;
        Ok(())
    }

    fn draw_content(
        &mut self,
        area: Rect,
        buffer: &mut Buffer,
        message: &Option<Message>,
    ) -> anyhow::Result<()> {
        match message.as_ref() {
            Some(Message::Terminal(Event::Key(key))) if key.is_press() => match key.code {
                KeyCode::Up => self.select_up(),
                KeyCode::Down => self.select_down(),
                KeyCode::Left => {}
                KeyCode::Right => {}
                KeyCode::Char(' ') => {
                    match self.model.tree.selected().as_ref().map(TreeIndex::as_slice) {
                        Some([commit_index]) => self.model.toggle_deltas(*commit_index)?,
                        Some([commit_index, file_index]) => {
                            self.model.show_delta(*commit_index, *file_index)?
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('q') => {
                    return Ok(self.exit());
                }
                _ => {}
            },
            _ => {}
        }

        let tree = Tree::new(&self.model.stack.commits)
            .indent_symbol("    ")
            .highlight_style(Style::new().bold().reversed())
            .block(
                Block::bordered()
                    .title("Commits")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            );

        let layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(2),
        ]);
        let [tree_area, _, preview_area] = layout.areas(area);
        StatefulWidget::render(tree, tree_area, buffer, &mut self.model.tree);

        self.model.preview.render_ref(preview_area, buffer);

        Ok(())
    }

    fn draw_footer(
        &mut self,
        area: Rect,
        buffer: &mut Buffer,
        message: &Option<Message>,
    ) -> anyhow::Result<()> {
        let layout = Layout::horizontal([Constraint::Fill(2), Constraint::Fill(1)]);
        let [tooltips_area, status_area] = layout.areas(area);

        let mut spans = Vec::new();
        // for tooltip in &state.tooltips {
        //     spans.push(" ".to_span());
        //     spans.push(tooltip.to_span());
        //     spans.push(" ".to_span());
        // }
        Line::from(spans)
            .style(Style::new().reversed())
            .alignment(Alignment::Left)
            .render(tooltips_area, buffer);

        // Line::from(state.status.to_span())
        //     .style(Style::new().reversed())
        //     .alignment(Alignment::Right)
        //     .render(status_area, buffer);

        Ok(())
    }
}

// MARK: Main

pub struct Options {
    pub base: String,
}

pub fn with_terminal<T, F: FnOnce(DefaultTerminal) -> T>(f: F) -> T {
    let terminal = ratatui::init();
    let result = f(terminal);
    ratatui::restore();
    result
}

pub fn main(repo: &Repository, options: Options) -> anyhow::Result<()> {
    with_terminal(|mut terminal| {
        let mut controller = Controller::new(Model::new(repo));
        controller.message(Message::Load(options.base));

        loop {
            let mut result = Ok(());
            terminal.draw(|frame| result = controller.draw(frame))?;
            result?;

            match controller.queue.front() {
                Some(Message::Exit) => break,
                None => controller.message(Message::Terminal(crossterm::event::read()?)),
                _ => {}
            }
        }

        Ok(())
    })
}
