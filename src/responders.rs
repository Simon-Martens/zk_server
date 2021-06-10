use crate::serializables::ResponseBodyGeneric;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::Responder;
use rocket::response::Response;
use rocket::response::Result;

#[derive(Debug)]
pub(crate) struct ApiResponse {
    headers: Vec<(String, String)>,
    response: ResponseBodyGeneric,
    status: Status,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ApiResponse {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        let mut res = Response::build_from(self.response.json().respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .finalize();
        for (header, value) in self.headers {
            res.set_raw_header(header.clone(), value.clone());
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

    #[allow(unused)]
    pub(crate) fn forbidden(response: ResponseBodyGeneric) -> ApiResponse {
        ApiResponse {
            headers: vec![(r#"Clear-Site-Data"#.to_string(), r#""*""#.to_string())],
            status: Status::Forbidden,
            response,
        }
    }
}
