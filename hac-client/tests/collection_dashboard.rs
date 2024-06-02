use hac_core::collection;

use hac_client::pages::{collection_dashboard::CollectionDashboard, Eventful, Renderable};

use std::fs::{create_dir, File};
use std::io::Write;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, layout::Rect, Frame, Terminal};
use tempfile::{tempdir, TempDir};

fn setup_temp_collections(amount: usize) -> (TempDir, String) {
    let tmp_data_dir = tempdir().expect("Failed to create temp data dir");

    let tmp_dir = tmp_data_dir.path().join("collections");
    create_dir(&tmp_dir).expect("Failed to create collections directory");

    for i in 0..amount {
        let file_path = tmp_dir.join(format!("test_collection_{}.json", i));
        let mut tmp_file = File::create(&file_path).expect("Failed to create file");

        write!(
            tmp_file,
            r#"{{"info": {{ "name": "test_collection_{}", "description": "test_description_{}" }}}}"#,
            i, i
        ).expect("Failed to write to file");

        tmp_file.flush().expect("Failed to flush file");
    }

    (tmp_data_dir, tmp_dir.to_string_lossy().to_string())
}

fn feed_keys(dashboard: &mut CollectionDashboard, events: &[KeyEvent]) {
    for event in events {
        _ = dashboard.handle_key_event(*event);
    }
}

fn get_rendered_from_buffer(frame: &mut Frame, size: Rect) -> Vec<String> {
    frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>()
}

#[test]
fn test_draw_empty_message() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let mut dashboard = CollectionDashboard::new(size, &colors, vec![], false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected = [
        "            █▖▐▌                ▝█  ▝█           ▟   ▀                          ",
        "            █▜▟▌▟▀▙     ▟▀▙ ▟▀▙  █   █  ▟▀▙ ▟▀▙ ▝█▀ ▝█  ▟▀▙ █▀▙ ▟▀▀             ",
        "            █ ▜▌█ █     █ ▄ █ █  █   █  █▀▀ █ ▄  █▗  █  █ █ █ █ ▝▀▙             ",
        "            ▀ ▝▘▝▀▘     ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘  ▝▘ ▝▀▘ ▝▀▘ ▀ ▀ ▀▀▘             ",
    ];

    dashboard.draw(&mut frame, size).unwrap();

    let rendered = frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    let result = [&rendered[10], &rendered[11], &rendered[12], &rendered[13]];

    assert_eq!(result, expected);
}

#[test]
fn test_draw_no_matches_message() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let (_guard, path) = setup_temp_collections(3);
    let collections = collection::collection::get_collections(path).unwrap();
    let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected = [
        "                    █▖▐▌                 ▟      ▜▌                              ",
        "                    █▜▟▌▟▀▙     █▄█▖▝▀▙ ▝█▀ ▟▀▙ ▐▙▜▖▟▀▙ ▟▀▀                     ",
        "                    █ ▜▌█ █     █▜▜▌▟▀█  █▗ █ ▄ ▐▌▐▌█▀▀ ▝▀▙                     ",
        "                    ▀ ▝▘▝▀▘     ▀ ▝▘▝▀▝▘ ▝▘ ▝▀▘ ▀▘▝▘▝▀▘ ▀▀▘                     ",
    ];

    feed_keys(
        &mut dashboard,
        &[
            KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
    );
    dashboard.draw(&mut frame, size).unwrap();

    let rendered = frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    let result = [&rendered[10], &rendered[11], &rendered[12], &rendered[13]];

    assert_eq!(result, expected);
}

#[test]
fn draw_hint_text() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let (_guard, path) = setup_temp_collections(3);
    let collections = collection::collection::get_collections(path).unwrap();
    let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected =
        [" [h/j/k/l to move] [n -> new] [enter -> select item] [? -> help] [<C-c> -> quit]"];

    dashboard.draw(&mut frame, size).unwrap();
    let rendered = get_rendered_from_buffer(&mut frame, size);
    let result = [rendered.last().unwrap().as_str()];

    assert_eq!(result, expected);
}

#[test]
fn draw_filter_prompt() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let (_guard, path) = setup_temp_collections(3);
    let collections = collection::collection::get_collections(path).unwrap();
    let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();
    let expected =
        [" /any_filter                                                                    "];

    feed_keys(
        &mut dashboard,
        &[
            KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('_'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
        ],
    );

    dashboard.draw(&mut frame, size).unwrap();
    let rendered = get_rendered_from_buffer(&mut frame, size);
    let result = [rendered.last().unwrap().as_str()];

    assert_eq!(result, expected);
}

#[test]
fn test_draw_title() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let mut dashboard = CollectionDashboard::new(size, &colors, vec![], false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected = [
        "  ▟▀▙     ▝█           ▟                      ▝█  ▝█           ▟   ▀            ",
        "  ▜▙  ▟▀▙  █  ▟▀▙ ▟▀▙ ▝█▀     ▝▀▙     ▟▀▙ ▟▀▙  █   █  ▟▀▙ ▟▀▙ ▝█▀ ▝█  ▟▀▙ █▀▙   ",
        "  ▄▝█ █▀▀  █  █▀▀ █ ▄  █▗     ▟▀█     █ ▄ █ █  █   █  █▀▀ █ ▄  █▗  █  █ █ █ █   ",
        "  ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘  ▝▘     ▝▀▝▘    ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘  ▝▘ ▝▀▘ ▝▀▘ ▀ ▀   ",
    ];

    dashboard.draw(&mut frame, size).unwrap();

    let rendered = frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    let result = [&rendered[1], &rendered[2], &rendered[3], &rendered[4]];

    assert_eq!(result, expected);
}

#[test]
fn test_draw_error() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let mut dashboard = CollectionDashboard::new(size, &colors, vec![], false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected = [
        "                                                                                ",
        "  ▟▀▙     ▝█       ┌─────────────────────────────────────┐     ▟   ▀            ",
        "  ▜▙  ▟▀▙  █  ▟▀▙ ▟│                                     │▟▀▙ ▝█▀ ▝█  ▟▀▙ █▀▙   ",
        "  ▄▝█ █▀▀  █  █▀▀ █│ any_error_message                   │█ ▄  █▗  █  █ █ █ █   ",
        "  ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝│                                     │▝▀▘  ▝▘ ▝▀▘ ▝▀▘ ▀ ▀   ",
        "                   │                                     │                      ",
        "                   │                                     │                      ",
        "                   │                                     │                      ",
        "                   │                                     │                      ",
        "                   │                                     │                      ",
        "            █▖▐▌   │                                     │                      ",
        "            █▜▟▌▟▀▙│                                     │▙ █▀▙ ▟▀▀             ",
        "            █ ▜▌█ █│                                     │█ █ █ ▝▀▙             ",
        "            ▀ ▝▘▝▀▘│                                     │▘ ▀ ▀ ▀▀▘             ",
        "                   │                                     │                      ",
        "                   │                                     │                      ",
        "                   │                                     │                      ",
        "                   │                                     │                      ",
        "                   │                (O)k                 │                      ",
        "                   │                                     │                      ",
        "                   └─────────────────────────────────────┘                      ",
        "                                                                                ",
    ];

    dashboard.display_error("any_error_message".into());
    dashboard.draw(&mut frame, size).unwrap();

    let rendered = frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    assert_eq!(rendered, expected);
}

#[test]
fn test_draw_help() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let mut dashboard = CollectionDashboard::new(size, &colors, vec![], false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected = [
            "助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助                                                   る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   h/<left>    - select left item                  る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   j/<down>    - select item below                 る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   k/<up>      - select item above                 る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   l/<right>   - select right item                 る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   n/c         - creates a new collection          る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   d           - deletes the selected collection   る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   ?           - toggle this help window           る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   enter       - select item under cursor          る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   /           - enter filter mode                 る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助   <C-c>       - quits the application             る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助                                                   る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助              press any key to go back             る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助                                                   る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 ",
            "助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 け る 助 ",
        ];

    feed_keys(
        &mut dashboard,
        &[KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)],
    );
    dashboard.draw(&mut frame, size).unwrap();

    let rendered = frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    assert_eq!(rendered, expected);
}

#[test]
fn test_draw_form_popup() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let mut dashboard = CollectionDashboard::new(size, &colors, vec![], false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected = [
            "新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新                                       新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新  ┌Name─────────────────────────────┐  新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新  │My awesome API                   │  新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新  └─────────────────────────────────┘  新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新  ┌Description──────────────────────┐  新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新  │Request testing                  │  新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新  └─────────────────────────────────┘  新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新                                       新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新         ╭────────╮ ╭────────╮         新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新         │ Create │ │ Cancel │         新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新         ╰────────╯ ╰────────╯         新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新                                       新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新  [Tab] to switch focus [Enter] to se  新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新                                       新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 ",
            "新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 新 ",
        ];

    feed_keys(
        &mut dashboard,
        &[KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)],
    );
    dashboard.draw(&mut frame, size).unwrap();

    let rendered = frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    println!("{:#?}", frame.buffer_mut().content());

    assert_eq!(rendered, expected);
}

#[test]
fn test_draw_delete_prompt() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let (_guard, path) = setup_temp_collections(3);
    let collections = collection::collection::get_collections(path).unwrap();
    let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected = [
        "                                                                                ",
        "  ▟▀▙     ▝█           ▟                      ▝█  ▝█           ▟   ▀            ",
        "  ▜▙  ▟▀▙  █  ▟▀▙ ▟▀▙ ▝█▀     ▝▀▙     ▟▀▙ ▟▀▙  █   █  ▟▀▙ ▟▀▙ ▝█▀ ▝█  ▟▀▙ █▀▙   ",
        "  ▄▝█ █▀▀  █  █▀▀ █ ▄  █▗     ▟▀█     █ ▄ █ █  █   █  █▀▀ █ ▄  █▗  █  █ █ █ █   ",
        "  ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘  ▝▘     ▝▀▝▘    ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘  ▝▘ ▝▀▘ ▝▀▘ ▀ ▀   ",
        "                                                                                ",
        " ╭────────────────────────────────────╮╭────────────────────────────────────╮ ↑ ",
        " │test_collection_0┌─────────────────────────────────────┐                  │ █ ",
        " │test_description_│                                     │                  │ █ ",
        " ╰─────────────────│  You really want to delete          │──────────────────╯ █ ",
        " ╭─────────────────│  collection test_collection_0?      │                    █ ",
        " │test_collection_2│                                     │                    █ ",
        " │test_description_│             (y)es (n)o              │                    █ ",
        " ╰─────────────────│                                     │                    █ ",
        "                   └─────────────────────────────────────┘                    █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              ↓ ",
        "                                                                                ",
    ];

    feed_keys(
        &mut dashboard,
        &[KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)],
    );
    dashboard.draw(&mut frame, size).unwrap();

    let rendered = frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    println!("{:?}", frame.buffer_mut());

    assert_eq!(rendered, expected);
}

#[test]
fn test_draw_collections_list() {
    let colors = hac_colors::Colors::default();
    let size = Rect::new(0, 0, 80, 22);
    let (_guard, path) = setup_temp_collections(3);
    let collections = collection::collection::get_collections(path).unwrap();
    let mut dashboard = CollectionDashboard::new(size, &colors, collections, false).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    let mut frame = terminal.get_frame();

    let expected = [
        "                                                                                ",
        "  ▟▀▙     ▝█           ▟                      ▝█  ▝█           ▟   ▀            ",
        "  ▜▙  ▟▀▙  █  ▟▀▙ ▟▀▙ ▝█▀     ▝▀▙     ▟▀▙ ▟▀▙  █   █  ▟▀▙ ▟▀▙ ▝█▀ ▝█  ▟▀▙ █▀▙   ",
        "  ▄▝█ █▀▀  █  █▀▀ █ ▄  █▗     ▟▀█     █ ▄ █ █  █   █  █▀▀ █ ▄  █▗  █  █ █ █ █   ",
        "  ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘  ▝▘     ▝▀▝▘    ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘ ▝▀▘  ▝▘ ▝▀▘ ▝▀▘ ▀ ▀   ",
        "                                                                                ",
        " ╭────────────────────────────────────╮╭────────────────────────────────────╮ ↑ ",
        " │test_collection_0                   ││test_collection_1                   │ █ ",
        " │test_description_0                  ││test_description_1                  │ █ ",
        " ╰────────────────────────────────────╯╰────────────────────────────────────╯ █ ",
        " ╭────────────────────────────────────╮                                       █ ",
        " │test_collection_2                   │                                       █ ",
        " │test_description_2                  │                                       █ ",
        " ╰────────────────────────────────────╯                                       █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              █ ",
        "                                                                              ↓ ",
        " [h/j/k/l to move] [n -> new] [enter -> select item] [? -> help] [<C-c> -> quit]",
    ];

    dashboard.draw(&mut frame, size).unwrap();

    let rendered = frame
        .buffer_mut()
        .content
        .chunks(size.width.into())
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();

    assert_eq!(rendered, expected);
}
