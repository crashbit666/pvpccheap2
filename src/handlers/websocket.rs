use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    
    // TODO: Implementar lògica del WebSocket
    // - Autenticació via token
    // - Subscripció a canvis d'estat de dispositius
    // - Notificacions en temps real
    
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Text(text) => {
                    // Handle text message
                    let _ = session.text(text).await;
                }
                Message::Binary(_) => {
                    // Handle binary message
                }
                Message::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                _ => {}
            }
        }
    });
    
    Ok(res)
}
