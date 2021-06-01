use rocket::fairing::Fairing;
use rocket::fairing::Info;
use rocket::fairing::Kind;
use rocket::http::ContentType;
use rocket::http::Header;
use rocket::response::Response;
use rocket::request::Request;

pub(crate) struct Gzip;

impl Fairing for Gzip {
    fn info(&self) -> Info {
        Info {
            name: "gzip compression",
            kind: Kind::Response,
        }
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        use flate2::{Compression, FlateReadExt};
        use std::io::{Cursor, Read};
        let headers = request.headers();
        if headers
            .get("Accept-Encoding")
            .any(|e| e.to_lowercase().contains("gzip"))
        {
            response.body_bytes().and_then(|body| {
                let mut enc = body.gz_encode(Compression::Default);
                let mut buf = Vec::with_capacity(body.len());
                enc.read_to_end(&mut buf)
                    .map(|_| {
                        response.set_sized_body(Cursor::new(buf));
                        response.set_raw_header("Content-Encoding", "gzip");
                    })
                    .map_err(|e| eprintln!("{}", e)).ok()
            });
        }
    }
}

pub(crate) struct Caching; 

impl Fairing for Caching {
    fn info(&self) -> Info {
        Info {
            name: "cache control",
            kind: Kind::Response,
        }
    }
    
    fn on_response(&self, _: &Request, response: &mut Response) {
        if response.content_type() == Some(ContentType::JavaScript) ||
            response.content_type() == Some(ContentType::CSS) ||
            response.content_type() == Some(ContentType::Icon) ||
            response.content_type() == Some(ContentType::PNG) ||
            response.content_type() == Some(ContentType::GIF) ||
            response.content_type() == Some(ContentType::JPEG)  {
                response.set_raw_header("Cache-Control", "public, max-age=604800, immutable");
            }
    }
}