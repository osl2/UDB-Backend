use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{future::{ready, Ready}, rc::Rc};

pub struct UploadFilter {
    pub filter: bool,
}

impl<S, B> Transform<S, ServiceRequest> for UploadFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = UploadFilterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UploadFilterMiddleware {
            service: Rc::new(service),
            filter: self.filter,
        }))
    }
}

pub struct UploadFilterMiddleware<S> {
    service: Rc<S>,
    filter: bool,
}

impl<S, B> Service<ServiceRequest> for UploadFilterMiddleware<S>
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
        if self.filter {
            match req.method().as_str() {
                "PUT" | "POST" => {
                    return Box::pin(async move {
                        Ok(req.into_response(
                            actix_web::HttpResponse::build(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS)
                                .finish(),
                        ).map_into_right_body())
                    });
                }
                _ => {}
            }
        }
        let svc = self.service.clone();
        Box::pin(async move {
            svc.call(req).await.map(|res| res.map_into_left_body())
        })
    }
}
