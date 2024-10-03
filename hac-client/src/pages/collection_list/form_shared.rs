use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::text_object::cursor::Cursor;
use hac_core::text_object::{TextObject, Write};
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ascii::LOGO_ASCII;
use crate::components::input::Input;
use crate::{HacColors, MIN_HEIGHT, MIN_WIDTH};

#[derive(Debug, Clone, Copy)]
pub struct FormLayout {
    pub name_input: Rect,
    pub hint: Rect,
    pub logo: Rect,
}

#[derive(Debug)]
pub enum FormEvent {
    Confirm,
    Cancel,
}

pub fn build_form_layout(area: Rect) -> FormLayout {
    let [_, form, _] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(MIN_WIDTH), Constraint::Fill(1)])
        .flex(Flex::Center)
        .areas(area);

    let [_, form, _] = Layout::vertical([Constraint::Fill(1), Constraint::Length(MIN_HEIGHT), Constraint::Fill(1)])
        .flex(Flex::End)
        .areas(form);

    let form = form.inner(&Margin::new(2, 0));

    let [logo, _, name_input, _, hint] = Layout::vertical([
        Constraint::Length(LOGO_ASCII.len() as u16),
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .flex(Flex::Center)
    .areas(form);

    FormLayout { name_input, hint, logo }
}

pub fn handle_form_key_event(
    key_event: KeyEvent,
    text_object: &mut TextObject<Write>,
    cursor: &mut Cursor,
) -> anyhow::Result<Option<FormEvent>> {
    match key_event.code {
        KeyCode::Enter => return Ok(Some(FormEvent::Confirm)),
        KeyCode::Esc => return Ok(Some(FormEvent::Cancel)),
        KeyCode::Char('b') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => cursor.move_left(1),
        KeyCode::Left if matches!(key_event.modifiers, KeyModifiers::ALT) => {
            let (col, row) = text_object.find_char_before_whitespace(cursor);
            cursor.move_to(col, row);
        }
        KeyCode::Right if matches!(key_event.modifiers, KeyModifiers::ALT) => {
            let (col, row) = text_object.find_char_after_whitespace(cursor);
            cursor.move_to(col, row);
        }
        KeyCode::Left => cursor.move_left(1),
        KeyCode::Char('e') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
            cursor.move_to_line_end(text_object.line_len(0))
        }
        KeyCode::Down => cursor.move_to_line_end(text_object.line_len(0)),
        KeyCode::Char('a') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => cursor.move_to_line_start(),
        KeyCode::Up => cursor.move_to_line_start(),
        KeyCode::Char('f') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => cursor.move_right(1),
        KeyCode::Right => {
            cursor.move_right(1);
            cursor.maybe_snap_to_col(text_object.line_len(0));
        }
        KeyCode::Char('d') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
            text_object.erase_current_char(cursor)
        }
        KeyCode::Char('u') if matches!(key_event.modifiers, KeyModifiers::CONTROL) => {
            text_object.clear_line(cursor);
            cursor.move_to_line_start();
        }
        KeyCode::Backspace => {
            text_object.erase_previous_char(cursor);
            cursor.move_left(1);
        }
        KeyCode::Char(c) => {
            text_object.insert_char(c, cursor);
            cursor.move_right(1);
        }
        _ => {}
    }

    Ok(None)
}

pub fn draw_form_layout(layout: FormLayout, name: String, colors: &HacColors, frame: &mut Frame) {
    let label = String::from("Collection Name");
    let name_input = Input::new(Some(&name), Some(&label), colors.clone())
        .value_style(Style::default().fg(colors.normal.white))
        .label_style(Style::default().fg(colors.bright.black));

    let hint = vec![
        "Enter".fg(colors.bright.green).bold(),
        " - Confirm • ".fg(colors.bright.black),
        "Esc".fg(colors.bright.green).bold(),
        " - Cancel".fg(colors.bright.black),
    ];

    let logo = Paragraph::new(
        LOGO_ASCII
            .iter()
            .map(|line| Line::from(line.to_string()).fg(colors.bright.red).centered())
            .collect::<Vec<_>>(),
    );

    frame.render_widget(logo, layout.logo);
    frame.render_widget(name_input, layout.name_input);
    frame.render_widget(Line::from(hint), layout.hint);
}

pub fn set_form_cursor(layout: FormLayout, cursor: &Cursor, frame: &mut Frame) {
    frame.set_cursor(
        layout.name_input.x + 1 + cursor.col() as u16,
        layout.name_input.y + 2 + cursor.row() as u16,
    );
}
