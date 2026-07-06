use std::any::TypeId;
use std::borrow::Borrow;

use crate::{
    element::{Element, State, Tree},
    runner::Context,
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui_core::widgets::StatefulWidget;
use ratatui_core::{buffer::Buffer, layout::Rect};
use ratatui_textarea::{TextArea, TextAreaState};

impl<'a, Content, Message> Element<'a, Message> for TextArea<'a, Content, Message>
where
    Message: 'static,
    Content: Borrow<str> + 'a,
{
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        tree.state_mut::<TextAreaState, _, _>(|s| {
            self.render(area, buffer, s);
        });
    }

    fn update(
        &self,
        tree: &Tree,
        area: Rect,
        event: crossterm::event::Event,
        ctx: &mut Context<Message>,
    ) {
        if !ctx.cursor().is_hovering(area) && !self.is_focused() {
            return;
        }

        if let Event::Key(key) = &event {
            let is_enter = matches!(key.code, KeyCode::Enter)
                || (key.code == KeyCode::Char('m')
                    && key.modifiers.contains(KeyModifiers::CONTROL));
            if is_enter
                && !key.modifiers.contains(KeyModifiers::SHIFT)
                && let Some(on_submit) = self.on_submit_fn()
            {
                tree.state::<TextAreaState, _, _>(|s| {
                    ctx.publish(on_submit(s.content()));
                });
                ctx.redraw();
                return;
            }
        }

        tree.state_mut::<TextAreaState, _, _>(|s| {
            s.sync_edit_config(self.tab_len(), self.is_hard_tab_indent());
            let modified = s.input(event);
            if modified {
                ctx.publish((self.on_input_fn())(s.content()));
            }
        });
        ctx.redraw();
    }

    fn diff(&self, tree: &mut Tree) {
        let content: &str = self.content();
        let changed = tree.state::<TextAreaState, _, _>(|s| s.content() != content);
        if changed {
            tree.state = Element::<'a, Message>::state(self);
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<TextArea<'static, &'static str, Message>>()
    }

    fn state(&self) -> Option<State> {
        Some(State::new(TextAreaState::new(self.content())))
    }
}
