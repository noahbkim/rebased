use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub struct DiffState {}

pub struct DiffLine<'a> {
    inner: Line<'a>,
}

pub struct Diff<'a> {
    inner: Paragraph<'a>,
}
