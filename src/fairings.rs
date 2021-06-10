use rocket::fairing::Fairing;
use rocket::fairing::Info;
use rocket::fairing::Kind;
use rocket::http::ContentType;
use rocket::http::Header;
use rocket::request::Request;
use rocket::response::Response;

pub(crate) struct Gzip;

#[rocket::async_trait]
impl Fairing for Gzip {
    fn info(&self) -> Info {
        Info {
            name: "gzip compression",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        use flate2::{Compression, FlateReadExt};
        use std::io::{Cursor, Read};
        let headers = request.headers();
        if headers
            .get("Accept-Encoding")
            .any(|e| e.to_lowercase().contains("gzip"))
        {
            response.body_mut().to_bytes().await.and_then(|body| {
                let mut enc = body.gz_encode(Compression::Default);
                let mut buf: Vec<u8> = Vec::with_capacity(body.len());
                enc.read_to_end(&mut buf)
                    .map(|s| {
                        response.set_sized_body(s, Cursor::new(buf));
                        response.set_raw_header("Content-Encoding", "gzip");
                    });
                Ok(body)
            });
        }
    }
}

pub(crate) struct XClacksOverhead;

#[rocket::async_trait]
impl Fairing for XClacksOverhead {
    fn info(&self) -> Info {
        Info {
            name: "X-Clacks-Overhead",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("X-Clacks-Overhead", "GNU Terry Pratchett"));
    }
}

pub(crate) struct Caching;

#[rocket::async_trait]
impl Fairing for Caching {
    fn info(&self) -> Info {
        Info {
            name: "cache control",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if response.content_type() == Some(ContentType::JavaScript)
            || response.content_type() == Some(ContentType::CSS)
            || response.content_type() == Some(ContentType::Icon)
            || response.content_type() == Some(ContentType::PNG)
            || response.content_type() == Some(ContentType::GIF)
            || response.content_type() == Some(ContentType::JPEG)
        {
            response.set_raw_header("Cache-Control", "public, max-age=604800, immutable");
        }
    }
}

pub(crate) struct CORS {
    pub(crate) origin: String,
}

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "CORS headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new(
            "Access-Control-Allow-Origin",
            self.origin.clone(),
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PUT, POST, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Max-Age", "86400"));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Cookie, Content-Type",
        ));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

pub(crate) struct XFRameOptions;

#[rocket::async_trait]
impl Fairing for XFRameOptions {
    fn info(&self) -> Info {
        Info {
            name: "X-Frame-Options",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let headers = request.headers();
        if headers
            .get("Accept-Encoding")
            .any(|e| e.to_lowercase().contains("gzip"))
        {
            response.set_header(Header::new("X-Frame-Options", "deny"));
        }
    }
}
