use std::any::TypeId;

use crossterm::event::Event;
use ratatui_core::{
    buffer::Buffer,
    layout::{Layout, Rect},
};

use crate::{
    element::{Element, Tree},
    runner::Context,
};

pub struct Container<'a, Message> {
    layout: Layout,
    children: Vec<Box<dyn Element<'a, Message> + 'a>>,
}

impl<'a, Message> Container<'a, Message> {
    pub fn layout(layout: Layout) -> Self {
        Self {
            layout,
            children: vec![],
        }
    }

    pub fn with(mut self, child: impl Element<'a, Message> + 'a) -> Self {
        self.children.push(Box::new(child));
        self
    }

    fn for_each_child<F>(&self, tree: &Tree, area: Rect, mut f: F)
    where
        F: FnMut(&(dyn Element<'a, Message> + 'a), &Tree, Rect),
    {
        self.layout
            .split(area)
            .iter()
            .zip(tree.children.iter())
            .zip(self.children.iter())
            .for_each(|((area, tree), child)| {
                f(&**child, tree, *area);
            });
    }
}

impl<'a, Message: 'static> Element<'a, Message> for Container<'a, Message> {
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        self.for_each_child(tree, area, |child, tree, area| {
            child.draw(tree, area, buffer);
        });
    }

    fn id(&self) -> TypeId {
        TypeId::of::<Container<'static, Message>>()
    }

    fn children(&self) -> &[Box<dyn Element<'a, Message> + 'a>] {
        &self.children
    }

    fn update(&self, tree: &Tree, area: Rect, event: Event, ctx: &mut Context<Message>) {
        self.for_each_child(tree, area, |child, tree, area| {
            child.update(tree, area, event.clone(), ctx);
        });
    }
}
