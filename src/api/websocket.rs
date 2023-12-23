use rocket::State;
use rocket_ws::{Message, Stream, WebSocket};

use crate::routes::SharedChannel;

#[get("/ws")]
pub fn echo_stream(state: &State<SharedChannel>, ws: WebSocket) -> Stream!['static] {
    let mut rx = state.receiver.resubscribe();

    Stream! { ws =>
        for await message in ws {
            yield message?;

            match rx.recv().await {
                Ok(uid_message) => {
                    yield Message::Text(uid_message)
                },
                Err(e) => {
                    // Handle error or channel closure
                }
            }
        }
    }
}