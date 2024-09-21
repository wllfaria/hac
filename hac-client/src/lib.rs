use std::rc::Rc;
use std::sync::RwLock;

use hac_colors::Colors;
use hac_config::Config;

pub mod app;
mod ascii;
mod components;
pub mod event_pool;
pub mod pages;
mod router;
pub mod screen_manager;
pub mod utils;

pub type HacConfig = Rc<RwLock<Config>>;
pub type HacColors = Rc<Colors>;
