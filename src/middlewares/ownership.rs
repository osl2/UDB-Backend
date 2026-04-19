use crate::schema;
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use diesel::{
    r2d2::{self, ConnectionManager},
    ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use futures_util::future::LocalBoxFuture;
use regex::Regex;
use std::{cell::RefCell, future::{ready, Ready}, rc::Rc};

pub struct OwnershipChecker {}

impl<S, B> Transform<S, ServiceRequest> for OwnershipChecker
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = OwnershipCheckerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OwnershipCheckerMiddleware { service: Rc::new(service) }))
    }
}

pub struct OwnershipCheckerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for OwnershipCheckerMiddleware<S>
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
        lazy_static::lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<uuid>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})$").unwrap();
        }

        let id = match RE.captures(req.path()) {
            Some(captures) => match captures.get(1) {
                Some(m) => Some(m.as_str().to_string()),
                None => None,
            },
            None => {
                let svc = self.service.clone();
                return Box::pin(async move {
                    svc.call(req).await.map(|res| res.map_into_left_body())
                });
            }
        };

        let result = {
            let extensions = req.extensions();
            let conn_cell =
                extensions.get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>();
            let token = extensions.get::<actix_web_jwt_middleware::AuthenticationData>();

            match req.method().as_str() {
                "PUT" | "DELETE" => {
                    match (conn_cell, token, id) {
                        (Some(conn_cell), Some(token), Some(id)) => {
                            let mut conn = conn_cell.borrow_mut();
                            schema::access::table
                                .filter(schema::access::object_id.eq(id))
                                .filter(
                                    schema::access::user_id.eq(token.claims.sub.clone().unwrap()),
                                )
                                .get_result::<(String, String)>(&mut *conn)
                                .map_err(|e| match e {
                                    diesel::result::Error::NotFound => {
                                        OwnershipCheckerError::NoAccess
                                    }
                                    e => {
                                        log::error!("Couldn't query object access: {}", e);
                                        OwnershipCheckerError::Undefined
                                    }
                                })
                                .map(|_| ())
                        }
                        _ => Err(OwnershipCheckerError::Undefined),
                    }
                }
                _ => Ok(()),
            }
        };

        let svc = self.service.clone();
        match result {
            Ok(_) => Box::pin(async move {
                svc.call(req).await.map(|res| res.map_into_left_body())
            }),
            Err(_) => Box::pin(async move {
                Ok(req.into_response(
                    actix_web::HttpResponse::Forbidden().finish(),
                ).map_into_right_body())
            }),
        }
    }
}

enum OwnershipCheckerError {
    Undefined,
    NoAccess,
}
