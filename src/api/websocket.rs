use rocket_ws::{Stream, WebSocket};

#[get("/ws")]
pub fn echo_stream(ws: WebSocket) -> Stream!['static] {
    Stream! { ws =>
        for await message in ws {
            yield message?;
        }
    }
}