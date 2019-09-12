use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::{
    future::{ok, FutureResult},
    Poll,
};

pub struct AuthToUuid {}

impl<S, B> Transform<S> for AuthToUuid
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthToUuidMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthToUuidMiddleware { service })
    }
}

pub struct AuthToUuidMiddleware<S> {
    service: S,
}

impl<S, B> Service for AuthToUuidMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let uuid_result = match dbg!(req
            .extensions()
            .get::<actix_web_jwt_middleware::AuthenticationData>())
        {
            Some(auth_data) => Some(
                uuid::Uuid::parse_str(&&(*auth_data).clone().claims.sub.clone().unwrap()).unwrap(),
            ),
            None => None,
        };

        match dbg!(uuid_result) {
            Some(uuid) => {
                req.extensions_mut().insert(uuid);
                self.service.call(req)
            }
            None => self.service.call(req),
        }
    }
}
