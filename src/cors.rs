use rocket::response::{Responder, Response};
use rocket::http::Status;

pub struct CORS<R>(pub Option<R>);

impl<'r, R: Responder<'r>> Responder<'r> for CORS<R> {
    default fn respond(self) -> Result<Response<'r>, Status> {
        let mut build = Response::build();
        if let Some(inner_res) = self.0 {
            build.merge(inner_res.respond()?);
        }

        build.raw_header("access-control-allow-origin", "*")
            .raw_header("access-control-Allow-Methods",
                        "OPTIONS, GET, POST, PATCH, DELETE")
            .raw_header("access-control-allow-headers", "Content-Type")
            .ok()
    }
}
