use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
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

    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);

        let _event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);

            _event_tx.send(Event::Init).unwrap();

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(CrosstermEvent::Key(key_event))) => {
                                if key_event.kind == KeyEventKind::Press {
                                    _event_tx.send(Event::Key(key_event)).unwrap();
                                }
                            }
                            Some(Ok(CrosstermEvent::Mouse(mouse_event))) => _event_tx.send(Event::Mouse(mouse_event)).unwrap(),
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

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}
