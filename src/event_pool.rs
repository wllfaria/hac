use crossterm::event::Event as CrosstermEvent;
use futures::{FutureExt, StreamExt};

#[derive(Debug)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
}

#[derive(Debug)]
pub struct EventPool {
    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

impl EventPool {
    pub fn new() -> Self {
        let tick_rate = std::time::Duration::from_millis(30);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);

            loop {
                let delay = interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(CrosstermEvent::Key(key))) => tx.send(Event::Key(key)).expect("failed to send event through channel"),
                            Some(Err(_)) => {}
                            Some(_) => {}
                            None => {}
                        }
                    }
                    _ = delay => {}
                }
            }
        });

        EventPool { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
