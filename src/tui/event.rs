use crate::error::AppError;
use crossterm::event::{self, Event as CrosserEvent, KeyEvent, KeyEventKind};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
}


pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
}


impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);

        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut last_tick = Instant::now();

            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::ZERO);

                if event::poll(timeout).expect("failed to poll terminal events") {
                    match event::read().expect("failed to read terminal event") {
                        CrosserEvent::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                                sender
                                    .send(Event::Key(key))
                                    .ok();
                            }
                        }

                        CrosserEvent::Resize(width, height) => {
                            sender
                                .send(Event::Resize(width, height))
                                .ok();
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if sender.send(Event::Tick).is_err() {
                        break;
                    }

                    last_tick = Instant::now();
                }
            }
        });

        Self { receiver }
    }

    pub fn next(&self) -> Result<Event, AppError> {
        self.receiver
            .recv()
            .map_err(|_| {
                AppError::InvalidOperation(
                    "Event channel closed".into()
                )
            })
    }
}