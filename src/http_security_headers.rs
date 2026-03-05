use axum::body::Body;
use axum::http::{header, HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;

pub(crate) async fn add_security_headers(req: Request<Body>, next: Next) -> Response {
    
    let mut res = next.run(req).await;

    // Baseline hardening headers for all responses (both HTML and JSON)
    // For reference: https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy
    res.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
            base-uri 'self'; \
            object-src 'none'; \
            frame-ancestors 'none'"
        )
    );

    // Blocks a request if either is true:
    // (1) request destination is of type 'style' and the MIME type is not 'text/css'.
    // (2) request destination is of type 'script' and the MIME type is not a JavaScript MIME type.
    // For reference: https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/X-Content-Type-Options
    res.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff")
    );

    // Send the origin, path, and query string when performing a same-origin request.
    // For cross-origin requests send the origin (only) when no change to protocol security level.
    // For reference: https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Referrer-Policy
    res.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin")
    );

    // The general form of the Permissions-Policy header is:
    // Permissions-Policy: <policy-name>=<allowlist>; <policy-name>=<allowlist>; ...
    // '()' is the empty allowlist. Hence, the following header will disallow all camera usage
    // Permissions-Policy: camera=()
    // For reference: https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Permissions-Policy
    res.headers_mut().insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), payment=(), usb=(), bluetooth=()"
            )
    );

    res
}