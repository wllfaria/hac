use std::rc::Rc;

use hac_store::collection::ReqMethod;
use hac_store::slab::Key;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ascii::LOGO_ASCII;
use crate::components::blending_list::BlendingList;
use crate::components::input::Input;
use crate::{HacColors, MIN_HEIGHT, MIN_WIDTH};

#[derive(Debug)]
pub enum FormStep {
    MainForm,
    ParentSelector,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum FieldFocus {
    #[default]
    Name,
    Methods,
    Parent,
}

impl FieldFocus {
    pub fn next(&mut self) {
        match self {
            FieldFocus::Name => *self = FieldFocus::Methods,
            FieldFocus::Methods => *self = FieldFocus::Parent,
            FieldFocus::Parent => *self = FieldFocus::Name,
        }
    }

    pub fn prev(&mut self) {
        match self {
            FieldFocus::Name => *self = FieldFocus::Parent,
            FieldFocus::Methods => *self = FieldFocus::Name,
            FieldFocus::Parent => *self = FieldFocus::Methods,
        }
    }
}

#[derive(Debug)]
pub struct ReqFormLayout {
    pub name: Rect,
    pub hint: Rect,
    pub logo: Rect,
    pub parent: Rect,
    pub methods: Rc<[Rect]>,
    pub parent_listing: Rect,
    pub parent_hint: Rect,
}

pub fn make_contextual_hint(focus: FieldFocus, colors: &HacColors) -> Vec<Span> {
    match focus {
        FieldFocus::Name => vec![
            "Enter".fg(colors.bright.green).bold(),
            " - Confirm • ".fg(colors.bright.black),
            "Esc".fg(colors.bright.green).bold(),
            " - Cancel • ".fg(colors.bright.black),
            "Tab".fg(colors.bright.green).bold(),
            " - Next • ".fg(colors.bright.black),
            "S-Tab".fg(colors.bright.green).bold(),
            " - Prev • ".fg(colors.bright.black),
            "Ctrl p".fg(colors.bright.green).bold(),
            " - Parent".fg(colors.bright.black),
        ],
        FieldFocus::Methods => vec![
            "Enter".fg(colors.bright.green).bold(),
            " - Confirm • ".fg(colors.bright.black),
            "Esc".fg(colors.bright.green).bold(),
            " - Cancel • ".fg(colors.bright.black),
            "Tab".fg(colors.bright.green).bold(),
            " - Next • ".fg(colors.bright.black),
            "S-Tab".fg(colors.bright.green).bold(),
            " - Prev • ".fg(colors.bright.black),
            "1-5".fg(colors.bright.green).bold(),
            " - Method".fg(colors.bright.black),
        ],
        FieldFocus::Parent => vec![
            "Enter".fg(colors.bright.green).bold(),
            " - Confirm • ".fg(colors.bright.black),
            "Esc".fg(colors.bright.green).bold(),
            " - Cancel • ".fg(colors.bright.black),
            "Ctrl p".fg(colors.bright.green).bold(),
            " - Parent • ".fg(colors.bright.black),
            "Backspace".fg(colors.bright.green).bold(),
            " - Remove parent".fg(colors.bright.black),
        ],
    }
}

pub fn draw_main_form(
    name: &str,
    method: ReqMethod,
    parent: Option<Key>,
    focus: FieldFocus,
    layout: &ReqFormLayout,
    colors: &HacColors,
    frame: &mut Frame,
) {
    let border_style = match focus == FieldFocus::Name {
        true => Style::new().fg(colors.bright.red),
        false => Style::new().fg(colors.bright.black),
    };
    let label = String::from("Request Name");
    let name_input = Input::new(Some(name), Some(&label), colors.clone())
        .border_style(border_style)
        .value_style(Style::default().fg(colors.normal.white))
        .label_style(Style::default().fg(colors.bright.black));

    let logo = Paragraph::new(
        LOGO_ASCII
            .iter()
            .map(|line| Line::from(line.to_string()).fg(colors.bright.red).centered())
            .collect::<Vec<_>>(),
    );

    for (idx, m) in ReqMethod::iter().enumerate() {
        let selected = m == method;
        let number_color = match selected {
            true => colors.bright.blue,
            false => colors.bright.black,
        };
        let area = layout.methods[idx];
        let method = m.to_string();
        let remaining_width = area.width as usize - 3 - method.len();
        let left_pad = remaining_width / 2;

        let parts = vec![
            (idx + 1).to_string().fg(number_color),
            " ".repeat(left_pad).into(),
            method.fg(number_color),
        ];

        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(colors.bright.black));
        if let FieldFocus::Methods = focus {
            block = block.border_style(Style::new().fg(colors.bright.red));
        }
        if selected {
            block = block.border_style(Style::new().fg(colors.bright.blue));
        }

        frame.render_widget(Paragraph::new(Line::from(parts)).block(block), area);
    }

    let hint = make_contextual_hint(focus, colors);
    let mut parent_name = "No Parent".to_string();
    if let Some(parent) = parent {
        hac_store::collection::get_folder(parent, |folder, _| parent_name.clone_from(&folder.name));
    };

    let parent_color = if focus == FieldFocus::Parent { colors.bright.red } else { colors.bright.black };

    let parent =
        Paragraph::new(Line::from(parent_name).centered()).block(Block::new().borders(Borders::ALL).fg(parent_color));

    frame.render_widget(name_input, layout.name);
    frame.render_widget(logo, layout.logo);
    frame.render_widget(Clear, layout.parent);
    frame.render_widget(parent, layout.parent);
    frame.render_widget(Line::from(hint).centered(), layout.hint);

    if let FieldFocus::Name = focus {
        frame.set_cursor(layout.name.x + 1 + name.chars().count() as u16, layout.name.y + 2);
    }
}

pub fn draw_parent_selector(
    parent_listing: &mut BlendingList,
    layout: &ReqFormLayout,
    colors: &HacColors,
    frame: &mut Frame,
) {
    let logo = Paragraph::new(
        LOGO_ASCII
            .iter()
            .map(|line| Line::from(line.to_string()).fg(colors.bright.red).centered())
            .collect::<Vec<_>>(),
    );

    let mut folders = vec![];
    hac_store::collection::folders(|folder| folders.push(folder.name.clone()));
    parent_listing.draw_with(frame, folders.iter(), |name| name, layout.parent_listing);

    let hint = vec![
        "Enter".fg(colors.bright.green).bold(),
        " - Confirm • ".fg(colors.bright.black),
        "Esc".fg(colors.bright.green).bold(),
        " - Cancel • ".fg(colors.bright.black),
        "j/k ↑/↓".fg(colors.normal.green),
        " - Choose".fg(colors.bright.black),
    ];

    frame.render_widget(Line::from(hint).centered(), layout.parent_hint);
    frame.render_widget(logo, layout.logo);
}

pub fn build_req_form_layout(area: Rect) -> ReqFormLayout {
    let [_, form, _] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(MIN_WIDTH), Constraint::Fill(1)])
        .flex(Flex::Center)
        .areas(area);

    let [_, form, _] = Layout::vertical([Constraint::Fill(1), Constraint::Length(MIN_HEIGHT), Constraint::Fill(1)])
        .flex(Flex::End)
        .areas(form);

    let form = form.inner(&Margin::new(2, 0));

    let [logo, _, name, _, methods, _, parent, hint] = Layout::vertical([
        Constraint::Length(LOGO_ASCII.len() as u16),
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .flex(Flex::Center)
    .areas(form);

    let [_, _, parent_listing, _, parent_hint] = Layout::vertical([
        Constraint::Length(LOGO_ASCII.len() as u16),
        Constraint::Length(1),
        Constraint::Length(13),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .flex(Flex::Center)
    .areas(form);

    let methods = Layout::horizontal((0..ReqMethod::size()).map(|_| Constraint::Fill(1))).split(methods);

    ReqFormLayout {
        name,
        hint,
        logo,
        methods,
        parent,
        parent_listing,
        parent_hint,
    }
}
