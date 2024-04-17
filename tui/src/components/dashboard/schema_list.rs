use std::{collections::VecDeque, ops::Add};

use httpretty::schema::Schema;
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
pub struct SchemaListState {
    selected: Option<usize>,
    pub(super) items: Vec<Schema>,
    scroll: usize,
}

impl SchemaListState {
    pub fn new(items: Vec<Schema>) -> Self {
        SchemaListState {
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

    pub fn set_items(&mut self, items: Vec<Schema>) {
        self.items = items;
    }
}

#[derive(Debug, Clone)]
pub struct SchemaList<'a> {
    colors: &'a colors::Colors,
    min_col_width: u16,
    row_height: u16,
}

impl<'a> SchemaList<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        SchemaList {
            colors,
            min_col_width: 30,
            row_height: 4,
        }
    }

    pub fn items_per_row(&self, size: &Rect) -> usize {
        (size.width.saturating_sub(1) / self.min_col_width).into()
    }

    pub fn total_rows(&self, size: &Rect) -> usize {
        (size.height / self.row_height).into()
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
                        size.y + (self.row_height * row as u16),
                        size.width,
                        self.row_height,
                    ))
                    .to_vec()
            })
            .collect::<VecDeque<_>>()
    }

    fn build_card(&self, state: &SchemaListState, schema: &Schema, index: usize) -> Paragraph<'_> {
        let lines = vec![
            schema
                .info
                .name
                .clone()
                .fg(self.colors.normal.yellow)
                .into(),
            schema
                .info
                .description
                .clone()
                .unwrap_or_default()
                .fg(self.colors.normal.white)
                .into(),
        ];

        let border_color = if state
            .selected
            .is_some_and(|selected| selected.eq(&(index + state.scroll)))
        {
            self.colors.normal.green
        } else {
            self.colors.bright.black
        };

        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color.into())),
        )
    }
}

impl StatefulWidget for SchemaList<'_> {
    type State = SchemaListState;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let list_size = Rect::new(size.x, size.y, size.width.saturating_sub(1), size.height);
        let scrollbar_size = Rect::new(size.width.saturating_sub(1), size.y, 1, size.height);
        let mut rects = self.build_layout(&list_size);

        let mut scrollbar_state =
            ScrollbarState::new(state.items.len() / self.items_per_row(&size))
                .position(state.scroll);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(self.colors.normal.magenta.into()))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let items_on_display = self.items_per_row(&size) * self.total_rows(&size);
        if let Some(index) = state.selected {
            index
                .gt(&items_on_display.saturating_sub(1).add(state.scroll))
                .then(|| state.scroll += self.items_per_row(&size));

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
            .map(|(i, schema)| self.build_card(state, schema, i))
            .for_each(|card| card.render(rects.pop_front().unwrap(), buf));

        scrollbar.render(scrollbar_size, buf, &mut scrollbar_state);
    }
}
