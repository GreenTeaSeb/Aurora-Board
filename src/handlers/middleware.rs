use actix_identity::RequestIdentity;
use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpResponse};
use futures::future::{ok, Either, Ready};

pub struct LoginAuth;

impl<S> Transform<S, ServiceRequest> for LoginAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = LoginAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoginAuthMiddleware { service })
    }
}

pub struct LoginAuthMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for LoginAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;
    fn poll_ready(&self, ctx: &mut std::task::Context) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let id = req.get_identity();
        let path = get_path(&req);
        let method = req.method().as_str().to_owned();

        if id.is_some() {
            let user_id_int: u32 = id.unwrap_or_default().parse().unwrap_or_default();
            if user_id_int > 0 {
                req.extensions_mut().insert(user_id_int);
                return Either::Left(self.service.call(req));
            }
        };

        Either::Right(ok(req.into_response(
            HttpResponse::Found()
                .append_header((
                    "location",
                    format!("/login?redirect={}&method={}", path, method),
                ))
                .body("Not logged in"),
        )))
    }
}

fn get_path(req: &ServiceRequest) -> String {
    req.path().to_string()
}
