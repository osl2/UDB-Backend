use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error,
};
use futures::{
    future::{ok, Either, FutureResult},
    Poll,
};

pub struct UploadFilter {
    pub filter: bool,
}

impl<S, B> Transform<S> for UploadFilter
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UploadFilterMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UploadFilterMiddleware {
            service,
            filter: self.filter,
        })
    }
}

pub struct UploadFilterMiddleware<S> {
    service: S,
    filter: bool,
}

impl<S, B> Service for UploadFilterMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = futures::future::Either<FutureResult<Self::Response, Self::Error>, S::Future>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if self.filter {
            match req.method().as_str() {
                "PUT" | "POST" => Either::A(ok(req.into_response(
                    actix_web::HttpResponse::build(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS)
                        .finish()
                        .into_body(),
                ))),
                _ => Either::B(self.service.call(req)),
            }
        } else {
            Either::B(self.service.call(req))
        }
    }
}
