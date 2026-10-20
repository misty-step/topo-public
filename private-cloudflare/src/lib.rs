use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use worker::{
    js_sys::{self, Array, Object},
    wasm_bindgen::{JsCast, JsValue},
    wasm_bindgen_futures::JsFuture,
    *,
};

const CSP: &str = "default-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'none'; frame-src 'none'; frame-ancestors 'none'; base-uri 'none'; object-src 'none'; form-action 'none'";

#[derive(Deserialize)]
struct JwtHeader {
    alg: String,
    kid: String,
    typ: Option<String>,
    crit: Option<serde_json::Value>,
    b64: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

impl Audience {
    fn includes(&self, expected: &str) -> bool {
        match self {
            Self::One(value) => value == expected,
            Self::Many(values) => values.iter().any(|value| value == expected),
        }
    }
}

#[derive(Deserialize)]
struct Claims {
    iss: String,
    aud: Audience,
    exp: u64,
    nbf: Option<u64>,
    #[serde(rename = "type")]
    token_type: String,
}

impl Claims {
    fn valid(&self, issuer: &str, audience: &str) -> bool {
        let now = (js_sys::Date::now() / 1000.0).floor() as u64;
        self.iss == issuer
            && self.aud.includes(audience)
            && self.token_type == "app"
            && self.exp > now
            && self.nbf.is_none_or(|not_before| not_before <= now)
    }
}

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize, Serialize)]
struct Jwk {
    kid: String,
    kty: String,
    alg: String,
    #[serde(rename = "use")]
    key_use: String,
    n: String,
    e: String,
}

fn rejected() -> Error {
    Error::RustError("Access verification failed".into())
}

fn decode_json<T: serde::de::DeserializeOwned>(encoded: &str) -> Result<T> {
    let bytes = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| rejected())?;
    serde_json::from_slice(&bytes).map_err(|_| rejected())
}

fn team_domain_valid(issuer: &str) -> bool {
    let Ok(url) = Url::parse(issuer) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let Some(team) = host.strip_suffix(".cloudflareaccess.com") else {
        return false;
    };
    !team.is_empty()
        && !team.starts_with('-')
        && !team.ends_with('-')
        && team
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && issuer == format!("https://{host}")
}

async fn verify_access(req: &Request, env: &Env) -> Result<()> {
    let issuer = env.var("ACCESS_TEAM_DOMAIN")?.to_string();
    let audience = env.var("ACCESS_AUD")?.to_string();
    if !team_domain_valid(&issuer) || audience.is_empty() || audience.trim() != audience {
        return Err(rejected());
    }
    // Only the application assertion is accepted. Service Auth policies produce
    // the same signed application JWT, without requiring email or a nonempty sub.
    let token = req
        .headers()
        .get("Cf-Access-Jwt-Assertion")?
        .ok_or_else(rejected)?;
    if token.len() > 32_768 {
        return Err(rejected());
    }
    let mut parts = token.split('.');
    let encoded_header = parts.next().ok_or_else(rejected)?;
    let encoded_claims = parts.next().ok_or_else(rejected)?;
    let encoded_signature = parts.next().ok_or_else(rejected)?;
    if parts.next().is_some() {
        return Err(rejected());
    }
    let header: JwtHeader = decode_json(encoded_header)?;
    // No algorithm negotiation, critical extensions, or unencoded JWS payloads.
    if header.alg != "RS256"
        || header.kid.is_empty()
        || header.typ.as_deref().is_some_and(|typ| typ != "JWT")
        || header.crit.is_some()
        || header.b64 == Some(false)
    {
        return Err(rejected());
    }
    let claims: Claims = decode_json(encoded_claims)?;
    if !claims.valid(&issuer, &audience) {
        return Err(rejected());
    }
    let signature = URL_SAFE_NO_PAD
        .decode(encoded_signature)
        .map_err(|_| rejected())?;
    if signature.is_empty() {
        return Err(rejected());
    }

    // Fetch only this configured team's public keys, never a URL from the token.
    // Both active and previous keys are supported, including Access key rotation.
    let certs_url =
        Url::parse(&format!("{issuer}/cdn-cgi/access/certs")).map_err(|_| rejected())?;
    let mut certs = Fetch::Url(certs_url).send().await?;
    if certs.status_code() != 200 {
        return Err(rejected());
    }
    let jwks: Jwks = certs.json().await?;
    let jwk = jwks
        .keys
        .iter()
        .find(|key| {
            key.kid == header.kid && key.kty == "RSA" && key.alg == "RS256" && key.key_use == "sig"
        })
        .ok_or_else(rejected)?;
    let key: Object = js_sys::JSON::parse(&serde_json::to_string(jwk)?)?.dyn_into()?;
    let algorithm = Object::new();
    js_sys::Reflect::set(
        &algorithm,
        &JsValue::from_str("name"),
        &JsValue::from_str("RSASSA-PKCS1-v1_5"),
    )?;
    js_sys::Reflect::set(
        &algorithm,
        &JsValue::from_str("hash"),
        &JsValue::from_str("SHA-256"),
    )?;
    let usages = Array::new();
    usages.push(&JsValue::from_str("verify"));
    let global: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    let subtle = global.crypto()?.subtle();
    let key: web_sys::CryptoKey = JsFuture::from(subtle.import_key_with_object(
        "jwk",
        &key,
        &algorithm,
        false,
        usages.as_ref(),
    )?)
    .await?
    .dyn_into()?;
    // The signed bytes are the original two encoded segments, not parsed JSON.
    let signed_len = encoded_header.len() + 1 + encoded_claims.len();
    let verified = JsFuture::from(subtle.verify_with_object_and_u8_array_and_u8_array(
        &algorithm,
        &key,
        &signature,
        &token.as_bytes()[..signed_len],
    )?)
    .await?;
    if verified.as_bool() != Some(true) || !claims.valid(&issuer, &audience) {
        return Err(rejected());
    }
    Ok(())
}

fn private_headers(mut response: Response) -> Result<Response> {
    // ASSETS response headers may be immutable. Copy metadata, not the body.
    let mutable = Headers::new();
    for (name, value) in response.headers().entries() {
        mutable.set(&name, &value)?;
    }
    response = response.with_headers(mutable);
    let headers = response.headers_mut();
    headers.set("Cache-Control", "private, no-store")?;
    headers.set("CDN-Cache-Control", "no-store")?;
    headers.set("Cloudflare-CDN-Cache-Control", "no-store")?;
    headers.set("Referrer-Policy", "no-referrer")?;
    headers.set("X-Content-Type-Options", "nosniff")?;
    headers.set("Content-Security-Policy", CSP)?;
    headers.set(
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=()",
    )?;
    Ok(response)
}

fn status_response(method: Method, status: u16, message: &str) -> Result<Response> {
    if method == Method::Head {
        Ok(Response::empty()?.with_status(status))
    } else {
        Response::error(message, status)
    }
}

async fn serve(req: Request, env: Env) -> Result<Response> {
    let method = req.method();
    let mut url = req.url()?;
    // Defense in depth against alternate routes, workers.dev, and preview URLs.
    let host = env.var("PRIVATE_HOST")?.to_string();
    if host.is_empty()
        || url.scheme() != "https"
        || url.host_str() != Some(host.as_str())
        || url.port().is_some()
    {
        return status_response(method, 403, "Forbidden");
    }
    if !matches!(method, Method::Get | Method::Head) {
        let mut response = status_response(method, 405, "Only GET and HEAD are supported")?;
        response.headers_mut().set("Allow", "GET, HEAD")?;
        return Ok(response);
    }
    // No redirect, asset lookup, or private response body precedes verification.
    if verify_access(&req, &env).await.is_err() {
        return status_response(method, 403, "Forbidden");
    }
    match url.path() {
        "/maps" | "/projects" => {
            let location = format!("{}/", url.path());
            let mut response = Response::empty()?.with_status(307);
            response.headers_mut().set("Location", &location)?;
            return Ok(response);
        }
        "/" | "/maps/" | "/projects/" => {
            let path = format!("{}index.html", url.path());
            url.set_path(&path);
            let mut init = RequestInit::new();
            init.with_method(method).with_headers(req.headers().clone());
            let rewritten = Request::new_with_init(url.as_str(), &init)?;
            env.assets("ASSETS")?.fetch_request(rewritten).await
        }
        _ => env.assets("ASSETS")?.fetch_request(req).await,
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let method = req.method();
    let response = match serve(req, env).await {
        Ok(response) => response,
        Err(_) => status_response(method, 503, "Private site unavailable")?,
    };
    // Apply protection equally to successful assets, redirects, denials, and errors.
    private_headers(response)
}
