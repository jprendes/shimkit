use shimkit::event::EventPublisher;

#[derive(Clone)]
pub struct Server {
    pub _publisher: EventPublisher,
}

mod sandbox;
mod task;
