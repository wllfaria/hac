use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::layout::Rect;
use std::ops::Div;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(Rect),
    Init,
    Tick,
    Render,
}

#[derive(Debug)]
pub struct EventPool {
    event_rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    event_tx: tokio::sync::mpsc::UnboundedSender<Event>,
    task: JoinHandle<()>,
    frame_rate: f64,
    tick_rate: f64,
}

impl EventPool {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let task = tokio::spawn(async {});

        EventPool {
            event_rx,
            event_tx,
            task,
            frame_rate,
            tick_rate,
        }
    }

    #[cfg_attr(test, mutants::skip)]
    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0.div(self.tick_rate));
        let render_delay = std::time::Duration::from_secs_f64(1.0.div(self.frame_rate));

        let _event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);

            _event_tx
                .send(Event::Init)
                .expect("failed to send event through channel");

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(CrosstermEvent::Key(key_event))) => {
                                if key_event.kind == KeyEventKind::Press {
                                    _event_tx.send(Event::Key(key_event)).expect("failed to send event through channel");
                                }
                            }
                            Some(Ok(CrosstermEvent::Resize(width, height))) => _event_tx
                                .send(Event::Resize(Rect::new(0, 0, width, height)))
                                .expect("failed to send event through channel"),
                            Some(Err(_)) => {}
                            Some(_) => {}
                            None => {}
                        }
                    }
                    _ = tick_delay => {
                        _event_tx.send(Event::Tick).unwrap();
                    },
                    _ = render_delay => {
                        _event_tx.send(Event::Render).unwrap();
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
