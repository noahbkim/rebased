use anyhow::Context;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use git2::Commit;
use git2::DiffDelta;
use git2::Oid;
use git2::Repository;
use git2::Time;
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::ToSpan;
use ratatui::text::ToText;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Paragraph;
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::Widget;
use ratatui::widgets::WidgetRef;
use ratatui::DefaultTerminal;
use ratatui_tree::TreeIndex;

use std::path::Path;
use std::path::PathBuf;

use ratatui_tree::tree_index;
use ratatui_tree::Tree;
use ratatui_tree::TreeItem;
use ratatui_tree::TreeState;

// MARK: ViewState

struct ViewState {
    tree: TreeState,
    tooltips: Vec<String>,
    status: String,
}

impl ViewState {
    fn new() -> Self {
        Self {
            tree: TreeState::new(),
            tooltips: Vec::new(),
            status: String::new(),
        }
    }
}

// MARK: View

struct View<'a> {
    tree: Tree<'a>,
}

impl<'a> View<'a> {
    fn new(tree: Tree<'a>) -> Self {
        Self { tree }
    }

    fn new_empty() -> Self {
        Self {
            tree: Tree::new_empty(),
        }
    }

    fn render(&self, area: Rect, buffer: &mut Buffer, state: &mut ViewState) {
        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]);
        let [content_area, footer_area] = layout.areas(area);
        self.render_content(content_area, buffer, state);
        self.render_footer(footer_area, buffer, state);
    }

    fn render_content(&self, area: Rect, buffer: &mut Buffer, state: &mut ViewState) {
        let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]);
        let [tree_area, preview_area] = layout.areas(area);
        StatefulWidgetRef::render_ref(&self.tree, tree_area, buffer, &mut state.tree);
    }

    fn render_footer(&self, area: Rect, buffer: &mut Buffer, state: &mut ViewState) {
        let layout = Layout::horizontal([Constraint::Fill(2), Constraint::Fill(1)]);
        let [tooltips_area, status_area] = layout.areas(area);

        let mut spans = Vec::new();
        for tooltip in &state.tooltips {
            spans.push(" ".to_span());
            spans.push(tooltip.to_span());
            spans.push(" ".to_span());
        }
        Line::from(spans)
            .style(Style::new().reversed())
            .alignment(Alignment::Left)
            .render(tooltips_area, buffer);

        Line::from(state.status.to_span())
            .style(Style::new().reversed())
            .alignment(Alignment::Right)
            .render(status_area, buffer);
    }
}

// MARK: App

fn ctrl(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
}

#[derive(Clone, Debug, Default)]
struct CommitFileNode {
    path: Option<PathBuf>,
    path_formatted: String,
}

impl<'a> From<DiffDelta<'a>> for CommitFileNode {
    fn from(delta: DiffDelta<'a>) -> Self {
        let path = delta.new_file().path();
        Self {
            path: path.map(Path::to_owned),
            path_formatted: path
                .and_then(|path| path.to_str())
                .unwrap_or("?")
                .to_owned(),
        }
    }
}

#[derive(Clone)]
struct CommitNode {
    oid: Oid,
    oid_formatted: String,
    time: Time,
    time_formatted: String,
    message: String,
    files: Option<Vec<CommitFileNode>>,
    is_collapsed: bool,
}

impl<'a> From<Commit<'a>> for CommitNode {
    fn from(commit: Commit<'a>) -> Self {
        let oid = commit.id();
        let mut oid_formatted = commit.id().to_string();
        oid_formatted.drain(8..);
        let time = commit.time();
        let time_formatted = chrono::DateTime::from_timestamp(time.seconds(), 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        Self {
            oid,
            oid_formatted,
            time,
            time_formatted,
            message: commit.message().unwrap_or("").to_owned(),
            files: None,
            is_collapsed: true,
        }
    }
}

struct App {
    repository: Repository,
    commits: Vec<CommitNode>,
}

impl App {
    pub fn new(repository: Repository) -> Self {
        Self {
            repository,
            commits: Vec::new(),
        }
    }

    pub fn load(&mut self, base: String) -> anyhow::Result<()> {
        let base_oid = self
            .repository
            .resolve_reference_from_short_name(base.as_str())
            .with_context(|| format!("failed to resolve base reference {}", base))?
            .target()
            .ok_or(anyhow::format_err!("no target OID for {}", base))?;
        let head_oid = self
            .repository
            .head()
            .context("failed to resolve HEAD")?
            .target()
            .ok_or(anyhow::format_err!("no target OID for HEAD"))?;
        let merge_base_oid = self
            .repository
            .merge_base(base_oid, head_oid)
            .with_context(|| format!("failed to resolve merge base with {} and HEAD", base))?;

        let mut revwalk = self
            .repository
            .revwalk()
            .context("failed to construct a revision iterator")?;
        revwalk
            .push_head()
            .context("failed to push HEAD onto the revision walk")?;
        revwalk
            .set_sorting(git2::Sort::TOPOLOGICAL)
            .context("failed to sort the revision walk topologically")?;

        self.commits.clear();
        for oid in revwalk {
            let oid = oid.context("invalid OID returned by revision walk")?;
            if oid == merge_base_oid {
                break;
            }
            let commit = self.repository.find_commit(oid)?;
            self.commits.push(CommitNode::from(commit));
        }

        Ok(())
    }

    pub fn view(&self) -> View<'_> {
        let mut children = Vec::new();
        for commit in self.commits.iter() {
            let mut item = TreeItem::new(Line::from(vec![
                if commit.is_collapsed {
                    " + ".to_span()
                } else {
                    " - ".to_span()
                },
                commit.oid_formatted.to_span().bold(),
                " ".to_span(),
                commit.message.to_span(),
            ]));

            if !commit.is_collapsed {
                let mut files = Vec::new();
                for file in commit.files.as_ref().unwrap().iter() {
                    files.push(TreeItem::new(file.path_formatted.to_span()));
                }

                item = item.children(files);
            }

            children.push(item);
        }

        View::new(
            Tree::new(children)
                .indent_symbol("    ")
                .highlight_style(Style::new().bold().reversed())
                .block(
                    Block::bordered()
                        .title("Commits")
                        .title_alignment(Alignment::Center)
                        .border_type(BorderType::Rounded),
                ),
        )
    }

    pub fn toggle_commit(&mut self, index: usize, state: &mut ViewState) {
        let Some(node) = self.commits.get_mut(index) else {
            state.status = format!("invalid commit index {}", index);
            return;
        };

        if node.files.is_none() {
            let Ok(commit) = self.repository.find_commit(node.oid) else {
                state.status = format!("failed to find commit {}", node.oid);
                return;
            };
            let diff = match crate::common::diff(&self.repository, &commit) {
                Ok(diff) => diff,
                Err(error) => {
                    state.status = format!("{}", error);
                    return;
                }
            };
            let mut files = Vec::new();
            for delta in diff.deltas() {
                files.push(CommitFileNode::from(delta));
            }
            node.files = Some(files);
        }

        node.is_collapsed = !node.is_collapsed;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let mut state = ViewState::new();
        state.tooltips.push("q Quit".to_owned());

        if !self.commits.is_empty() {
            state.tree.select(Some(tree_index![0]));
        }

        loop {
            let view = self.view();
            terminal.draw(|frame| view.render(frame.area(), frame.buffer_mut(), &mut state))?;
            match crossterm::event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if ctrl(key) => break,
                    KeyCode::Char('d') if ctrl(key) => break,
                    KeyCode::Left => view.tree.select_parent_state(&mut state.tree),
                    KeyCode::Up => view.tree.select_up_state(&mut state.tree),
                    KeyCode::Down => view.tree.select_down_state(&mut state.tree),
                    KeyCode::Char(' ') => {
                        match state.tree.selected().as_ref().map(TreeIndex::as_slice) {
                            Some([commit_index]) => self.toggle_commit(*commit_index, &mut state),
                            Some([commit_index, file_index]) => {}
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }
}

// MARK: Main

pub fn main(repository: Repository, base: String) -> anyhow::Result<()> {
    let mut app = App::new(repository);
    app.load(base)?;

    let mut terminal = ratatui::init();
    app.run(&mut terminal)?;
    ratatui::restore();

    return Ok(());
}
