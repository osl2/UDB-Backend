use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use diesel::r2d2::{self, ConnectionManager};
use futures::{
    future::{ok, Either, FutureResult},
    Poll,
};

pub struct DatabaseConnection<C: 'static>
where
    C: diesel::Connection,
{
    pub pool: r2d2::Pool<ConnectionManager<C>>,
}

impl<S, B, C> Transform<S> for DatabaseConnection<C>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    C: diesel::Connection,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = DatabaseConnectionMiddleware<C, S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DatabaseConnectionMiddleware {
            service,
            pool: self.pool.clone(),
        })
    }
}

pub struct DatabaseConnectionMiddleware<C: 'static, S>
where
    C: diesel::Connection,
{
    service: S,
    pool: r2d2::Pool<ConnectionManager<C>>,
}

impl<S, B, C> Service for DatabaseConnectionMiddleware<C, S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    C: diesel::Connection,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = futures::future::Either<S::Future, FutureResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        match self.pool.get() {
            Ok(pooled_conn) => {
                req.extensions_mut().insert(pooled_conn);
                Either::A(self.service.call(req))
            }
            Err(e) => {
                log::error!("Database connection is broken: {}", e);
                Either::B(ok(req.into_response(
                    actix_web::HttpResponse::InternalServerError()
                        .finish()
                        .into_body(),
                )))
            }
        }
    }
}
