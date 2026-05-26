use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io;

use crate::runner::{Application, Runner};

pub mod element;
pub mod runner;
pub mod stream;
pub mod task;

pub async fn run<A, B>(terminal: Terminal<B>) -> io::Result<()>
where
    A: Application,
    B: Backend,
{
    Runner::<A, B>::new(terminal)?.run().await
}
