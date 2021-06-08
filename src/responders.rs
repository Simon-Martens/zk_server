use crate::serializables::ResponseBodyGeneric;
use rocket::http::ContentType;
use rocket::http::Header;
use rocket::http::HeaderMap;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::Responder;
use rocket::response::Response;
use rocket::response::Result;
use rocket_contrib::json::JsonValue;

#[derive(Debug)]
pub(crate) struct ApiResponse {
    headers: Vec<(String, String)>,
    response: ResponseBodyGeneric,
    status: Status,
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> Result<'r> {
        let mut res = Response::build_from(self.response.json().respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .finalize();
        for (header, value) in self.headers {
            let res = res.set_raw_header(header.clone(), value.clone());
        }
        Ok(res)
    }
}

impl ApiResponse {
    pub(crate) fn ok(response: ResponseBodyGeneric) -> ApiResponse {
        ApiResponse {
            headers: Vec::default(),
            status: Status::Ok,
            response,
        }
    }

    pub(crate) fn not_found(response: ResponseBodyGeneric) -> ApiResponse {
        ApiResponse {
            headers: Vec::default(),
            status: Status::NotFound,
            response,
        }
    }

    pub(crate) fn unauthorized(response: ResponseBodyGeneric) -> ApiResponse {
        ApiResponse {
            headers: vec![(r#"Clear-Site-Data"#.to_string(), r#""*""#.to_string())],
            status: Status::Unauthorized,
            response,
        }
    }

    pub(crate) fn forbidden(response: ResponseBodyGeneric) -> ApiResponse {
        ApiResponse {
            headers: vec![(r#"Clear-Site-Data"#.to_string(), r#""*""#.to_string())],
            status: Status::Forbidden,
            response,
        }
    }
}
