use hac_core::collection::types::{BodyType, Info, Request, RequestKind, RequestMethod};
use hac_core::collection::Collection;
use hac_core::syntax::highlighter::Highlighter;

use hac_client::pages::collection_viewer::{collection_store::CollectionStore, CollectionViewer};
use hac_client::pages::{Eventful, Renderable};
use hac_client::utils::build_syntax_highlighted_lines;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use lazy_static::lazy_static;
use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use tree_sitter::Tree;

fn main() {
    divan::main();
}

fn create_sample_collection() -> Collection {
    Collection {
        info: Info {
            name: "sample collection".to_string(),
            description: None,
        },
        path: "any_path".into(),
        requests: Some(Arc::new(RwLock::new(vec![
            RequestKind::Single(Arc::new(RwLock::new(Request {
                id: "any id".to_string(),
                headers: None,
                name: "testing".to_string(),
                parent: None,
                auth_method: None,
                uri: "https://jsonplaceholder.typicode.com/users".to_string(),
                method: RequestMethod::Get,
                body: Some("[\r\n  {\r\n    \"id\": 1,\r\n    \"name\": \"Leanne Graham\",\r\n    \"username\": \"Bret\",\r\n    \"email\": \"Sincere@april.biz\",\r\n    \"address\": {\r\n      \"street\": \"Kulas Light\",\r\n      \"suite\": \"Apt. 556\",\r\n      \"city\": \"Gwenborough\",\r\n      \"zipcode\": \"92998-3874\",\r\n      \"geo\": {\r\n        \"lat\": \"-37.3159\",\r\n        \"lng\": \"81.1496\"\r\n      }\r\n    },\r\n    \"phone\": \"1-770-736-8031 x56442\",\r\n    \"website\": \"hildegard.org\",\r\n    \"company\": {\r\n      \"name\": \"Romaguera-Crona\",\r\n      \"catchPhrase\": \"Multi-layered client-server neural-net\",\r\n      \"bs\": \"harness real-time e-markets\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 2,\r\n    \"name\": \"Ervin Howell\",\r\n    \"username\": \"Antonette\",\r\n    \"email\": \"Shanna@melissa.tv\",\r\n    \"address\": {\r\n      \"street\": \"Victor Plains\",\r\n      \"suite\": \"Suite 879\",\r\n      \"city\": \"Wisokyburgh\",\r\n      \"zipcode\": \"90566-7771\",\r\n      \"geo\": {\r\n        \"lat\": \"-43.9509\",\r\n        \"lng\": \"-34.4618\"\r\n      }\r\n    },\r\n    \"phone\": \"010-692-6593 x09125\",\r\n    \"website\": \"anastasia.net\",\r\n    \"company\": {\r\n      \"name\": \"Deckow-Crist\",\r\n      \"catchPhrase\": \"Proactive didactic contingency\",\r\n      \"bs\": \"synergize scalable supply-chains\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 3,\r\n    \"name\": \"Clementine Bauch\",\r\n    \"username\": \"Samantha\",\r\n    \"email\": \"Nathan@yesenia.net\",\r\n    \"address\": {\r\n      \"street\": \"Douglas Extension\",\r\n      \"suite\": \"Suite 847\",\r\n      \"city\": \"McKenziehaven\",\r\n      \"zipcode\": \"59590-4157\",\r\n      \"geo\": {\r\n        \"lat\": \"-68.6102\",\r\n        \"lng\": \"-47.0653\"\r\n      }\r\n    },\r\n    \"phone\": \"1-463-123-4447\",\r\n    \"website\": \"ramiro.info\",\r\n    \"company\": {\r\n      \"name\": \"Romaguera-Jacobson\",\r\n      \"catchPhrase\": \"Face to face bifurcated interface\",\r\n      \"bs\": \"e-enable strategic applications\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 4,\r\n    \"name\": \"Patricia Lebsack\",\r\n    \"username\": \"Karianne\",\r\n    \"email\": \"Julianne.OConner@kory.org\",\r\n    \"address\": {\r\n      \"street\": \"Hoeger Mall\",\r\n      \"suite\": \"Apt. 692\",\r\n      \"city\": \"South Elvis\",\r\n      \"zipcode\": \"53919-4257\",\r\n      \"geo\": {\r\n        \"lat\": \"29.4572\",\r\n        \"lng\": \"-164.2990\"\r\n      }\r\n    },\r\n    \"phone\": \"493-170-9623 x156\",\r\n    \"website\": \"kale.biz\",\r\n    \"company\": {\r\n      \"name\": \"Robel-Corkery\",\r\n      \"catchPhrase\": \"Multi-tiered zero tolerance productivity\",\r\n      \"bs\": \"transition cutting-edge web services\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 5,\r\n    \"name\": \"Chelsey Dietrich\",\r\n    \"username\": \"Kamren\",\r\n    \"email\": \"Lucio_Hettinger@annie.ca\",\r\n    \"address\": {\r\n      \"street\": \"Skiles Walks\",\r\n      \"suite\": \"Suite 351\",\r\n      \"city\": \"Roscoeview\",\r\n      \"zipcode\": \"33263\",\r\n      \"geo\": {\r\n        \"lat\": \"-31.8129\",\r\n        \"lng\": \"62.5342\"\r\n      }\r\n    },\r\n    \"phone\": \"(254)954-1289\",\r\n    \"website\": \"demarco.info\",\r\n    \"company\": {\r\n      \"name\": \"Keebler LLC\",\r\n      \"catchPhrase\": \"User-centric fault-tolerant solution\",\r\n      \"bs\": \"revolutionize end-to-end systems\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 6,\r\n    \"name\": \"Mrs. Dennis Schulist\",\r\n    \"username\": \"Leopoldo_Corkery\",\r\n    \"email\": \"Karley_Dach@jasper.info\",\r\n    \"address\": {\r\n      \"street\": \"Norberto Crossing\",\r\n      \"suite\": \"Apt. 950\",\r\n      \"city\": \"South Christy\",\r\n      \"zipcode\": \"23505-1337\",\r\n      \"geo\": {\r\n        \"lat\": \"-71.4197\",\r\n        \"lng\": \"71.7478\"\r\n      }\r\n    },\r\n    \"phone\": \"1-477-935-8478 x6430\",\r\n    \"website\": \"ola.org\",\r\n    \"company\": {\r\n      \"name\": \"Considine-Lockman\",\r\n      \"catchPhrase\": \"Synchronised bottom-line interface\",\r\n      \"bs\": \"e-enable innovative applications\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 7,\r\n    \"name\": \"Kurtis Weissnat\",\r\n    \"username\": \"Elwyn.Skiles\",\r\n    \"email\": \"Telly.Hoeger@billy.biz\",\r\n    \"address\": {\r\n      \"street\": \"Rex Trail\",\r\n      \"suite\": \"Suite 280\",\r\n      \"city\": \"Howemouth\",\r\n      \"zipcode\": \"58804-1099\",\r\n      \"geo\": {\r\n        \"lat\": \"24.8918\",\r\n        \"lng\": \"21.8984\"\r\n      }\r\n    },\r\n    \"phone\": \"210.067.6132\",\r\n    \"website\": \"elvis.io\",\r\n    \"company\": {\r\n      \"name\": \"Johns Group\",\r\n      \"catchPhrase\": \"Configurable multimedia task-force\",\r\n      \"bs\": \"generate enterprise e-tailers\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 8,\r\n    \"name\": \"Nicholas Runolfsdottir V\",\r\n    \"username\": \"Maxime_Nienow\",\r\n    \"email\": \"Sherwood@rosamond.me\",\r\n    \"address\": {\r\n      \"street\": \"Ellsworth Summit\",\r\n      \"suite\": \"Suite 729\",\r\n      \"city\": \"Aliyaview\",\r\n      \"zipcode\": \"45169\",\r\n      \"geo\": {\r\n        \"lat\": \"-14.3990\",\r\n        \"lng\": \"-120.7677\"\r\n      }\r\n    },\r\n    \"phone\": \"586.493.6943 x140\",\r\n    \"website\": \"jacynthe.com\",\r\n    \"company\": {\r\n      \"name\": \"Abernathy Group\",\r\n      \"catchPhrase\": \"Implemented secondary concept\",\r\n      \"bs\": \"e-enable extensible e-tailers\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 9,\r\n    \"name\": \"Glenna Reichert\",\r\n    \"username\": \"Delphine\",\r\n    \"email\": \"Chaim_McDermott@dana.io\",\r\n    \"address\": {\r\n      \"street\": \"Dayna Park\",\r\n      \"suite\": \"Suite 449\",\r\n      \"city\": \"Bartholomebury\",\r\n      \"zipcode\": \"76495-3109\",\r\n      \"geo\": {\r\n        \"lat\": \"24.6463\",\r\n        \"lng\": \"-168.8889\"\r\n      }\r\n    },\r\n    \"phone\": \"(775)976-6794 x41206\",\r\n    \"website\": \"conrad.com\",\r\n    \"company\": {\r\n      \"name\": \"Yost and Sons\",\r\n      \"catchPhrase\": \"Switchable contextually-based project\",\r\n      \"bs\": \"aggregate real-time technologies\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 10,\r\n    \"name\": \"Clementina DuBuque\",\r\n    \"username\": \"Moriah.Stanton\",\r\n    \"email\": \"Rey.Padberg@karina.biz\",\r\n    \"address\": {\r\n      \"street\": \"Kattie Turnpike\",\r\n      \"suite\": \"Suite 198\",\r\n      \"city\": \"Lebsackbury\",\r\n      \"zipcode\": \"31428-2261\",\r\n      \"geo\": {\r\n        \"lat\": \"-38.2386\",\r\n        \"lng\": \"57.2232\"\r\n      }\r\n    },\r\n    \"phone\": \"024-648-3804\",\r\n    \"website\": \"ambrose.net\",\r\n    \"company\": {\r\n      \"name\": \"Hoeger LLC\",\r\n      \"catchPhrase\": \"Centralized empowering task-force\",\r\n      \"bs\": \"target end-to-end models\"\r\n    }\r\n  }\r\n]".to_string()),
                body_type: Some(BodyType::Json),
            }))),
            RequestKind::Single(Arc::new(RwLock::new(Request {
                id: "any_other_id".to_string(),
                name: "testing".to_string(),
                auth_method: None,
                uri: "https://jsonplaceholder.typicode.com/users".to_string(),
                method: RequestMethod::Get,
                parent: None,
                headers: None,
                body: Some("[\r\n  {\r\n    \"id\": 1,\r\n    \"name\": \"Leanne Graham\",\r\n    \"username\": \"Bret\",\r\n    \"email\": \"Sincere@april.biz\",\r\n    \"address\": {\r\n      \"street\": \"Kulas Light\",\r\n      \"suite\": \"Apt. 556\",\r\n      \"city\": \"Gwenborough\",\r\n      \"zipcode\": \"92998-3874\",\r\n      \"geo\": {\r\n        \"lat\": \"-37.3159\",\r\n        \"lng\": \"81.1496\"\r\n      }\r\n    },\r\n    \"phone\": \"1-770-736-8031 x56442\",\r\n    \"website\": \"hildegard.org\",\r\n    \"company\": {\r\n      \"name\": \"Romaguera-Crona\",\r\n      \"catchPhrase\": \"Multi-layered client-server neural-net\",\r\n      \"bs\": \"harness real-time e-markets\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 2,\r\n    \"name\": \"Ervin Howell\",\r\n    \"username\": \"Antonette\",\r\n    \"email\": \"Shanna@melissa.tv\",\r\n    \"address\": {\r\n      \"street\": \"Victor Plains\",\r\n      \"suite\": \"Suite 879\",\r\n      \"city\": \"Wisokyburgh\",\r\n      \"zipcode\": \"90566-7771\",\r\n      \"geo\": {\r\n        \"lat\": \"-43.9509\",\r\n        \"lng\": \"-34.4618\"\r\n      }\r\n    },\r\n    \"phone\": \"010-692-6593 x09125\",\r\n    \"website\": \"anastasia.net\",\r\n    \"company\": {\r\n      \"name\": \"Deckow-Crist\",\r\n      \"catchPhrase\": \"Proactive didactic contingency\",\r\n      \"bs\": \"synergize scalable supply-chains\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 3,\r\n    \"name\": \"Clementine Bauch\",\r\n    \"username\": \"Samantha\",\r\n    \"email\": \"Nathan@yesenia.net\",\r\n    \"address\": {\r\n      \"street\": \"Douglas Extension\",\r\n      \"suite\": \"Suite 847\",\r\n      \"city\": \"McKenziehaven\",\r\n      \"zipcode\": \"59590-4157\",\r\n      \"geo\": {\r\n        \"lat\": \"-68.6102\",\r\n        \"lng\": \"-47.0653\"\r\n      }\r\n    },\r\n    \"phone\": \"1-463-123-4447\",\r\n    \"website\": \"ramiro.info\",\r\n    \"company\": {\r\n      \"name\": \"Romaguera-Jacobson\",\r\n      \"catchPhrase\": \"Face to face bifurcated interface\",\r\n      \"bs\": \"e-enable strategic applications\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 4,\r\n    \"name\": \"Patricia Lebsack\",\r\n    \"username\": \"Karianne\",\r\n    \"email\": \"Julianne.OConner@kory.org\",\r\n    \"address\": {\r\n      \"street\": \"Hoeger Mall\",\r\n      \"suite\": \"Apt. 692\",\r\n      \"city\": \"South Elvis\",\r\n      \"zipcode\": \"53919-4257\",\r\n      \"geo\": {\r\n        \"lat\": \"29.4572\",\r\n        \"lng\": \"-164.2990\"\r\n      }\r\n    },\r\n    \"phone\": \"493-170-9623 x156\",\r\n    \"website\": \"kale.biz\",\r\n    \"company\": {\r\n      \"name\": \"Robel-Corkery\",\r\n      \"catchPhrase\": \"Multi-tiered zero tolerance productivity\",\r\n      \"bs\": \"transition cutting-edge web services\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 5,\r\n    \"name\": \"Chelsey Dietrich\",\r\n    \"username\": \"Kamren\",\r\n    \"email\": \"Lucio_Hettinger@annie.ca\",\r\n    \"address\": {\r\n      \"street\": \"Skiles Walks\",\r\n      \"suite\": \"Suite 351\",\r\n      \"city\": \"Roscoeview\",\r\n      \"zipcode\": \"33263\",\r\n      \"geo\": {\r\n        \"lat\": \"-31.8129\",\r\n        \"lng\": \"62.5342\"\r\n      }\r\n    },\r\n    \"phone\": \"(254)954-1289\",\r\n    \"website\": \"demarco.info\",\r\n    \"company\": {\r\n      \"name\": \"Keebler LLC\",\r\n      \"catchPhrase\": \"User-centric fault-tolerant solution\",\r\n      \"bs\": \"revolutionize end-to-end systems\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 6,\r\n    \"name\": \"Mrs. Dennis Schulist\",\r\n    \"username\": \"Leopoldo_Corkery\",\r\n    \"email\": \"Karley_Dach@jasper.info\",\r\n    \"address\": {\r\n      \"street\": \"Norberto Crossing\",\r\n      \"suite\": \"Apt. 950\",\r\n      \"city\": \"South Christy\",\r\n      \"zipcode\": \"23505-1337\",\r\n      \"geo\": {\r\n        \"lat\": \"-71.4197\",\r\n        \"lng\": \"71.7478\"\r\n      }\r\n    },\r\n    \"phone\": \"1-477-935-8478 x6430\",\r\n    \"website\": \"ola.org\",\r\n    \"company\": {\r\n      \"name\": \"Considine-Lockman\",\r\n      \"catchPhrase\": \"Synchronised bottom-line interface\",\r\n      \"bs\": \"e-enable innovative applications\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 7,\r\n    \"name\": \"Kurtis Weissnat\",\r\n    \"username\": \"Elwyn.Skiles\",\r\n    \"email\": \"Telly.Hoeger@billy.biz\",\r\n    \"address\": {\r\n      \"street\": \"Rex Trail\",\r\n      \"suite\": \"Suite 280\",\r\n      \"city\": \"Howemouth\",\r\n      \"zipcode\": \"58804-1099\",\r\n      \"geo\": {\r\n        \"lat\": \"24.8918\",\r\n        \"lng\": \"21.8984\"\r\n      }\r\n    },\r\n    \"phone\": \"210.067.6132\",\r\n    \"website\": \"elvis.io\",\r\n    \"company\": {\r\n      \"name\": \"Johns Group\",\r\n      \"catchPhrase\": \"Configurable multimedia task-force\",\r\n      \"bs\": \"generate enterprise e-tailers\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 8,\r\n    \"name\": \"Nicholas Runolfsdottir V\",\r\n    \"username\": \"Maxime_Nienow\",\r\n    \"email\": \"Sherwood@rosamond.me\",\r\n    \"address\": {\r\n      \"street\": \"Ellsworth Summit\",\r\n      \"suite\": \"Suite 729\",\r\n      \"city\": \"Aliyaview\",\r\n      \"zipcode\": \"45169\",\r\n      \"geo\": {\r\n        \"lat\": \"-14.3990\",\r\n        \"lng\": \"-120.7677\"\r\n      }\r\n    },\r\n    \"phone\": \"586.493.6943 x140\",\r\n    \"website\": \"jacynthe.com\",\r\n    \"company\": {\r\n      \"name\": \"Abernathy Group\",\r\n      \"catchPhrase\": \"Implemented secondary concept\",\r\n      \"bs\": \"e-enable extensible e-tailers\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 9,\r\n    \"name\": \"Glenna Reichert\",\r\n    \"username\": \"Delphine\",\r\n    \"email\": \"Chaim_McDermott@dana.io\",\r\n    \"address\": {\r\n      \"street\": \"Dayna Park\",\r\n      \"suite\": \"Suite 449\",\r\n      \"city\": \"Bartholomebury\",\r\n      \"zipcode\": \"76495-3109\",\r\n      \"geo\": {\r\n        \"lat\": \"24.6463\",\r\n        \"lng\": \"-168.8889\"\r\n      }\r\n    },\r\n    \"phone\": \"(775)976-6794 x41206\",\r\n    \"website\": \"conrad.com\",\r\n    \"company\": {\r\n      \"name\": \"Yost and Sons\",\r\n      \"catchPhrase\": \"Switchable contextually-based project\",\r\n      \"bs\": \"aggregate real-time technologies\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 10,\r\n    \"name\": \"Clementina DuBuque\",\r\n    \"username\": \"Moriah.Stanton\",\r\n    \"email\": \"Rey.Padberg@karina.biz\",\r\n    \"address\": {\r\n      \"street\": \"Kattie Turnpike\",\r\n      \"suite\": \"Suite 198\",\r\n      \"city\": \"Lebsackbury\",\r\n      \"zipcode\": \"31428-2261\",\r\n      \"geo\": {\r\n        \"lat\": \"-38.2386\",\r\n        \"lng\": \"57.2232\"\r\n      }\r\n    },\r\n    \"phone\": \"024-648-3804\",\r\n    \"website\": \"ambrose.net\",\r\n    \"company\": {\r\n      \"name\": \"Hoeger LLC\",\r\n      \"catchPhrase\": \"Centralized empowering task-force\",\r\n      \"bs\": \"target end-to-end models\"\r\n    }\r\n  }\r\n]".to_string()),
                body_type: Some(BodyType::Json),
            }))),
        ])))
    }
}

fn feed_keys(widget: &mut CollectionViewer, key_codes: Vec<KeyCode>) {
    key_codes.into_iter().for_each(|code| {
        widget
            .handle_key_event(KeyEvent {
                code,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            .unwrap();
    });
}

#[divan::bench]
fn handling_key_events() {
    let colors = hac_colors::Colors::default();
    let collection = create_sample_collection();
    let size = Rect::new(0, 0, 80, 24);
    let config = hac_config::load_config();
    let mut store = CollectionStore::default();
    store.set_state(collection);
    let mut api_explorer =
        CollectionViewer::new(size, Rc::new(RefCell::new(store)), &colors, &config, false);
    let mut terminal = Terminal::new(TestBackend::new(size.width, size.height)).unwrap();
    let mut frame = terminal.get_frame();

    api_explorer.draw(&mut frame, size).unwrap();
    feed_keys(
        &mut api_explorer,
        vec![
            KeyCode::Char('j'),
            KeyCode::Char('j'),
            KeyCode::Char('j'),
            KeyCode::Char('j'),
            KeyCode::Char('j'),
            KeyCode::Char('j'),
            KeyCode::Char('j'),
        ],
    );
    api_explorer.draw(&mut frame, size).unwrap();
}

#[divan::bench]
fn creating_with_highlight() {
    let colors = hac_colors::Colors::default();
    let collection = create_sample_collection();
    let size = Rect::new(0, 0, 80, 24);
    let config = hac_config::load_config();
    let mut store = CollectionStore::default();
    store.set_state(collection);
    let mut api_explorer =
        CollectionViewer::new(size, Rc::new(RefCell::new(store)), &colors, &config, false);
    let mut terminal = Terminal::new(TestBackend::new(size.width, size.height)).unwrap();
    let _frame = terminal.get_frame();

    // we are simulating changing the active request 12 times, which will rebuild the highlight
    // tree.
    feed_keys(
        &mut api_explorer,
        vec![
            KeyCode::Tab,
            KeyCode::Tab,
            KeyCode::Tab,
            KeyCode::Char('j'),
            KeyCode::Enter,
            KeyCode::Char('k'),
            KeyCode::Enter,
            KeyCode::Char('j'),
            KeyCode::Enter,
            KeyCode::Char('k'),
            KeyCode::Enter,
            KeyCode::Char('j'),
            KeyCode::Enter,
            KeyCode::Char('k'),
            KeyCode::Enter,
            KeyCode::Char('j'),
            KeyCode::Enter,
            KeyCode::Char('k'),
            KeyCode::Enter,
            KeyCode::Char('j'),
            KeyCode::Enter,
            KeyCode::Char('k'),
            KeyCode::Enter,
            KeyCode::Char('j'),
            KeyCode::Enter,
            KeyCode::Char('k'),
            KeyCode::Enter,
        ],
    );
}

lazy_static! {
    static ref BODY: &'static str = "[\r\n  {\r\n    \"id\": 1,\r\n    \"name\": \"Leanne Graham\",\r\n    \"username\": \"Bret\",\r\n    \"email\": \"Sincere@april.biz\",\r\n    \"address\": {\r\n      \"street\": \"Kulas Light\",\r\n      \"suite\": \"Apt. 556\",\r\n      \"city\": \"Gwenborough\",\r\n      \"zipcode\": \"92998-3874\",\r\n      \"geo\": {\r\n        \"lat\": \"-37.3159\",\r\n        \"lng\": \"81.1496\"\r\n      }\r\n    },\r\n    \"phone\": \"1-770-736-8031 x56442\",\r\n    \"website\": \"hildegard.org\",\r\n    \"company\": {\r\n      \"name\": \"Romaguera-Crona\",\r\n      \"catchPhrase\": \"Multi-layered client-server neural-net\",\r\n      \"bs\": \"harness real-time e-markets\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 2,\r\n    \"name\": \"Ervin Howell\",\r\n    \"username\": \"Antonette\",\r\n    \"email\": \"Shanna@melissa.tv\",\r\n    \"address\": {\r\n      \"street\": \"Victor Plains\",\r\n      \"suite\": \"Suite 879\",\r\n      \"city\": \"Wisokyburgh\",\r\n      \"zipcode\": \"90566-7771\",\r\n      \"geo\": {\r\n        \"lat\": \"-43.9509\",\r\n        \"lng\": \"-34.4618\"\r\n      }\r\n    },\r\n    \"phone\": \"010-692-6593 x09125\",\r\n    \"website\": \"anastasia.net\",\r\n    \"company\": {\r\n      \"name\": \"Deckow-Crist\",\r\n      \"catchPhrase\": \"Proactive didactic contingency\",\r\n      \"bs\": \"synergize scalable supply-chains\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 3,\r\n    \"name\": \"Clementine Bauch\",\r\n    \"username\": \"Samantha\",\r\n    \"email\": \"Nathan@yesenia.net\",\r\n    \"address\": {\r\n      \"street\": \"Douglas Extension\",\r\n      \"suite\": \"Suite 847\",\r\n      \"city\": \"McKenziehaven\",\r\n      \"zipcode\": \"59590-4157\",\r\n      \"geo\": {\r\n        \"lat\": \"-68.6102\",\r\n        \"lng\": \"-47.0653\"\r\n      }\r\n    },\r\n    \"phone\": \"1-463-123-4447\",\r\n    \"website\": \"ramiro.info\",\r\n    \"company\": {\r\n      \"name\": \"Romaguera-Jacobson\",\r\n      \"catchPhrase\": \"Face to face bifurcated interface\",\r\n      \"bs\": \"e-enable strategic applications\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 4,\r\n    \"name\": \"Patricia Lebsack\",\r\n    \"username\": \"Karianne\",\r\n    \"email\": \"Julianne.OConner@kory.org\",\r\n    \"address\": {\r\n      \"street\": \"Hoeger Mall\",\r\n      \"suite\": \"Apt. 692\",\r\n      \"city\": \"South Elvis\",\r\n      \"zipcode\": \"53919-4257\",\r\n      \"geo\": {\r\n        \"lat\": \"29.4572\",\r\n        \"lng\": \"-164.2990\"\r\n      }\r\n    },\r\n    \"phone\": \"493-170-9623 x156\",\r\n    \"website\": \"kale.biz\",\r\n    \"company\": {\r\n      \"name\": \"Robel-Corkery\",\r\n      \"catchPhrase\": \"Multi-tiered zero tolerance productivity\",\r\n      \"bs\": \"transition cutting-edge web services\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 5,\r\n    \"name\": \"Chelsey Dietrich\",\r\n    \"username\": \"Kamren\",\r\n    \"email\": \"Lucio_Hettinger@annie.ca\",\r\n    \"address\": {\r\n      \"street\": \"Skiles Walks\",\r\n      \"suite\": \"Suite 351\",\r\n      \"city\": \"Roscoeview\",\r\n      \"zipcode\": \"33263\",\r\n      \"geo\": {\r\n        \"lat\": \"-31.8129\",\r\n        \"lng\": \"62.5342\"\r\n      }\r\n    },\r\n    \"phone\": \"(254)954-1289\",\r\n    \"website\": \"demarco.info\",\r\n    \"company\": {\r\n      \"name\": \"Keebler LLC\",\r\n      \"catchPhrase\": \"User-centric fault-tolerant solution\",\r\n      \"bs\": \"revolutionize end-to-end systems\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 6,\r\n    \"name\": \"Mrs. Dennis Schulist\",\r\n    \"username\": \"Leopoldo_Corkery\",\r\n    \"email\": \"Karley_Dach@jasper.info\",\r\n    \"address\": {\r\n      \"street\": \"Norberto Crossing\",\r\n      \"suite\": \"Apt. 950\",\r\n      \"city\": \"South Christy\",\r\n      \"zipcode\": \"23505-1337\",\r\n      \"geo\": {\r\n        \"lat\": \"-71.4197\",\r\n        \"lng\": \"71.7478\"\r\n      }\r\n    },\r\n    \"phone\": \"1-477-935-8478 x6430\",\r\n    \"website\": \"ola.org\",\r\n    \"company\": {\r\n      \"name\": \"Considine-Lockman\",\r\n      \"catchPhrase\": \"Synchronised bottom-line interface\",\r\n      \"bs\": \"e-enable innovative applications\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 7,\r\n    \"name\": \"Kurtis Weissnat\",\r\n    \"username\": \"Elwyn.Skiles\",\r\n    \"email\": \"Telly.Hoeger@billy.biz\",\r\n    \"address\": {\r\n      \"street\": \"Rex Trail\",\r\n      \"suite\": \"Suite 280\",\r\n      \"city\": \"Howemouth\",\r\n      \"zipcode\": \"58804-1099\",\r\n      \"geo\": {\r\n        \"lat\": \"24.8918\",\r\n        \"lng\": \"21.8984\"\r\n      }\r\n    },\r\n    \"phone\": \"210.067.6132\",\r\n    \"website\": \"elvis.io\",\r\n    \"company\": {\r\n      \"name\": \"Johns Group\",\r\n      \"catchPhrase\": \"Configurable multimedia task-force\",\r\n      \"bs\": \"generate enterprise e-tailers\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 8,\r\n    \"name\": \"Nicholas Runolfsdottir V\",\r\n    \"username\": \"Maxime_Nienow\",\r\n    \"email\": \"Sherwood@rosamond.me\",\r\n    \"address\": {\r\n      \"street\": \"Ellsworth Summit\",\r\n      \"suite\": \"Suite 729\",\r\n      \"city\": \"Aliyaview\",\r\n      \"zipcode\": \"45169\",\r\n      \"geo\": {\r\n        \"lat\": \"-14.3990\",\r\n        \"lng\": \"-120.7677\"\r\n      }\r\n    },\r\n    \"phone\": \"586.493.6943 x140\",\r\n    \"website\": \"jacynthe.com\",\r\n    \"company\": {\r\n      \"name\": \"Abernathy Group\",\r\n      \"catchPhrase\": \"Implemented secondary concept\",\r\n      \"bs\": \"e-enable extensible e-tailers\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 9,\r\n    \"name\": \"Glenna Reichert\",\r\n    \"username\": \"Delphine\",\r\n    \"email\": \"Chaim_McDermott@dana.io\",\r\n    \"address\": {\r\n      \"street\": \"Dayna Park\",\r\n      \"suite\": \"Suite 449\",\r\n      \"city\": \"Bartholomebury\",\r\n      \"zipcode\": \"76495-3109\",\r\n      \"geo\": {\r\n        \"lat\": \"24.6463\",\r\n        \"lng\": \"-168.8889\"\r\n      }\r\n    },\r\n    \"phone\": \"(775)976-6794 x41206\",\r\n    \"website\": \"conrad.com\",\r\n    \"company\": {\r\n      \"name\": \"Yost and Sons\",\r\n      \"catchPhrase\": \"Switchable contextually-based project\",\r\n      \"bs\": \"aggregate real-time technologies\"\r\n    }\r\n  },\r\n  {\r\n    \"id\": 10,\r\n    \"name\": \"Clementina DuBuque\",\r\n    \"username\": \"Moriah.Stanton\",\r\n    \"email\": \"Rey.Padberg@karina.biz\",\r\n    \"address\": {\r\n      \"street\": \"Kattie Turnpike\",\r\n      \"suite\": \"Suite 198\",\r\n      \"city\": \"Lebsackbury\",\r\n      \"zipcode\": \"31428-2261\",\r\n      \"geo\": {\r\n        \"lat\": \"-38.2386\",\r\n        \"lng\": \"57.2232\"\r\n      }\r\n    },\r\n    \"phone\": \"024-648-3804\",\r\n    \"website\": \"ambrose.net\",\r\n    \"company\": {\r\n      \"name\": \"Hoeger LLC\",\r\n      \"catchPhrase\": \"Centralized empowering task-force\",\r\n      \"bs\": \"target end-to-end models\"\r\n    }\r\n  }\r\n]";
    static ref TREE: Option<Tree> = Highlighter::default().parse(&BODY);
}

#[divan::bench]
fn benchmarking_building_content() {
    let colors = hac_colors::Colors::default();
    build_syntax_highlighted_lines(&BODY, TREE.as_ref(), &colors);
}
