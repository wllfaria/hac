use std::cell::RefCell;
use std::ops::{Add, Div, Sub};
use std::rc::Rc;

use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::ascii::LOGO_ASCII;
use crate::pages::collection_viewer::collection_store::CollectionStore;
use crate::pages::input::Input;
use crate::pages::overlay::make_overlay_old;
use crate::pages::Renderable;

/// set of events `DirectoryForm` can send the parent to
/// handle
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DirectoryFormEvent {
    /// when user confirms the directory creation we should notify
    /// the parent to properly handle the event
    Confirm,
    /// when the user cancels the creation, we should also notify
    /// the parent to properly clean up things
    Cancel,
}

#[derive(Debug)]
pub struct DirectoryFormCreate;

#[derive(Debug)]
pub struct DirectoryFormEdit;

#[derive(Debug)]
pub struct DirectoryForm<'df, State = DirectoryFormCreate> {
    pub colors: &'df hac_colors::Colors,
    pub dir_name: String,
    pub collection_store: Rc<RefCell<CollectionStore>>,
    /// the id of the directory being edited, this is only used when editing a directory
    /// this is (dir_id, dir_name)
    pub directory: Option<(String, String)>,

    pub marker: std::marker::PhantomData<State>,
}

impl<'cdf, State> DirectoryForm<'cdf, State> {
    pub fn reset(&mut self) {
        self.dir_name.clear();
    }
}

impl<State> Renderable for DirectoryForm<'_, State> {
    fn draw(&mut self, frame: &mut ratatui::prelude::Frame, _: ratatui::prelude::Rect) -> anyhow::Result<()> {
        make_overlay_old(self.colors, self.colors.normal.black, 0.1, frame);

        let logo = LOGO_ASCII;
        let logo_size = logo.len() as u16;

        let size = frame.size();
        let size = Rect::new(
            size.width.div(2).sub(25),
            size.height.div(2).saturating_sub(logo_size.div(2)).saturating_sub(2),
            50,
            logo_size.add(4),
        );

        let logo = logo
            .iter()
            .map(|line| Line::from(line.to_string().fg(self.colors.normal.red)).centered())
            .collect::<Vec<_>>();

        let mut input = Input::new(self.colors, "Name".into());
        input.focus();

        let hint = Line::from("[Confirm: Enter] [Cancel: Esc]")
            .fg(self.colors.bright.black)
            .centered();

        let logo_size = Rect::new(size.x, size.y, size.width, logo_size);
        let input_size = Rect::new(size.x, logo_size.y.add(logo_size.height).add(1), size.width, 3);
        let hint_size = Rect::new(size.x, input_size.y.add(4), size.width, 1);

        frame.render_widget(Paragraph::new(logo), logo_size);
        frame.render_stateful_widget(input, input_size, &mut self.dir_name);
        frame.render_widget(hint, hint_size);

        frame.set_cursor(
            input_size.x.add(self.dir_name.chars().count() as u16).add(1),
            input_size.y.add(1),
        );

        Ok(())
    }
}
