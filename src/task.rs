use crate::stream::Source;

pub struct Task<Message> {
    fut: futures::future::LocalBoxFuture<'static, Message>,
}

impl<Message> Task<Message> {
    pub fn new<Fut: Future<Output = Message> + 'static>(fut: Fut) -> Self {
        Self { fut: Box::pin(fut) }
    }
}

impl<Message: 'static> From<Task<Message>> for Source<Message> {
    fn from(val: Task<Message>) -> Self {
        Source::future(val.fut)
    }
}
