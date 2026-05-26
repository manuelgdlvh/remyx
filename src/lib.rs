use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io;
use std::pin::pin;

use crate::task::Task;
use crate::{
    element::{Element, Tree},
    stream::Source,
};

pub mod element;
pub mod stream;
pub mod task;

pub async fn run<A: Application, B: Backend>(mut terminal: Terminal<B>) -> io::Result<()> {
    let (mut app, boot_fn) = A::init();

    let mut view = app.view();
    let mut tree = Tree::init(&view);
    let _ = terminal.draw(|frame| {
        view.draw(&tree, frame.area(), frame.buffer_mut());
    });

    let mut subscriptions = app.subscription();
    if let Some(boot_fn) = boot_fn {
        subscriptions.push(boot_fn.into());
    }

    let mut subscription_events = pin!(stream::Stream::init(subscriptions));
    let mut terminal_events = pin!(stream::terminal_event().fuse());
    let mut messages = Vec::new();
    loop {
        futures::select_biased! {
            event = subscription_events.next() => match event {
                Some(msg) =>  {
                      if let Some(task) = app.update(msg) {
                          subscription_events.add(task.into());
                      }

                view = app.view();
                tree.diff(&app.view());
                },
                None => break,
            },
            event = terminal_events.next() => match event {
                Some(Ok(event)) => {
                    let mut shell = Shell::new(&mut messages);
                    let area = terminal.get_frame().area();
                    view.update(&tree, area, event, &mut shell);

                    if !shell.redraw() {
                        continue;
                    }

                    for msg in messages.drain(..) {

                                 if let Some(task) = app.update(msg) {
                          subscription_events.add(task.into());
                      }
                    }

                    view = app.view();
                    tree.diff(&app.view());
                },
                Some(Err(e)) => {
                    return Err(e);
                }
                None => break,
            },
        }

        let _ = terminal.draw(|frame| {
            view.draw(&tree, frame.area(), frame.buffer_mut());
        });
    }

    Ok(())
}

pub trait Application {
    type Message: 'static;

    fn init() -> (Self, Option<Task<Self::Message>>)
    where
        Self: Sized;
    fn view(&self) -> impl Element<Self::Message> + use<Self>;
    fn update(&mut self, message: Self::Message) -> Option<Task<Self::Message>>;
    fn subscription(&self) -> Vec<Source<Self::Message>> {
        vec![]
    }
}

#[derive(Debug)]
pub struct Shell<'a, Message> {
    messages: &'a mut Vec<Message>,
    redraw_requested: bool,
}

impl<'a, Message> Shell<'a, Message> {
    pub fn new(messages: &'a mut Vec<Message>) -> Self {
        Self {
            messages,
            redraw_requested: false,
        }
    }

    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn redraw(&self) -> bool {
        self.redraw_requested || !self.messages.is_empty()
    }

    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }
}
