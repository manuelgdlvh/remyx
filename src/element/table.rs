use std::any::TypeId;
use std::borrow::Borrow;

use crate::{
    element::{Element, State, Tree},
    runner::Context,
};
use ratatui_core::widgets::StatefulWidget;
use ratatui_core::{buffer::Buffer, layout::Rect};
use remyx_widgets::{
    focus::Focusable,
    table::{Row, Table, TableState},
};

impl<'a, Item, Items, Message> Element<'a, Message> for Table<'a, Item, Items, Message>
where
    Message: 'static,
    Item: PartialEq + 'static,
    Items: Borrow<[Item]> + 'a,
    for<'b> Row<'a>: From<&'b Item>,
{
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        tree.state_mut::<TableState, _, _>(|s| {
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
        enum Movement {
            Previous,
            Next,
            At(usize),
        }

        if !ctx.cursor().is_hovering(area) && !self.is_focused() {
            return;
        }

        let items_area = self.items_layout(area);
        let movement = match event {
            crossterm::event::Event::Key(key_event) => match key_event.code {
                crossterm::event::KeyCode::Up => Some(Movement::Previous),
                crossterm::event::KeyCode::Down => Some(Movement::Next),
                _ => None,
            },
            crossterm::event::Event::Mouse(mouse_event) => match mouse_event.kind {
                crossterm::event::MouseEventKind::Up(mouse_button)
                    if mouse_button.eq(&crossterm::event::MouseButton::Left) =>
                {
                    ctx.cursor()
                        .is_hovering(items_area)
                        .then(|| {
                            let offset = tree.state::<TableState, _, _>(|s| s.offset());
                            let click_position = (mouse_event.row - items_area.y) as usize;

                            let mut item_height = 0usize;
                            self.row_heights().enumerate().skip(offset).find_map(
                                |(index, height)| {
                                    item_height += height as usize;
                                    (click_position < item_height).then_some(Movement::At(index))
                                },
                            )
                        })
                        .flatten()
                }
                crossterm::event::MouseEventKind::ScrollUp => {
                    tree.state_mut::<TableState, _, _>(|s| {
                        *s.offset_mut() = s.offset().saturating_sub(1);
                    });
                    ctx.redraw();
                    None
                }
                crossterm::event::MouseEventKind::ScrollDown => {
                    let max_offset = self.len().saturating_sub(1);
                    tree.state_mut::<TableState, _, _>(|s| {
                        *s.offset_mut() = (s.offset() + 1).min(max_offset);
                    });
                    ctx.redraw();
                    None
                }
                _ => None,
            },
            _ => None,
        };

        if let Some(movement) = movement {
            let len = self.len();
            if len == 0 {
                return;
            }

            let current = self.selected();
            let new_index = match &movement {
                Movement::Previous => current.map_or(0, |i| i.saturating_sub(1)),
                Movement::Next => current.map_or(0, |i| (i + 1).min(len - 1)),
                Movement::At(index) => (*index).min(len - 1),
            };

            if current == Some(new_index) {
                return;
            }

            if matches!(movement, Movement::Previous | Movement::Next) {
                let offset = tree.state::<TableState, _, _>(|s| s.offset());
                if !is_selected_visible(
                    new_index,
                    offset,
                    items_area.height as usize,
                    self.row_heights(),
                ) {
                    tree.state_mut::<TableState, _, _>(|s| {
                        *s.offset_mut() = new_index;
                    });
                }
            }

            ctx.redraw();
            if let Some(item) = self.items().get(new_index) {
                ctx.publish(self.on_select_fn()(item));
            }
        }
    }

    fn diff(&self, tree: &mut Tree) {
        let old_length = tree.state::<TableState, _, _>(|s| s.len());
        if old_length > self.len() {
            tree.state = Element::<'a, Message>::state(self);
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<Table<'static, Item, &'static [Item], Message>>()
    }

    fn state(&self) -> Option<State> {
        let length = self.len();
        Some(State::new(TableState::new(length)))
    }
}

fn is_selected_visible(
    index: usize,
    offset: usize,
    visible_height: usize,
    row_heights: impl Iterator<Item = u16>,
) -> bool {
    let mut height_sum = 0usize;
    for (i, h) in row_heights.enumerate().skip(offset) {
        height_sum += h as usize;
        if height_sum > visible_height {
            return false;
        }
        if i == index {
            return true;
        }
    }
    false
}
