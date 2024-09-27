pub mod app;
mod ascii;
mod components;
pub mod event_pool;
pub mod pages;
mod renderable;
mod router;
pub mod utils;

use std::cell::RefCell;
use std::rc::Rc;

use hac_colors::Colors;
use hac_config::Config;

pub static MIN_WIDTH: u16 = 80;
pub static MIN_HEIGHT: u16 = 30;

pub type HacConfig = Rc<RefCell<Config>>;
pub type HacColors = Rc<Colors>;
