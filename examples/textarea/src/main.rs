use remyx::crossterm::crossterm::event::{
    DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, KeyCode,
    KeyEvent, KeyModifiers,
};
use remyx::crossterm::crossterm::execute;
use remyx::crossterm::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use remyx::element::container::Container;
use remyx::ratatui::layout::{Constraint, Layout};
use remyx::ratatui::style::{Color, Modifier, Style};
use remyx::runtime::tokio::Tokio;
use remyx::terminal::crossterm::Crossterm;
use remyx::textarea::TextArea;
use remyx::widgets::block::Block;
use remyx::widgets::borders::Borders;
use remyx::widgets::focus::Focusable;
use remyx::widgets::paragraph::Paragraph;
use remyx::{Application, Element, Subscription, Task, runtime, terminal};
use std::io;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnableMouseCapture,
        EnableFocusChange,
        EnableBracketedPaste,
    )?;

    let terminal = Crossterm::<Tokio>::new();
    remyx::run::<App, Tokio, _>(terminal)?;

    execute!(io::stdout(), DisableMouseCapture)?;
    disable_raw_mode()
}

pub struct App {
    input: String,
    messages: Vec<String>,
    focus: Focus,
    exit: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Log,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Input => Focus::Log,
            Focus::Log => Focus::Input,
        }
    }
}

pub enum Message {
    InputChanged(String),
    Submit(String),
    FocusNext,
    Exit,
}

impl Application for App {
    type Message = Message;

    fn init<Runtime: runtime::Runtime>() -> (Self, Option<Task<Message>>) {
        let app = Self {
            input: String::new(),
            messages: Vec::new(),
            focus: Focus::Input,
            exit: false,
        };
        (app, None)
    }

    fn view(&self) -> impl Element<'_, Self::Message> {
        let input_border_color = if self.focus == Focus::Input {
            Color::Yellow
        } else {
            Color::Cyan
        };
        let log_border_color = if self.focus == Focus::Log {
            Color::Yellow
        } else {
            Color::Blue
        };

        let textarea = TextArea::new(self.input.as_str(), Message::InputChanged)
            .on_submit(Message::Submit)
            .block(
                Block::default()
                    .title_top("Input (Enter=submit, Shift+Enter=newline)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(input_border_color)),
            )
            .style(Style::default().fg(Color::White))
            .cursor_style(Style::default().add_modifier(Modifier::REVERSED))
            .placeholder("Type a message...")
            .focus(self.focus == Focus::Input);

        let log_text = if self.messages.is_empty() {
            "No messages yet. Type something and press Enter!".to_string()
        } else {
            self.messages
                .iter()
                .enumerate()
                .map(|(i, msg)| format!("[{}] {}", i + 1, msg))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let log = Paragraph::new(log_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Messages")
                    .border_style(Style::default().fg(log_border_color)),
            )
            .style(Style::default().fg(Color::White))
            .focus(self.focus == Focus::Log);

        Container::layout(Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ]))
        .with(textarea)
        .with(log)
    }

    fn update<Runtime: runtime::Runtime>(
        &mut self,
        message: Self::Message,
    ) -> Option<Task<Self::Message>> {
        match message {
            Message::InputChanged(text) => {
                self.input = text;
                None
            }
            Message::Submit(text) => {
                if !text.trim().is_empty() {
                    self.messages.push(text);
                    self.input.clear();
                }
                None
            }
            Message::FocusNext => {
                self.focus = self.focus.next();
                None
            }
            Message::Exit => {
                self.exit = true;
                None
            }
        }
    }

    fn subscription<Terminal: terminal::Terminal>(
        &self,
    ) -> Vec<Subscription<Terminal, Self::Message>> {
        vec![Subscription::key(handle_keys)]
    }

    fn exit(&self) -> bool {
        self.exit
    }
}

fn handle_keys(key: KeyEvent) -> impl futures::Stream<Item = Message> {
    let msg = match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Message::Exit),
        KeyCode::Tab => Some(Message::FocusNext),
        _ => None,
    };
    futures::stream::iter(msg)
}
