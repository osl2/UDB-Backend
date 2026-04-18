use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use diesel::r2d2::{self, ConnectionManager};
use futures_util::future::LocalBoxFuture;
use std::{future::{ready, Ready}, rc::Rc};

pub struct DatabaseConnection<C: 'static>
where
    C: diesel::Connection,
{
    pub pool: r2d2::Pool<ConnectionManager<C>>,
}

impl<S, B, C> Transform<S, ServiceRequest> for DatabaseConnection<C>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    C: diesel::Connection + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = DatabaseConnectionMiddleware<C, S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DatabaseConnectionMiddleware {
            service: Rc::new(service),
            pool: self.pool.clone(),
        }))
    }
}

pub struct DatabaseConnectionMiddleware<C: 'static, S>
where
    C: diesel::Connection,
{
    service: Rc<S>,
    pool: r2d2::Pool<ConnectionManager<C>>,
}

impl<S, B, C> Service<ServiceRequest> for DatabaseConnectionMiddleware<C, S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    C: diesel::Connection + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let pool = self.pool.clone();
        Box::pin(async move {
            let mut i = 0;
            loop {
                i += 1;
                match pool.get() {
                    Ok(pooled_conn) => {
                        req.extensions_mut().insert(pooled_conn);
                        return svc.call(req).await.map(|res| res.map_into_left_body());
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
                    return Ok(req.into_response(
                        actix_web::HttpResponse::InternalServerError().finish(),
                    ).map_into_right_body());
                }
            }
        })
    }
}
