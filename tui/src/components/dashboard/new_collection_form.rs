use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum FormFocus {
    #[default]
    Name,
    Description,
}

struct FormLayout {
    name_input: Rect,
    desc_input: Rect,
    name_input_title: Rect,
    desc_input_title: Rect,
}

#[derive(Debug, Default)]
pub struct FormState {
    name: String,
    description: String,
    focused_field: FormFocus,
    is_focused: bool,
}

#[derive(Debug)]
pub struct NewCollectionForm<'a> {
    colors: &'a colors::Colors,
}

impl<'a> NewCollectionForm<'a> {
    pub fn new(colors: &'a colors::Colors) -> Self {
        NewCollectionForm { colors }
    }

    fn build_layout(&self, size: Rect) -> FormLayout {
        let size = Rect {
            x: size.x + 2,
            y: size.y + 2,
            width: size.width.saturating_sub(4),
            height: size.height.saturating_sub(4),
        };
        let [name_input, desc_input] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .areas(size);

        let [name_input_title, name_input] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(2)])
            .areas(name_input);

        let [desc_input_title, desc_input] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(2)])
            .areas(desc_input);

        FormLayout {
            name_input_title,
            name_input,
            desc_input_title,
            desc_input,
        }
    }

    fn get_field_border(&self, state: &FormState, field: &FormFocus) -> Style {
        if state.is_focused && state.focused_field.eq(field) {
            Style::default().fg(self.colors.normal.green.into())
        } else {
            Style::default().fg(self.colors.bright.black.into())
        }
    }
}

impl StatefulWidget for NewCollectionForm<'_> {
    type State = FormState;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = self.build_layout(size);

        let name_input_title = Paragraph::new(Line::from("Name".fg(self.colors.normal.white)));
        let name_input = Paragraph::new(Line::from("".fg(self.colors.normal.white))).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(self.get_field_border(state, &FormFocus::Name)),
        );

        let block_border = if state.is_focused {
            Style::default().fg(self.colors.normal.green.into())
        } else {
            Style::default().fg(self.colors.bright.black.into())
        };

        let full_block = Block::default()
            .borders(Borders::ALL)
            .border_style(block_border)
            .border_type(BorderType::Rounded);

        full_block.render(size, buf);
        name_input_title.render(layout.name_input_title, buf);
        name_input.render(layout.name_input, buf);
    }
}
