use actix_web::{dev::{ServiceRequest, ServiceResponse, Service, Transform}, HttpMessage, Error};
use futures::future::{ok, Either, FutureResult};
use futures::{Poll};
use lazy_static::lazy_static;
use regex::Regex;
use crate::JwtKey;
use log;
use chrono::{Utc, TimeZone};

pub use frank_jwt::{Algorithm};


/// JWT based authentication middleware for actix-web
#[derive(Clone)]
pub struct JwtAuthentication {
    /// The keys used for verifying the tokens
    pub key: JwtKey,
    /// The algorithm used for verifying the tokens
    pub algorithm: Algorithm,
    /// Regex to match paths that do not need authentication
    pub except: Regex,
}

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
        ok(JwtAuthenticationMiddleware {
            key: self.key.clone(),
            algorithm: self.algorithm,
            except: self.except.clone(),
            service: service,
        })
    }
}

pub struct JwtAuthenticationMiddleware<S> {
    key: JwtKey,
    algorithm: Algorithm,
    except: Regex,
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
        if self.except.is_match(req.path()) {
            return Either::B(self.service.call(req));
        }
        let token = match get_token(&req) {
            Ok(token) => token,
            Err(error) => {
                log::debug!("Could not extract token from request: {}", error);
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
                let auth_data = crate::AuthenticationData {
                    header: header,
                    claims: crate::Claims {
                        all: claims.clone(),
                        sub: match claims.clone() {
                            serde_json::Value::Object(map) => match map.get("sub") {
                                Some(sub) => match sub {
                                    serde_json::Value::String(sub) => Some(sub.to_owned()),
                                    _ => None,
                                },
                                _ => None,
                            },
                            _ => None,
                        },
                        exp: match claims {
                            serde_json::Value::Object(map) => match map.get("exp") {
                                Some(exp) => match exp {
                                    serde_json::Value::Number(exp) => exp.as_i64(),
                                    _ => None,
                                },
                                _ => None,
                            },
                            _ => None,
                        },
                    },
                };
                match auth_data.claims.exp {
                    Some(exp) => {
                        if Utc.timestamp(exp, 0) > Utc::now() {
                            return Either::A(ok(req.into_response(
                                actix_web::HttpResponse::Unauthorized().finish().into_body(),
                            )));
                        }
                    }
                    _ => (),
                }
                req.extensions_mut().insert(auth_data);
                Either::B(self.service.call(req))
            }
            Err(error) => {
                log::debug!("Could not decode token: {}", error);
                Either::A(ok(req.into_response(
                    actix_web::HttpResponse::Unauthorized().finish().into_body()
                )))
            }
        }
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
