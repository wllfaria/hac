use httpretty::schema::Schema;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};

pub struct SchemaListState<'a> {
    selected: Option<usize>,
    items: &'a [Schema],
}

impl<'a> SchemaListState<'a> {
    pub fn new(items: &'a [Schema]) -> Self {
        SchemaListState {
            selected: None,
            items,
        }
    }
}

#[derive(Default)]
pub struct SchemaList<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> SchemaList<'a> {
    fn build_layout(&self, size: &Rect) -> (u16, Vec<Rect>) {
        let col_width = 40;
        let items_no_spacing = col_width / size.width;
        let items_per_row = items_no_spacing * size.width;

        let row_height = 4;
        let total_rows = size.height / row_height;
        let items_per_row = size.width / col_width;
        let total_items = total_rows * items_per_row;

        let items = (0..total_rows)
            .flat_map(|row| {
                let l = Layout::default()
                    .direction(Direction::Horizontal)
                    .areas(*size);
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Fill(1)])
                    .flex(Flex::SpaceBetween)
                    .constraints((0..items_per_row).map(j))
                    .split(*row)
            })
            .collect::<Vec<_>>();

        (total_items, items)
    }
}

impl<'a> StatefulWidget for SchemaList<'a> {
    type State = SchemaListState<'a>;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (total_items, items) = self.build_layout(&size);

        items.into_iter().for_each(|item| {
            Paragraph::new(Line::from("lol"))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .render(item, buf)
        });
    }
}
