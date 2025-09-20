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

use crate::stack::Stack;

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

struct App<'repo> {
    repository: &'repo Repository,
    stack: Stack<'repo>,
}

impl<'repo> App<'repo> {
    pub fn new(repository: &'repo Repository) -> Self {
        Self {
            repository,
            stack: Stack::new(),
        }
    }

    pub fn new_with(repository: &'repo Repository, stack: Stack<'repo>) -> Self {
        Self { repository, stack }
    }

    pub fn view(&self) -> View<'_> {}

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
                Event::Key(key) if key.is_press() => match key.code {
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
