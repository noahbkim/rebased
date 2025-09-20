use std::collections::VecDeque;
use std::ops::{Index, SubAssign};
use std::path::Path;

use git2::{Commit, Oid};
use git2::{DiffFile, Repository};

use anyhow::Context as _;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Style, Stylize, Widget};
use ratatui::text::{Line, ToSpan};
use ratatui::widgets::{Block, BorderType, Paragraph, StatefulWidgetRef};
use ratatui::{DefaultTerminal, Frame};
use ratatui_tree::{tree_index, Tree, TreeIndex, TreeItem, TreeState, TreeView};

// MARK: Stack

#[derive(Debug)]
pub struct StackCommitFile<'repo> {
    pub(crate) inner: DiffFile<'repo>,
}

impl<'a, 'repo> Into<TreeItem<'a>> for &'a StackCommitFile<'repo> {
    fn into(self) -> TreeItem<'a> {
        let path = self.inner.path().and_then(Path::to_str).unwrap_or("?");
        TreeItem::new_empty(Line::from(path))
    }
}

#[derive(Debug)]
pub struct StackCommit<'repo> {
    pub(crate) inner: Commit<'repo>,
    pub(crate) files: Vec<StackCommitFile<'repo>>,
    pub(crate) is_collapsed: bool,
}

impl<'repo> StackCommit<'repo> {
    pub fn new(inner: Commit<'repo>) -> Self {
        Self {
            inner,
            files: Vec::new(),
            is_collapsed: false,
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl<'a, 'repo> Into<TreeItem<'a>> for &'a StackCommit<'repo> {
    fn into(self) -> TreeItem<'a> {
        let icon: &'static str = if self.is_collapsed { " + " } else { " - " };
        let mut hash = self.inner.id().to_string();
        hash.truncate(8);
        let message = self
            .inner
            .message()
            .and_then(|message| message.lines().next())
            .unwrap_or("");
        let label = Line::from(vec![icon.into(), hash.into(), " ".into(), message.into()]);
        if self.is_collapsed {
            TreeItem::new_empty(label)
        } else {
            TreeItem::new(label, self.files.iter())
        }
    }
}

impl<'repo> Index<usize> for StackCommit<'repo> {
    type Output = StackCommitFile<'repo>;

    fn index(&self, index: usize) -> &Self::Output {
        self.files.index(index)
    }
}

#[derive(Debug, Default)]
pub struct Stack<'repo> {
    pub(crate) commits: Vec<StackCommit<'repo>>,
}

impl<'repo> Stack<'repo> {
    pub fn new() -> Stack<'repo> {
        Self::default()
    }

    pub fn push(&mut self, commit: Commit<'repo>) {
        self.commits.push(StackCommit::new(commit));
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

impl<'repo> Index<usize> for Stack<'repo> {
    type Output = StackCommit<'repo>;

    fn index(&self, index: usize) -> &Self::Output {
        self.commits.index(index)
    }
}

// MARK: Model

struct Model<'repo> {
    repo: &'repo Repository,
    stack: Stack<'repo>,
    tree: TreeState,
}

impl<'repo> Model<'repo> {
    pub fn new(repo: &'repo Repository) -> Self {
        Self {
            repo,
            stack: Stack::new(),
            tree: TreeState::new(),
        }
    }

    pub fn select_up(&mut self) {
        if let Some(selected) = self.tree.selected_mut() {
            match selected.as_mut_slice() {
                [0] => {}
                [i] => {
                    *i -= 1;
                    self.stack[*i]
                        .len()
                        .checked_sub(1)
                        .map(|i| selected.push(i));
                }
                [i, 0] => {
                    selected.pop();
                }
                [i, j] => *j -= 1,
                _ => {}
            };
        };
    }

    pub fn select_down(&mut self) {
        if let Some(selected) = self.tree.selected_mut() {
            match selected.as_mut_slice() {
                [i] => {
                    if self.stack[*i].is_empty() {
                        *i = (*i + 1).min(self.stack.len().saturating_sub(1))
                    } else {
                        selected.push(0)
                    }
                }
                [i, j] => {
                    if j.saturating_add(1) < self.stack[*i].len() {
                        *j += 1;
                    } else if i.saturating_add(1) < self.stack.len() {
                        *i += 1;
                        selected.pop();
                    }
                }
                _ => {}
            };
        };
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

struct Controller<'a, 'repo> {
    repo: &'repo Repository,
    model: &'a mut Model<'repo>,
    queue: VecDeque<Message>,
}

impl<'a, 'repo> Controller<'a, 'repo> {
    fn new(repo: &'repo Repository, model: &'a mut Model<'repo>) -> Self {
        Self {
            repo,
            model,
            queue: VecDeque::new(),
        }
    }

    fn message(&mut self, message: Message) {
        self.queue.push_back(message);
    }

    fn load(&mut self, base: &str) -> anyhow::Result<()> {
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

        self.model.stack.clear();
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
        for result in revwalk {
            let id = result.context("failed to retrieve commit from revwalk")?;
            if id == merge_base_id {
                break;
            }
            let commit = self
                .repo
                .find_commit(id)
                .with_context(|| format!("failed to find commit {}", id))?;
            self.model.stack.push(commit);
        }

        if self.model.stack.is_empty() {
            self.model.tree.select(None)
        } else {
            self.model.tree.select(Some(TreeIndex::new(0)));
        }

        Ok(())
    }

    fn toggle(&mut self) {
        // let Some(node) = self.commits.get_mut(index) else {
        //     state.status = format!("invalid commit index {}", index);
        //     return;
        // };
        //
        // if node.files.is_none() {
        //     let Ok(commit) = self.repository.find_commit(node.oid) else {
        //         state.status = format!("failed to find commit {}", node.oid);
        //         return;
        //     };
        //     let diff = match crate::common::diff(&self.repository, &commit) {
        //         Ok(diff) => diff,
        //         Err(error) => {
        //             state.status = format!("{}", error);
        //             return;
        //         }
        //     };
        //     let mut files = Vec::new();
        //     for delta in diff.deltas() {
        //         files.push(CommitFileNode::from(delta));
        //     }
        //     node.files = Some(files);
        // }
        //
        // node.is_collapsed = !node.is_collapsed;
        //
    }

    fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let message = self.queue.pop_front();

        match message.as_ref() {
            Some(Message::Terminal(Event::Key(key))) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if matches!(key.code, KeyCode::Char('c' | 'q' | 'x')) {
                        self.queue.clear();
                        self.message(Message::Exit);
                        return Ok(());
                    }
                }
            }
            Some(Message::Load(base)) => self.load(base)?,
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
            Some(Message::Terminal(Event::Key(key))) => match key.code {
                KeyCode::Up => self.model.select_up(),
                KeyCode::Down => self.model.select_down(),
                KeyCode::Left => {}
                KeyCode::Right => {}
                KeyCode::Char(' ') => {
                    // match self.model.tree.selected().as_ref().map(TreeIndex::as_slice) {
                    //     Some([commit_index]) => self.,
                    //     Some([commit_index, file_index]) => {}
                    //     _ => {}
                    // }
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

        let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]);
        let [tree_area, preview_area] = layout.areas(area);
        StatefulWidget::render(tree, tree_area, buffer, &mut self.model.tree);

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
        let mut model = Model::new(repo);
        let mut context = Controller::new(&repo, &mut model);
        context.message(Message::Load(options.base));

        loop {
            let mut result = Ok(());
            terminal.draw(|frame| result = context.draw(frame))?;
            result?;

            match context.queue.front() {
                Some(Message::Exit) => break,
                None => context.message(Message::Terminal(crossterm::event::read()?)),
                _ => {}
            }
        }

        Ok(())
    })
}
