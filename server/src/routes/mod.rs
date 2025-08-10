pub mod games;
pub mod hits;
pub mod users;

use crate::GlobalEvent;
use rocket::{
    Shutdown, State,
    response::stream::{Event, EventStream},
    tokio::{
        select,
        sync::broadcast::{Sender, error::RecvError},
    },
};

#[get("/events")]
pub async fn events(queue: &State<Sender<GlobalEvent>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg).event(msg.get_name());
        }
    }
}
