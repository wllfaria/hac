use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::layout::Rect;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Resize(Rect),
}

/// Core component responsible for pooling events from crossterm and sending
/// them over to be handled
#[derive(Debug)]
pub struct EventPool {
    event_rx: std::sync::mpsc::Receiver<Event>,
    event_tx: std::sync::mpsc::Sender<Event>,
}

impl Default for EventPool {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPool {
    pub fn new() -> Self {
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        EventPool { event_rx, event_tx }
    }

    #[cfg_attr(test, mutants::skip)]
    pub fn start(&mut self) {
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();

            loop {
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(CrosstermEvent::Key(key_event))) => {
                                if key_event.kind == KeyEventKind::Press {
                                    event_tx.send(Event::Key(key_event)).expect("failed to send event through channel");
                                }
                            }
                            Some(Ok(CrosstermEvent::Resize(width, height))) => event_tx
                                .send(Event::Resize(Rect::new(0, 0, width, height)))
                                .expect("failed to send event through channel"),
                            _ => {}
                        }
                    }
                }
            }
        });
    }

    #[cfg_attr(test, mutants::skip)]
    pub fn next_event(&mut self) -> Option<Event> {
        self.event_rx.try_recv().ok()
    }
}
