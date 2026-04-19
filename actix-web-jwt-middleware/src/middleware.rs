use crate::JwtKey;
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
    http::Method,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
pub use jsonwebtoken::Algorithm;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{future::{ready, Ready}, rc::Rc};

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    sub: Option<String>,
    exp: Option<i64>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// JWT based authentication middleware for actix-web
#[derive(Clone)]
pub struct JwtAuthentication {
    /// The keys used for verifying the tokens
    pub key: JwtKey,
    /// The algorithm used for verifying the tokens
    pub algorithm: Algorithm,
    /// Regexes to match paths and a list of methods on those that do not need authentication
    pub except: Vec<(Regex, Vec<Method>)>,
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuthentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthenticationMiddleware {
            key: self.key.clone(),
            algorithm: self.algorithm,
            except: self.except.clone(),
            service: Rc::new(service),
        }))
    }
}

pub struct JwtAuthenticationMiddleware<S> {
    key: JwtKey,
    algorithm: Algorithm,
    except: Vec<(Regex, Vec<Method>)>,
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        for (reg, methods) in &self.except {
            if reg.is_match(req.path()) && methods.contains(req.method()) {
                let fut = self.service.call(req);
                return Box::pin(async move {
                    fut.await.map(|res| res.map_into_left_body())
                });
            }
        }

        let token = match get_token(&req) {
            Ok(token) => token,
            Err(error) => {
                log::debug!("Could not extract token from request: {}", error);
                return Box::pin(async move {
                    Ok(req.into_response(
                        actix_web::HttpResponse::Unauthorized().finish(),
                    ).map_into_right_body())
                });
            }
        };

        let key_str = match &self.key {
            JwtKey::Inline(key) => key.clone(),
            JwtKey::File(path) => match std::fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => {
                    log::error!("Couldn't read JWT key file: {}", e);
                    return Box::pin(async move {
                        Ok(req.into_response(
                            actix_web::HttpResponse::InternalServerError().finish(),
                        ).map_into_right_body())
                    });
                }
            },
        };

        let algorithm = self.algorithm;
        let mut validation = Validation::new(algorithm);
        validation.validate_exp = false;

        match decode::<serde_json::Value>(&token, &DecodingKey::from_secret(key_str.as_bytes()), &validation) {
            Ok(token_data) => {
                let claims_value = token_data.claims;
                let sub = match &claims_value {
                    serde_json::Value::Object(map) => match map.get("sub") {
                        Some(serde_json::Value::String(s)) => Some(s.clone()),
                        _ => None,
                    },
                    _ => None,
                };
                let exp = match &claims_value {
                    serde_json::Value::Object(map) => match map.get("exp") {
                        Some(serde_json::Value::Number(n)) => n.as_i64(),
                        _ => None,
                    },
                    _ => None,
                };

                let auth_data = crate::AuthenticationData {
                    header: serde_json::Value::Object(serde_json::Map::new()),
                    claims: crate::Claims {
                        all: claims_value,
                        sub,
                        exp,
                    },
                };

                req.extensions_mut().insert(auth_data);
                let svc = self.service.clone();
                Box::pin(async move {
                    svc.call(req).await.map(|res| res.map_into_left_body())
                })
            }
            Err(error) => {
                log::debug!("Could not decode token: {}", error);
                Box::pin(async move {
                    Ok(req.into_response(
                        actix_web::HttpResponse::Unauthorized().finish(),
                    ).map_into_right_body())
                })
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
