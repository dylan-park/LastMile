use axum::{
    extract::FromRequestParts,
    http::{Request, StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::Cookie;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SessionId(pub String);

pub async fn session_middleware(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract session ID from cookie or generate new one
    let mut session_id = None;

    if let Some(cookie_header) = req.headers().get(axum::http::header::COOKIE)
        && let Ok(cookie_str) = cookie_header.to_str()
    {
        for cookie in cookie_str.split(';') {
            let parts: Vec<&str> = cookie.splitn(2, '=').collect();
            if parts.len() == 2 && parts[0].trim() == "lastmile_session" {
                session_id = Some(parts[1].trim().to_string());
                break;
            }
        }
    }

    let (final_session_id, is_new) = if let Some(id) = session_id {
        (id, false)
    } else {
        (Uuid::new_v4().to_string(), true)
    };

    // Insert SessionId into request extensions for handlers
    req.extensions_mut()
        .insert(SessionId(final_session_id.clone()));
    let mut response = next.run(req).await;

    // Set the Set-Cookie header if it's a new session
    if is_new {
        let cookie = Cookie::build(("lastmile_session", final_session_id))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .build();

        if let Ok(value) = cookie.to_string().parse() {
            response
                .headers_mut()
                .append(axum::http::header::SET_COOKIE, value);
        }
    }

    Ok(response)
}

// Async trait for extractor
impl<S> FromRequestParts<S> for SessionId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<SessionId>()
            .cloned()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "SessionId missing"))
    }
}
