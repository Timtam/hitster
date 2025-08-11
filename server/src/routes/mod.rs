pub mod games;
pub mod hits;
pub mod users;

use crate::{GlobalEvent, services::ServiceStore};
use rocket::{
    Shutdown, State,
    response::stream::{Event, EventStream},
    tokio::{
        select,
        sync::broadcast::{Sender, error::RecvError},
    },
};

#[get("/events")]
pub async fn events(
    svc: &State<ServiceStore>,
    queue: &State<Sender<GlobalEvent>>,
    mut end: Shutdown,
) -> EventStream![] {
    let hs = svc.hit_service();
    let mut rx = queue.subscribe();

    let available = hs.lock().get_hits().len();
    let downloading = hs.lock().downloading();
    let processing = hs.lock().processing();
    let _ = queue.send(GlobalEvent::ProcessHits {
        available,
        downloading,
        processing,
    });

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
