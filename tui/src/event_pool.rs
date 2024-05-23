use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::layout::Rect;
use std::ops::Div;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Resize(Rect),
    Tick,
    Render,
}

/// Core component responsible for pooling events from crossterm and sending
/// them over to be handled
#[derive(Debug)]
pub struct EventPool {
    event_rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    event_tx: tokio::sync::mpsc::UnboundedSender<Event>,
    frame_rate: f64,
    tick_rate: f64,
}

impl EventPool {
    pub fn new(frame_rate: f64, tick_rate: f64) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

        EventPool {
            event_rx,
            event_tx,
            frame_rate,
            tick_rate,
        }
    }

    #[cfg_attr(test, mutants::skip)]
    pub fn start(&mut self) {
        let render_delay = std::time::Duration::from_secs_f64(1.0.div(self.frame_rate));
        let tick_delay = std::time::Duration::from_secs_f64(1.0.div(self.tick_rate));

        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut render_interval = tokio::time::interval(render_delay);
            let mut tick_interval = tokio::time::interval(tick_delay);

            loop {
                let render_delay = render_interval.tick();
                let tick_delay = tick_interval.tick();
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
                    _ = tick_delay => {
                        event_tx.send(Event::Tick).expect("failed to send event through channel");
                    },
                    _ = render_delay => {
                        event_tx.send(Event::Render).expect("failed to send event through channel");
                    },
                }
            }
        });
    }

    #[cfg_attr(test, mutants::skip)]
    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}
