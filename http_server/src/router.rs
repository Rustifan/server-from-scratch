use std::{io::Write};

use http::http_request::{HttpRequest, Method};

use crate::handler::{StaticPageHandler, PageNotFoundHandler, Handler};

pub struct Router;

impl Router{
    pub fn route(req: HttpRequest, mut stream: &mut impl Write){
        let response = match req.method {
            Method::Get => {
                StaticPageHandler::handle(&req)
            }
            _=>PageNotFoundHandler::handle(&req)
        };
        let _ = response.send_response(&mut stream);
    }
}