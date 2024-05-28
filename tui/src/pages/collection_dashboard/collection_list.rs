use std::{
    collections::VecDeque,
    ops::{Add, Div, Mul},
};

use hac::collection::Collection;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget,
    },
};

#[derive(Debug)]
pub struct CollectionListState {
    selected: Option<usize>,
    pub(super) items: Vec<Collection>,
    scroll: usize,
}

impl CollectionListState {
    pub fn new(items: Vec<Collection>) -> Self {
        CollectionListState {
            selected: None,
            items,
            scroll: 0,
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn set_items(&mut self, items: Vec<Collection>) {
        self.items = items;
    }
}

#[derive(Debug, Clone)]
pub struct CollectionList<'cl> {
    colors: &'cl colors::Colors,
    min_col_width: u16,
    row_height: u16,
}

impl<'a> CollectionList<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        CollectionList {
            colors,
            min_col_width: 30,
            row_height: 4,
        }
    }

    pub fn items_per_row(&self, size: &Rect) -> usize {
        (size.width.saturating_sub(1).div(self.min_col_width)).into()
    }

    pub fn total_rows(&self, size: &Rect) -> usize {
        (size.height.div(self.row_height)).into()
    }

    fn build_layout(&self, size: &Rect) -> VecDeque<Rect> {
        let total_rows = self.total_rows(size);
        let items_per_row = self.items_per_row(size);

        (0..total_rows)
            .flat_map(|row| {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .flex(Flex::SpaceAround)
                    .constraints((0..items_per_row).map(|_| Constraint::Min(self.min_col_width)))
                    .split(Rect::new(
                        size.x,
                        size.y + (self.row_height.mul(row as u16)),
                        size.width,
                        self.row_height,
                    ))
                    .to_vec()
            })
            .collect::<VecDeque<_>>()
    }

    fn build_card(
        &self,
        state: &CollectionListState,
        collection: &Collection,
        index: usize,
    ) -> Paragraph<'_> {
        let lines = vec![
            collection
                .info
                .name
                .clone()
                .fg(self.colors.normal.white)
                .into(),
            collection
                .info
                .description
                .clone()
                .unwrap_or_default()
                .fg(self.colors.bright.yellow)
                .into(),
        ];

        let border_color = if state
            .selected
            .is_some_and(|selected| selected.eq(&(index.add(state.scroll))))
        {
            self.colors.bright.magenta
        } else {
            self.colors.primary.hover
        };

        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        )
    }
}

impl StatefulWidget for CollectionList<'_> {
    type State = CollectionListState;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let list_size = Rect::new(size.x, size.y, size.width.saturating_sub(3), size.height);
        let scrollbar_size = Rect::new(size.width.saturating_sub(1), size.y, 1, size.height);
        let mut rects = self.build_layout(&list_size);

        let mut scrollbar_state =
            ScrollbarState::new(state.items.len().div(self.items_per_row(&size)))
                .position(state.scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(self.colors.normal.magenta))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let items_on_display = self
            .items_per_row(&list_size)
            .mul(self.total_rows(&list_size));
        if let Some(index) = state.selected {
            index
                .gt(&items_on_display.saturating_sub(1).add(state.scroll))
                .then(|| state.scroll = state.scroll.add(self.items_per_row(&list_size)));

            state.scroll.gt(&0).then(|| {
                index
                    .add(1)
                    .saturating_sub(state.scroll)
                    .eq(&0)
                    .then(|| state.scroll = state.scroll.saturating_sub(self.items_per_row(&size)));
            });
        };

        state
            .items
            .iter()
            .skip(state.scroll)
            .take(rects.len())
            .enumerate()
            .map(|(i, collection)| self.build_card(state, collection, i))
            .for_each(|card| card.render(rects.pop_front().unwrap(), buf));

        scrollbar.render(scrollbar_size, buf, &mut scrollbar_state);
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Sub;

    use super::*;
    use hac::collection::types::*;
    use ratatui::{backend::TestBackend, buffer::Cell, Terminal};

    fn sample_collection() -> Collection {
        Collection {
            info: Info {
                name: String::from("any_name"),
                description: None,
            },
            path: "any_path".into(),
            requests: None,
        }
    }

    #[test]
    fn test_build_layout() {
        let colors = colors::Colors::default();
        let collection_list = CollectionList::new(&colors);
        let size = Rect::new(0, 0, 31, 10);

        let layout = collection_list.build_layout(&size);

        assert!(!layout.is_empty());
        assert_eq!(layout.len(), 2);
    }

    #[test]
    fn test_items_per_row() {
        let colors = colors::Colors::default();
        let collection_list = CollectionList::new(&colors);
        let zero_items = Rect::new(0, 0, 30, 10);
        let one_item = Rect::new(0, 0, 31, 10);

        let amount = collection_list.items_per_row(&zero_items);
        assert_eq!(amount, 0);

        let amount = collection_list.items_per_row(&one_item);
        assert_eq!(amount, 1);
    }

    #[test]
    fn test_build_card() {
        let colors = colors::Colors::default();
        let collection_list = CollectionList::new(&colors);
        let collections = vec![Collection {
            info: Info {
                name: String::from("any_name"),
                description: None,
            },
            path: "any_path".into(),
            requests: None,
        }];
        let state = CollectionListState::new(collections.clone());

        let lines = vec![
            "any_name".fg(colors.normal.white).into(),
            "".fg(colors.bright.yellow).into(),
        ];
        let expected = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.primary.hover)),
        );

        let card = collection_list.build_card(&state, &collections[0], 0);

        assert_eq!(card, expected);
    }

    #[test]
    fn test_rendering() {
        let colors = colors::Colors::default();
        let collections = (0..100).map(|_| sample_collection()).collect::<Vec<_>>();

        let backend = TestBackend::new(80, 22);
        let mut terminal = Terminal::new(backend).unwrap();
        let size = terminal.size().unwrap();
        let mut frame = terminal.get_frame();

        let mut state = CollectionListState::new(collections.clone());
        let collection_list = CollectionList::new(&colors);

        for cell in &frame.buffer_mut().content {
            assert_eq!(cell, &Cell::default());
        }

        collection_list.render(size, frame.buffer_mut(), &mut state);

        for cell in frame
            .buffer_mut()
            .content
            .iter()
            .skip(size.width.sub(1).into())
            .step_by(size.width.into())
        {
            assert!(cell.symbol().ne(" "));
        }
    }
}
