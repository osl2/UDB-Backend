use actix_web::{dev::{ServiceRequest, ServiceResponse, Service, Transform}, HttpMessage, Error};
use futures::future::{ok, Either, FutureResult};
use futures::{Future, Poll};
use lazy_static::lazy_static;
use regex::Regex;
use crate::JwtKey;

pub use frank_jwt::{Algorithm};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
#[derive(Clone)]
pub struct JwtAuthentication {
    pub key: JwtKey,
    pub algorithm: Algorithm,
}

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for JwtAuthentication
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthenticationMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthenticationMiddleware { key: self.key.clone(), algorithm: self.algorithm, service: service })
    }
}

pub struct JwtAuthenticationMiddleware<S> {
    key: JwtKey,
    algorithm: Algorithm,
    service: S,
}

impl<S, B> Service for JwtAuthenticationMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = futures::future::Either<FutureResult<Self::Response, Self::Error> , S::Future>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }



    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let token = match dbg!(get_token(&req)) {
            Ok(token) => token,
            Err(error) => {
                return Either::A(ok(req.into_response(
                    actix_web::HttpResponse::Unauthorized().finish().into_body(),
                )));
            }
        };

        match dbg!(match &self.key {
            JwtKey::Inline(key) => frank_jwt::decode(&token, key, self.algorithm),
            JwtKey::File(key) => frank_jwt::decode(&token, key, self.algorithm),
        }) {
            Ok((header, claims)) => {
                //TODO: frank_jwt does not validate things yet,
                //we should either validate things here or patch frank_jwt
                req.extensions_mut().insert(crate::AuthenticationData(claims))
            }
            Err(error) => {
                return Either::A(ok(req.into_response(
                    actix_web::HttpResponse::Unauthorized().finish().into_body()
                )));
            }
        }

        Either::B(self.service.call(req))
    }
}

fn get_token(req: &ServiceRequest) -> Result<String, String> {
    match req.headers().get(actix_web::http::header::AUTHORIZATION) {
        Some(header_value) => {
            lazy_static! {
                static ref RE: Regex = Regex::new("Bearer (.*)").unwrap();
            }
            match RE.captures(match header_value.to_str() {
                Ok(header) => header,
                Err(_) => {
                    return Err("Header contains non-ASCII characters".to_string());
                }
            }) {
                Some(capture) => match capture.get(1) {
                    Some(matched) => Ok(String::from(matched.as_str())),
                    None => Err("Couldn't find token in authorization header.".to_owned()),
                },
                None => Err("Invalid authorization header.".to_owned()),
            }
        }
        None => Err("No authorization header supplied.".to_owned()),
    }
}
