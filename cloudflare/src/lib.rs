use worker::*;

// This Worker delivers public product assets only. There is no map upload,
// customer database, provider secret, inference binding or mutation endpoint.
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if !matches!(req.method(), Method::Get | Method::Head) {
        let mut response = Response::error(
            "This site serves public docs and downloads only; private maps stay local.",
            405,
        )?;
        response.headers_mut().set("Allow", "GET, HEAD")?;
        return Ok(response);
    }
    let path = req.path();
    let mut response = env.assets("ASSETS")?.fetch_request(req).await?;
    let mutable = Headers::new();
    for (name, value) in response.headers().entries() {
        mutable.set(&name, &value)?;
    }
    response = response.with_headers(mutable);
    let headers = response.headers_mut();
    headers.set("X-Content-Type-Options", "nosniff")?;
    headers.set("Referrer-Policy", "no-referrer")?;
    headers.set(
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=()",
    )?;
    headers.set("Content-Security-Policy","default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; frame-src 'self'; frame-ancestors 'self'; base-uri 'self'; object-src 'none'; form-action 'none'")?;
    if path.starts_with("/releases/v") {
        headers.set("Cache-Control", "public, max-age=31536000, immutable")?;
        if !path.ends_with(".sha256") {
            headers.set("Content-Type", "application/octet-stream")?;
        }
    } else if path == "/install.sh" {
        headers.set("Content-Type", "text/plain; charset=utf-8")?;
        headers.set("Cache-Control", "no-cache")?;
    }
    Ok(response)
}
