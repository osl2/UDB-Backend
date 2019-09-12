use crate::schema;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use futures::{
    future::{ok, Either, FutureResult},
    Poll,
};
use regex::Regex;

pub struct OwnershipChecker {}

impl<S, B> Transform<S> for OwnershipChecker
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = OwnershipCheckerMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(OwnershipCheckerMiddleware { service })
    }
}

pub struct OwnershipCheckerMiddleware<S> {
    service: S,
}

impl<S, B> Service for OwnershipCheckerMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = futures::future::Either<S::Future, FutureResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        lazy_static::lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<uuid>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})$").unwrap();
        }
        let id = match RE.captures(req.path()) {
            Some(captures) => captures.get(1),
            None => return Either::A(self.service.call(req)),
        };
        let result = {
            let extensions = req.extensions();
            let conn = req
                .app_data::<crate::AppData>()
                .unwrap()
                .database
                .get_connection()
                .unwrap();
            let token = extensions.get::<uuid::Uuid>();

            match req.method().as_str() {
                "PUT" | "DELETE" => {
                    match (token, id) {
                        (Some(token), Some(id)) => {
                            // Check whether the user has access to the object
                            schema::access::table
                                .filter(schema::access::object_id.eq(id.as_str().to_string()))
                                .filter(schema::access::user_id.eq(token.to_string()))
                                .get_result::<(String, String)>(&*conn)
                                .map_err(|e| match e {
                                    diesel::result::Error::NotFound => {
                                        OwnershipCheckerError::NoAccess
                                    }
                                    e => {
                                        log::error!("Couldn't query object access: {:?}", e);
                                        OwnershipCheckerError::Undefined
                                    }
                                })
                                .map(|_| ())
                        }
                        error => {
                            log::error!("Couldn't query object access: {:?}", error);
                            Err(OwnershipCheckerError::Undefined)
                        }
                    }
                }
                _ => Ok(()),
            }
        };

        match result {
            Ok(_) => Either::A(self.service.call(req)),
            Err(error) => Either::B(ok(req.into_response(
                actix_web::HttpResponse::Forbidden()
                    .body(format!("No access to resource: {:?}", error))
                    .into_body(),
            ))),
        }
    }
}
#[derive(Debug)]
enum OwnershipCheckerError {
    Undefined,
    NoAccess,
}
