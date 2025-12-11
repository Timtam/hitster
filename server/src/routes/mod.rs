pub mod captcha;
pub mod games;
pub mod hits;
pub mod users;

use crate::{GlobalEvent, services::ServiceStore};
use rocket::{
    Shutdown, State,
    futures::stream::Stream,
    response::stream::{Event, EventStream},
    tokio::{
        select,
        sync::broadcast::{Sender, error::RecvError},
    },
};
use rocket_okapi::openapi;

/// # Subscribe to global events
///
/// All global events will be distributed via this event stream (Server-Side Events) in real-time.
/// The following table lists all the possible payloads that are provided as JSON. The event name also is the root key of the JSON object received.
/// If no key is specified, the returned value is the direct value for the event name key.
///
/// <table>
///   <thead>
///     <th>Event Name</th>
///     <th>Key</th>
///     <th>Description</th>
///   </thead>
///   <tbody>
///     <tr>
///       <td>create_game</td>
///       <td></td>
///       <td>Game object of the game that was just created</td>
///     </tr>
///     <tr>
///       <th rowSpan="4">process_hits</th>
///     </tr>
///     <tr>
///       <td>available</td>
///       <td>amount of hits downloaded</td>
///     </tr>
///     <tr>
///       <td>downloading</td>
///       <td>amount of hits currently downloading</td>
///     </tr>
///     <tr>
///       <td>processing</td>
///       <td>amount of hits currently processing</td>
///     </tr>
///     <tr>
///       <td>remove_game</td>
///       <td></td>
///       <td>ID of the game that was removed</td>
///     </tr>
///   </tbody>
/// </table>

#[openapi(tag = "Global")]
#[get("/events")]
pub async fn events(
    svc: &State<ServiceStore>,
    queue: &State<Sender<GlobalEvent>>,
    mut end: Shutdown,
) -> EventStream<impl Stream<Item = Event>> {
    let hs = svc.hit_service();
    let mut rx = queue.subscribe();

    let hsl = hs.lock();

    let available = hsl.get_hits().iter().filter(|h| h.downloaded).count();
    let downloading = hsl.downloading();
    let processing = hsl.processing();

    drop(hsl);

    EventStream! {
        let msg = GlobalEvent::ProcessHits {
            available,
            downloading,
            processing,
        };

        yield Event::json(&msg).event(msg.get_name());

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
