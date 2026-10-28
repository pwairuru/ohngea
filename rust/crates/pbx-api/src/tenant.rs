use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn require_tenant(
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !auth.starts_with("Bearer ") || auth.len() < 20 {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // Real impl: decode JWT, stash domain_id in extensions, run SET LOCAL in db layer (Task 4 wires pool)
    Ok(next.run(req).await)
}
