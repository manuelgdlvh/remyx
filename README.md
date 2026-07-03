<div align="center">
  
# Remyx

[![Crates.io](https://img.shields.io/crates/v/remyx.svg)](https://crates.io/crates/remyx)
[![Documentation](https://docs.rs/remyx/badge.svg)](https://docs.rs/remyx/latest/remyx/)
[![Downloads](https://img.shields.io/crates/d/remyx.svg)](https://crates.io/crates/remyx)

Framework for building TUIs on top of Ratatui.  
It focuses on simplicity and ease of use, and it is inspired by Iced.

<img width="600" height="300" alt="html_picker" src="https://github.com/user-attachments/assets/8b23549f-7785-4c52-9b80-cf6b499e5dbe" />

</div>

## Overview

Remyx follows the Elm architecture and is fully compatible with Ratatui widgets.

The framework lets you focus on describing the UI in a declarative way and implementing your application logic, handling behind the scenes the event loop, async tasks, and terminal interactions automatically.

The main trait to implement is `Application`, where you define the `view` and `update` logic.

You can also use the `subscription()` method to define async stream sources that produce your custom `Message` type. These messages are automatically routed to the update logic.

Async tasks can be spawned during initialization or inside the update function by providing a future that returns a message.

## Getting Started

Add `remyx` to your `Cargo.toml`:

```toml
[dependencies]
remyx = { version = "0.1.0-beta.4", features = ["tokio"] }
```

Implement the `Application` trait following these steps:

### 1. Define your state and messages

Your state holds everything the app needs to render. Messages represent things that can happen.

```rust
struct App {
    quote_index: usize,
    exit: bool,
}

enum Message {
    Next,
    Previous,
    Exit,
}
```

### 2. Initialize

Return the initial state. You can optionally return a `Task` to run async work at startup.

```rust
fn init<Runtime: runtime::Runtime>() -> (Self, Option<Task<Message>>) {
    (Self { quote_index: 0, exit: false }, None)
}
```

### 3. Describe the view

Build your UI from the current state. Any Ratatui widget works as an `Element`.

```rust
fn view(&self) -> impl Element<Self::Message> {
    Paragraph::new(QUOTES[self.quote_index])
}
```

### 4. Handle updates

React to messages by changing state. Return a `Task` if you need to do async work.

```rust
fn update<Runtime: runtime::Runtime>(
    &mut self,
    message: Self::Message,
) -> Option<Task<Self::Message>> {
    match message {
        Message::Next => { self.quote_index = (self.quote_index + 1) % QUOTES.len(); }
        Message::Previous => { self.quote_index = self.quote_index.saturating_sub(1); }
        Message::Exit => { self.exit = true; }
    }
    None
}
```

### 5. Subscribe to events

Map terminal events (keys, mouse, ...) to your messages.

```rust
fn subscription<Terminal: terminal::Terminal>(
    &self,
) -> Vec<Subscription<Terminal, Self::Message>> {
    vec![Subscription::key(|key| match key.code {
        KeyCode::Right => Some(Message::Next),
        KeyCode::Left => Some(Message::Previous),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Message::Exit),
        _ => None,
    })]
}
```

### 6. Exit condition

Tell the framework when to stop.

```rust
fn exit(&self) -> bool {
    self.exit
}
```

That's the full cycle: **subscriptions** turn events into messages, **update** changes state, and **view** renders it.

## Status

Remyx is in an early experimental stage. The API is subject to breaking changes in future releases.
