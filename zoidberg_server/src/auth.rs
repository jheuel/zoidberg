use actix_web::{dev, error::ErrorBadRequest, Error, FromRequest, HttpRequest, Result};
use futures::future::{err, ok, Ready};

pub struct Authorization {}

impl FromRequest for Authorization {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        if let Some(head) = req.headers().get("cookie") {
            if let Ok(cookie) = head.to_str() {
                if let Some(secret) = req.app_data::<String>() {
                    if secret == cookie {
                        return ok(Authorization {});
                    } else {
                        return err(ErrorBadRequest("no auth"));
                    }
                }
            }
        }
        err(ErrorBadRequest("no auth"))
    }
}
