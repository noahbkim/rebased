use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::style::Styled;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::HighlightSpacing;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::Widget;
use ratatui::widgets::WidgetRef;
use unicode_width::UnicodeWidthStr;

use crate::TreeIndex;
use crate::TreeView;

// MARK: TreeItem

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TreeItem<'a> {
    pub(crate) content: Text<'a>,
    pub(crate) style: Style,
    pub(crate) children: Vec<TreeItem<'a>>,
}

impl<'a> TreeItem<'a> {
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            content: content.into(),
            style: Style::default(),
            children: Vec::default(),
        }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn children<C: Into<Vec<TreeItem<'a>>>>(mut self, children: C) -> Self {
        self.children = children.into();
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn child<C: Into<TreeItem<'a>>>(mut self, child: C) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn height(&self) -> usize {
        self.content.height()
    }

    pub fn width(&self) -> usize {
        self.content.width()
    }
}

impl<'a> Styled for TreeItem<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a> TreeView<TreeItem<'a>> for TreeItem<'a> {
    type ChildIter<'c>
        = std::slice::Iter<'c, TreeItem<'a>>
    where
        TreeItem<'a>: 'c;

    fn iter_children(&self) -> Self::ChildIter<'_> {
        self.children.iter()
    }
}

// MARK: Tree

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Tree<'a> {
    pub(crate) block: Option<Block<'a>>,
    pub(crate) children: Vec<TreeItem<'a>>,
    pub(crate) style: Style,
    pub(crate) indent_symbol: Option<&'a str>,
    pub(crate) highlight_symbol: Option<&'a str>,
    pub(crate) highlight_style: Style,
    pub(crate) repeat_highlight_symbol: bool,
    pub(crate) highlight_spacing: HighlightSpacing,
    pub(crate) scroll_padding: usize,
}

impl<'a> Tree<'a> {
    pub fn new<T>(children: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<TreeItem<'a>>,
    {
        Self {
            block: None,
            children: children.into_iter().map(Into::into).collect(),
            style: Style::default(),
            ..Self::default()
        }
    }

    pub fn new_empty() -> Self {
        Self::new(Vec::<TreeItem<'a>>::new())
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn children<C: Into<Vec<TreeItem<'a>>>>(mut self, children: C) -> Self {
        self.children = children.into();
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn child<C: Into<TreeItem<'a>>>(mut self, child: C) -> Self {
        self.children.push(child.into());
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn indent_symbol(mut self, indent_symbol: &'a str) -> Self {
        self.indent_symbol = Some(indent_symbol);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn highlight_symbol(mut self, highlight_symbol: &'a str) -> Self {
        self.highlight_symbol = Some(highlight_symbol);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.highlight_style = style.into();
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn repeat_highlight_symbol(mut self, repeat: bool) -> Self {
        self.repeat_highlight_symbol = repeat;
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn highlight_spacing(mut self, value: HighlightSpacing) -> Self {
        self.highlight_spacing = value;
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn scroll_padding(mut self, padding: usize) -> Self {
        self.scroll_padding = padding;
        self
    }
}

impl<'a> Styled for Tree<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a> TreeView<TreeItem<'a>> for Tree<'a> {
    type ChildIter<'c>
        = std::slice::Iter<'c, TreeItem<'a>>
    where
        TreeItem<'a>: 'c;

    fn iter_children(&self) -> Self::ChildIter<'_> {
        self.children.iter()
    }
}

// MARK: State

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct TreeState {
    pub(crate) offset: usize,
    pub(crate) selected: Option<TreeIndex>,
}

impl TreeState {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_selected<S: Into<TreeIndex>>(mut self, selected: S) -> Self {
        self.selected = Some(selected.into());
        self
    }

    pub fn select(&mut self, selected: Option<TreeIndex>) {
        self.selected = selected
    }

    pub const fn selected(&self) -> &Option<TreeIndex> {
        &self.selected
    }

    pub fn selected_mut(&mut self) -> &mut Option<TreeIndex> {
        &mut self.selected
    }
}

impl<'a> Tree<'a> {
    pub fn select_up(&self, selected: &mut Option<TreeIndex>) {
        *selected = match selected {
            None => self.find_first_child().map(|r| TreeIndex::new_at(r.0)),
            Some(index) => match self.find_previous_relative_of(index) {
                Some((next, _)) => Some(next),
                None => return,
            },
        };
    }

    pub fn select_up_state(&self, state: &mut TreeState) {
        self.select_up(&mut state.selected);
    }

    pub fn select_down(&self, selected: &mut Option<TreeIndex>) {
        *selected = match selected {
            None => self.find_last_descendant().map(|r| r.0),
            Some(index) => match self.find_next_relative_of(index) {
                Some((next, _)) => Some(next),
                None => return,
            },
        };
    }

    pub fn select_down_state(&self, state: &mut TreeState) {
        self.select_down(&mut state.selected);
    }

    pub fn select_parent(&self, selected: &mut Option<TreeIndex>) {
        *selected = match selected {
            None => self.find_first_child().map(|r| TreeIndex::new_at(r.0)),
            Some(index) => match self.find_parent_of(index.clone()) {
                Some((next, _)) => Some(next),
                None => return,
            },
        }
    }

    pub fn select_parent_state(&self, state: &mut TreeState) {
        self.select_parent(&mut state.selected);
    }
}

// MARK: Rendering

impl Widget for Tree<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl WidgetRef for Tree<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut state = TreeState::default();
        StatefulWidgetRef::render_ref(self, area, buf, &mut state);
    }
}

impl StatefulWidget for Tree<'_> {
    type State = TreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidgetRef::render_ref(&self, area, buf, state);
    }
}

impl StatefulWidgetRef for Tree<'_> {
    type State = TreeState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);
        self.block.render(area, buf);
        let tree_area = self.block.inner_if_some(area);

        if tree_area.is_empty() {
            return;
        }

        if self.is_empty() {
            state.select(None);
            return;
        }

        // If the selected index is out of bounds, set it to the nearest item.
        state.selected = state
            .selected
            .as_ref()
            .and_then(|index| self.find_nearest_to(&index))
            .map(|(index, _)| index);
        let selected_offset = state
            .selected
            .as_ref()
            .and_then(|index| self.find_offset_of_index(&index))
            .map(|(offset, _)| offset);

        let tree_height = tree_area.height as usize;

        let (first_visible_index, last_visible_index) =
            self.get_items_bounds(&state.selected, state.offset, tree_height);

        // Important: this changes the state's offset to be the beginning of the now viewable items
        state.offset = first_visible_index;

        // Get our set highlighted symbol (if one was set)
        let highlight_symbol = self.highlight_symbol.unwrap_or("");
        let blank_symbol = " ".repeat(highlight_symbol.width());

        let mut current_height = 0;
        let selection_spacing = match self.highlight_spacing {
            HighlightSpacing::Always => true,
            HighlightSpacing::WhenSelected => state.selected.is_some(),
            HighlightSpacing::Never => false,
        }; // TODO: unprivate

        for (i, (depth, item)) in self
            .iter_descendants_with_depth()
            .enumerate()
            .skip(state.offset)
            .take(last_visible_index - first_visible_index)
        {
            let item_style = self.style.patch(item.style);

            // Draw indentation and offset further children.
            let mut indent = tree_area.left();
            let indent_symbol = self.indent_symbol.unwrap_or("  ");
            for _ in 0..depth {
                buf.set_stringn(
                    indent,
                    tree_area.top() + current_height,
                    indent_symbol,
                    tree_area.width as usize,
                    item_style,
                );
                indent += indent_symbol.width() as u16;
            }

            let (x, y) = {
                let pos = (indent, tree_area.top() + current_height);
                current_height += item.height() as u16;
                pos
            };

            let row_area = Rect {
                x,
                y,
                width: tree_area.width - indent,
                height: item.height() as u16,
            };

            buf.set_style(row_area, item_style);

            let is_selected = selected_offset.map_or(false, |s| s == i);

            let item_area = if selection_spacing {
                let highlight_symbol_width = self.highlight_symbol.unwrap_or("").width() as u16;
                Rect {
                    x: row_area.x + highlight_symbol_width,
                    width: row_area.width.saturating_sub(highlight_symbol_width),
                    ..row_area
                }
            } else {
                row_area
            };
            item.content.render_ref(item_area, buf);

            for j in 0..item.content.height() {
                // if the item is selected, we need to display the highlight symbol:
                // - either for the first line of the item only,
                // - or for each line of the item if the appropriate option is set
                let symbol = if is_selected && (j == 0 || self.repeat_highlight_symbol) {
                    highlight_symbol
                } else {
                    &blank_symbol
                };
                if selection_spacing {
                    buf.set_stringn(
                        x,
                        y + j as u16,
                        symbol,
                        tree_area.width as usize,
                        item_style,
                    );
                }
            }

            if is_selected {
                buf.set_style(row_area, self.highlight_style);
            }
        }
    }
}

impl<'a> Tree<'a> {
    /// Given an offset, calculate which items can fit in a given area
    fn get_items_bounds(
        &self,
        selected: &Option<TreeIndex>,
        offset: usize,
        max_height: usize,
    ) -> (usize, usize) {
        let offset = offset.min(self.len_descendants().saturating_sub(1));

        // Note: visible here implies visible in the given area
        let mut first_visible_index = offset;
        let mut last_visible_index = offset;

        // Current height of all items in the list to render, beginning at the offset
        let mut height_from_offset = 0;

        // Calculate the last visible index and total height of the items
        // that will fit in the available space
        for item in self.iter_descendants().skip(offset) {
            if height_from_offset + item.height() > max_height {
                break;
            }

            height_from_offset += item.height();
            last_visible_index += 1;
        }

        // Get the selected index and apply scroll_padding to it, but still honor the offset if
        // nothing is selected. This allows for the list to stay at a position after select()ing
        // None.
        let index_to_display = self
            .apply_scroll_padding_to_selected_index(
                selected,
                max_height,
                first_visible_index,
                last_visible_index,
            )
            .unwrap_or(offset);

        // Recall that last_visible_index is the index of what we
        // can render up to in the given space after the offset
        // If we have an item selected that is out of the viewable area (or
        // the offset is still set), we still need to show this item
        while index_to_display >= last_visible_index {
            height_from_offset = height_from_offset.saturating_add(
                self.get_descendant_infix(last_visible_index)
                    .unwrap()
                    .height(),
            );

            last_visible_index += 1;

            // Now we need to hide previous items since we didn't have space
            // for the selected/offset item
            while height_from_offset > max_height {
                height_from_offset = height_from_offset.saturating_sub(
                    self.get_descendant_infix(first_visible_index)
                        .unwrap()
                        .height(),
                );

                // Remove this item to view by starting at the next item index
                first_visible_index += 1;
            }
        }

        // Here we're doing something similar to what we just did above
        // If the selected item index is not in the viewable area, let's try to show the item
        while index_to_display < first_visible_index {
            first_visible_index -= 1;

            height_from_offset = height_from_offset.saturating_add(
                self.get_descendant_infix(first_visible_index)
                    .unwrap()
                    .height(),
            );

            // Don't show an item if it is beyond our viewable height
            while height_from_offset > max_height {
                last_visible_index -= 1;

                height_from_offset = height_from_offset.saturating_sub(
                    self.get_descendant_infix(last_visible_index)
                        .unwrap()
                        .height(),
                );
            }
        }

        (first_visible_index, last_visible_index)
    }

    /// Applies scroll padding to the selected index, reducing the padding value to keep the
    /// selected item on screen even with items of inconsistent sizes
    ///
    /// This function is sensitive to how the bounds checking function handles item height
    fn apply_scroll_padding_to_selected_index(
        &self,
        selected: &Option<TreeIndex>,
        max_height: usize,
        first_visible_index: usize,
        last_visible_index: usize,
    ) -> Option<usize> {
        let last_valid_index = self.len_descendants().saturating_sub(1);
        let selected = selected
            .as_ref()
            .and_then(|index| self.find_offset_of_index(index))?
            .0
            .min(last_valid_index);

        // The bellow loop handles situations where the list item sizes may not be consistent,
        // where the offset would have excluded some items that we want to include, or could
        // cause the offset value to be set to an inconsistent value each time we render.
        // The padding value will be reduced in case any of these issues would occur
        let mut scroll_padding = self.scroll_padding;
        while scroll_padding > 0 {
            let mut height_around_selected = 0;
            for index in selected.saturating_sub(scroll_padding)
                ..=selected
                    .saturating_add(scroll_padding)
                    .min(last_valid_index)
            {
                height_around_selected += self.get_descendant_infix(index).unwrap().height();
            }
            if height_around_selected <= max_height {
                break;
            }
            scroll_padding -= 1;
        }

        Some(
            if (selected + scroll_padding).min(last_valid_index) >= last_visible_index {
                selected + scroll_padding
            } else if selected.saturating_sub(scroll_padding) < first_visible_index {
                selected.saturating_sub(scroll_padding)
            } else {
                selected
            }
            .min(last_valid_index),
        )
    }
}
