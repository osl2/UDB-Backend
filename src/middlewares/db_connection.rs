use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::{
    future::{ok, Either, FutureResult},
    Poll,
};

pub struct DatabaseConnection {
    pub database: crate::database::Database,
}

impl<S, B> Transform<S> for DatabaseConnection
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = DatabaseConnectionMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DatabaseConnectionMiddleware {
            service,
            database: self.database.clone(),
        })
    }
}

pub struct DatabaseConnectionMiddleware<S> {
    service: S,
    database: crate::database::Database,
}

impl<S, B> Service for DatabaseConnectionMiddleware<S>
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
        let mut i = 0;
        loop {
            i += 1;
            match self.database.get_connection() {
                Ok(pooled_conn) => {
                    req.extensions_mut().insert(pooled_conn);
                    return Either::A(self.service.call(req));
                }
                Err(e) => {
                    log::error!(
                        "Database connection is broken{}: {}",
                        if i >= 5 {
                            ", returning an error"
                        } else {
                            ", trying again"
                        },
                        e
                    );
                }
            }
            if i >= 5 {
                return Either::B(ok(req.into_response(
                    actix_web::HttpResponse::InternalServerError()
                        .finish()
                        .into_body(),
                )));
            }
        }
    }
}
