use rocket::request::Request;
use rocket::http::Status;
use rocket::http::ContentType;
use rocket::response::Responder;
use rocket::response::Response;
use rocket::response::Result;
use rocket_contrib::json::JsonValue;
use crate::serializables::ResponseBodyGeneric;


#[derive(Debug)]
pub(crate) struct ApiResponse {
    pub(crate) json: ResponseBodyGeneric<JsonValue>,
    pub(crate) status: Status,
}


impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> Result<'r> {
        let j = json!(self.json);
        Response::build_from(
            j.respond_to(&req).unwrap()
        )
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

