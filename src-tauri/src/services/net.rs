use futures_util::StreamExt;
use reqwest::header::ACCEPT;
use reqwest::{redirect, Client, Response, Url};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;

pub const MAX_IMAGE_DOWNLOAD_BYTES: usize = 30 * 1024 * 1024;
pub const MAX_HTML_DOWNLOAD_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_UPDATE_DOWNLOAD_BYTES: u64 = 300 * 1024 * 1024;
pub const STICKUP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) StickUp/1.0";
const MAX_REDIRECTS: usize = 5;

/// Formatea un digest binario como cadena hexadecimal en minúsculas.
pub fn hex_encode_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}

/// Normaliza un checksum SHA-256 proveniente de cabeceras ETag/X-Linked-Etag o API de GitHub.
pub fn normalize_sha256_hex(raw: &str) -> Option<String> {
    let trimmed = raw
        .trim()
        .trim_matches('"')
        .trim();
    let without_prefix = trimmed
        .strip_prefix("sha256:")
        .or_else(|| trimmed.strip_prefix("SHA256:"))
        .unwrap_or(trimmed);
    if without_prefix.len() == 64 && without_prefix.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(without_prefix.to_ascii_lowercase())
    } else {
        None
    }
}

/// Extrae el checksum SHA-256 esperado para un archivo desde el contenido de un archivo SHA256SUMS.txt.
pub fn parse_sha256_sums_for_asset(sums_text: &str, filename: &str) -> Option<String> {
    let target = filename.trim();
    if target.is_empty() {
        return None;
    }

    for line in sums_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let hash_part = parts.next()?;
        let file_part = parts.next()?.trim().trim_start_matches('*');
        let normalized_hash = match normalize_sha256_hex(hash_part) {
            Some(h) => h,
            None => continue,
        };
        let basename = file_part
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(file_part);
        if basename == target {
            return Some(normalized_hash);
        }
    }
    None
}

fn is_forbidden_v4(v4: Ipv4Addr) -> bool {
    let octets = v4.octets();
    let is_cgnat = octets[0] == 100 && (octets[1] & 0b1100_0000) == 64;
    let is_ietf_prot = octets[0] == 192 && octets[1] == 0 && octets[2] == 0;
    let is_benchmark = octets[0] == 198 && (octets[1] == 18 || octets[1] == 19);

    v4.is_loopback()
        || v4.is_private()
        || v4.is_link_local()
        || v4.is_unspecified()
        || v4.is_broadcast()
        || v4.is_multicast()
        || is_cgnat
        || is_ietf_prot
        || is_benchmark
}

fn is_forbidden_v6(v6: Ipv6Addr) -> bool {
    if let Some(mapped) = v6.to_ipv4_mapped() {
        return is_forbidden_v4(mapped);
    }
    let segs = v6.segments();
    let is_link_local = (segs[0] & 0xffc0) == 0xfe80;
    let is_ula = (segs[0] & 0xfe00) == 0xfc00;
    let is_ipv4_compat = segs[0..6] == [0; 6] && !v6.is_loopback() && !v6.is_unspecified();

    v6.is_loopback()
        || v6.is_unspecified()
        || v6.is_multicast()
        || is_link_local
        || is_ula
        || is_ipv4_compat
}

/// Determina si una dirección IP apunta a loopback, red privada, link-local, metadatos o rangos reservados.
pub fn is_forbidden_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_forbidden_v4(v4),
        IpAddr::V6(v6) => is_forbidden_v6(v6),
    }
}

/// Valida sintácticamente una URL HTTP/HTTPS externa antes de abrirla en el navegador del usuario.
pub fn validate_external_open_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("La URL contiene caracteres inválidos o espacios".to_string());
    }
    let parsed = Url::parse(trimmed).map_err(|e| format!("URL inválida: {}", e))?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("Solo se permiten URLs http o https".to_string()),
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("La URL no puede contener credenciales incrustadas".to_string());
    }
    if parsed.host_str().is_none() {
        return Err("La URL no tiene un host válido".to_string());
    }
    Ok(parsed.to_string())
}

/// Valida sintácticamente una URL pública HTTP/HTTPS para descargas iniciadas por la aplicación (anti-SSRF).
pub fn validate_public_http_url_syntax(raw: &str) -> Result<Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.chars().any(|c| c.is_control()) {
        return Err("La URL está vacía o contiene caracteres de control".to_string());
    }
    let parsed = Url::parse(trimmed).map_err(|e| format!("URL inválida: {}", e))?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("Solo se permiten esquemas http y https".to_string()),
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("No se permiten credenciales en la URL".to_string());
    }

    let host_raw = parsed
        .host_str()
        .ok_or_else(|| "La URL carece de host".to_string())?;
    let host_unbracketed = host_raw
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(host_raw);

    if let Ok(ip) = host_unbracketed.parse::<IpAddr>() {
        if is_forbidden_ip(ip) {
            return Err("Acceso bloqueado a dirección IP privada o reservada".to_string());
        }
    } else {
        let lower = host_raw.trim_end_matches('.').to_ascii_lowercase();
        if lower.is_empty()
            || lower == "localhost"
            || lower.ends_with(".localhost")
            || lower.ends_with(".local")
            || lower.ends_with(".internal")
        {
            return Err("Acceso bloqueado a host local o interno".to_string());
        }
    }

    Ok(parsed)
}

/// Valida la sintaxis y resuelve DNS de la URL verificando que ninguna IP resuelta sea privada/reservada.
pub fn resolve_and_validate_public_url(raw: &str) -> Result<(Url, Option<(String, Vec<SocketAddr>)>), String> {
    let parsed = validate_public_http_url_syntax(raw)?;
    let host_raw = parsed
        .host_str()
        .ok_or_else(|| "La URL carece de host".to_string())?;
    let host_unbracketed = host_raw
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(host_raw);

    if host_unbracketed.parse::<IpAddr>().is_err() {
        let port = parsed
            .port_or_known_default()
            .ok_or_else(|| "Puerto desconocido en la URL".to_string())?;
        let addrs: Vec<SocketAddr> = (host_raw, port)
            .to_socket_addrs()
            .map_err(|e| format!("No se pudo resolver el host '{}': {}", host_raw, e))?
            .collect();
        if addrs.is_empty() {
            return Err(format!("El host '{}' no resolvió ninguna dirección IP", host_raw));
        }
        for addr in &addrs {
            if is_forbidden_ip(addr.ip()) {
                return Err(format!(
                    "Acceso bloqueado: '{}' resuelve a una IP no pública",
                    host_raw
                ));
            }
        }
        let host_owned = host_raw.to_string();
        Ok((parsed, Some((host_owned, addrs))))
    } else {
        Ok((parsed, None))
    }
}

/// Extrae un hash SHA-256 válido desde cabeceras HTTP `X-Linked-Etag` o `ETag`.
pub fn extract_header_sha256(headers: &reqwest::header::HeaderMap) -> Option<String> {
    for name in ["x-linked-etag", "etag"] {
        if let Some(val) = headers.get(name).and_then(|v| v.to_str().ok()) {
            if let Some(hex) = normalize_sha256_hex(val) {
                return Some(hex);
            }
        }
    }
    None
}

/// Valida que una URL de descarga de actualización pertenezca al repositorio oficial de GitHub por HTTPS.
pub fn validate_github_release_asset_url(raw: &str) -> Result<Url, String> {
    let parsed = validate_public_http_url_syntax(raw)?;
    if parsed.scheme() != "https" {
        return Err("Las descargas de actualización requieren HTTPS".to_string());
    }
    if parsed.host_str() != Some("github.com") {
        return Err("La descarga de actualización debe provenir de github.com".to_string());
    }
    if !parsed
        .path()
        .starts_with("/FrovaHappy/stickup/releases/download/")
    {
        return Err("La URL no corresponde a un release oficial de StickUp".to_string());
    }
    Ok(parsed)
}

/// Ejecuta una petición HTTP GET validando SSRF, esquema opcional HTTPS estricto y capturando ETag SHA-256.
pub async fn send_get_tracked(
    url: &str,
    timeout_secs: u64,
    accept_header: Option<&str>,
    https_only: bool,
) -> Result<(Response, Option<String>), String> {
    let mut current_url = url.to_string();
    let mut header_sha256: Option<String> = None;

    for _hop in 0..=MAX_REDIRECTS {
        let (parsed, resolved) = resolve_and_validate_public_url(&current_url)?;
        if https_only && parsed.scheme() != "https" {
            return Err("Se requiere conexión HTTPS segura en toda la cadena de descarga".to_string());
        }

        let mut builder = Client::builder()
            .redirect(redirect::Policy::none())
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(STICKUP_USER_AGENT);

        if let Some((ref domain, ref addrs)) = resolved {
            builder = builder.resolve_to_addrs(domain, addrs);
        }

        let client = builder
            .build()
            .map_err(|e| format!("Error inicializando cliente HTTP: {}", e))?;

        let mut req = client.get(parsed.clone());
        if let Some(accept) = accept_header {
            req = req.header(ACCEPT, accept);
        }

        let res = req
            .send()
            .await
            .map_err(|e| format!("Error de red descargando '{}': {}", current_url, e))?;

        if header_sha256.is_none() {
            header_sha256 = extract_header_sha256(res.headers());
        }

        if res.status().is_redirection() {
            let location = res
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| "Redirección HTTP sin cabecera Location".to_string())?;
            let next_url = parsed
                .join(location)
                .map_err(|e| format!("URL de redirección inválida: {}", e))?;
            current_url = next_url.to_string();
            continue;
        }

        return Ok((res, header_sha256));
    }

    Err("Demasiadas redirecciones HTTP".to_string())
}

/// Descarga en streaming hacia un archivo temporal verificando límite de tamaño y checksum SHA-256 opcional.
pub async fn stream_response_to_file_verified<F>(
    response: Response,
    tmp_path: &Path,
    dest_path: &Path,
    min_bytes: u64,
    max_bytes: u64,
    expected_sha256: Option<&str>,
    mut on_progress: F,
) -> Result<String, String>
where
    F: FnMut(u64, Option<u64>),
{
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "El servidor devolvió un error HTTP ({})",
            status.as_u16()
        ));
    }

    let total_bytes = response.content_length();
    if let Some(total) = total_bytes {
        if total > max_bytes {
            return Err(format!(
                "El archivo remoto ({} bytes) excede el máximo permitido ({} bytes)",
                total, max_bytes
            ));
        }
    }

    if tmp_path.exists() {
        let _ = fs::remove_file(tmp_path);
    }

    let mut hasher = Sha256::new();
    let mut downloaded_bytes: u64 = 0;
    let mut stream = response.bytes_stream();

    let stream_res: Result<(), String> = async {
        let mut file = File::create(tmp_path)
            .map_err(|e| format!("No se pudo crear el archivo temporal de descarga: {}", e))?;

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res
                .map_err(|e| format!("Error durante la descarga de datos: {}", e))?;

            let next_total = downloaded_bytes.saturating_add(chunk.len() as u64);
            if next_total > max_bytes {
                return Err(format!(
                    "La descarga superó el tamaño máximo permitido ({} bytes)",
                    max_bytes
                ));
            }

            file.write_all(&chunk)
                .map_err(|e| format!("Error escribiendo descarga en disco: {}", e))?;

            hasher.update(&chunk);
            downloaded_bytes = next_total;
            on_progress(downloaded_bytes, total_bytes);
        }

        file.flush()
            .map_err(|e| format!("Error sincronizando archivo en disco: {}", e))?;
        Ok(())
    }
    .await;

    if let Err(err) = stream_res {
        let _ = fs::remove_file(tmp_path);
        return Err(err);
    }

    if downloaded_bytes < min_bytes {
        let _ = fs::remove_file(tmp_path);
        return Err(format!(
            "El archivo descargado está incompleto o corrupto (tamaño: {} bytes).",
            downloaded_bytes
        ));
    }

    let computed_sha256 = hex_encode_lower(&hasher.finalize());
    if let Some(expected) = expected_sha256 {
        let norm_expected = normalize_sha256_hex(expected).ok_or_else(|| {
            let _ = fs::remove_file(tmp_path);
            "Formato de suma SHA-256 esperada inválido".to_string()
        })?;
        if computed_sha256 != norm_expected {
            let _ = fs::remove_file(tmp_path);
            return Err(format!(
                "Verificación SHA-256 fallida: esperado {}, obtenido {}",
                norm_expected, computed_sha256
            ));
        }
    }

    if dest_path.exists() {
        let _ = fs::remove_file(dest_path);
    }
    fs::rename(tmp_path, dest_path).map_err(|e| {
        let _ = fs::remove_file(tmp_path);
        format!("Error finalizando archivo descargado: {}", e)
    })?;

    Ok(computed_sha256)
}

/// Ejecuta una petición HTTP GET validando SSRF y fijando IPs resueltas en cada salto de redirección.
pub async fn send_public_get(
    url: &str,
    timeout_secs: u64,
    accept_header: Option<&str>,
) -> Result<Response, String> {
    let (res, _) = send_get_tracked(url, timeout_secs, accept_header, false).await?;
    Ok(res)
}

/// Descarga el cuerpo de una respuesta HTTP en streaming aplicando un límite máximo estricto de bytes.
pub async fn read_response_bytes_limited(
    response: Response,
    max_bytes: usize,
) -> Result<Vec<u8>, String> {
    if !response.status().is_success() {
        return Err(format!("Error HTTP {}", response.status()));
    }

    if let Some(len) = response.content_length() {
        if len > max_bytes as u64 {
            return Err(format!(
                "El recurso remoto ({} bytes) excede el límite permitido ({} bytes)",
                len, max_bytes
            ));
        }
    }

    let initial_cap = response
        .content_length()
        .map(|l| (l as usize).min(max_bytes))
        .unwrap_or(0);
    let mut out = Vec::with_capacity(initial_cap);
    let mut stream = response.bytes_stream();

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.map_err(|e| format!("Error leyendo flujo HTTP: {}", e))?;
        if out.len().saturating_add(chunk.len()) > max_bytes {
            return Err(format!(
                "La descarga superó el tamaño máximo permitido de {} bytes",
                max_bytes
            ));
        }
        out.extend_from_slice(&chunk);
    }

    Ok(out)
}

/// Descarga bytes desde una URL pública validando SSRF y aplicando límite máximo en memoria.
pub async fn fetch_bytes_limited(
    url: &str,
    max_bytes: usize,
    timeout_secs: u64,
    accept_header: Option<&str>,
) -> Result<Vec<u8>, String> {
    let response = send_public_get(url, timeout_secs, accept_header).await?;
    read_response_bytes_limited(response, max_bytes).await
}

/// Descarga texto (ej. HTML) desde una URL pública validando SSRF y límite máximo en memoria.
pub async fn fetch_text_limited(
    url: &str,
    max_bytes: usize,
    timeout_secs: u64,
) -> Result<String, String> {
    let response = send_public_get(url, timeout_secs, Some("text/html,application/xhtml+xml,*/*")).await?;
    let bytes = read_response_bytes_limited(response, max_bytes).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_is_forbidden_ip_blocks_private_and_metadata_ranges() {
        let forbidden: Vec<IpAddr> = vec![
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(172, 16, 5, 4)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)),
            IpAddr::V4(Ipv4Addr::new(100, 64, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            IpAddr::V6("fe80::1".parse().unwrap()),
            IpAddr::V6("fc00::1".parse().unwrap()),
            IpAddr::V6("::ffff:127.0.0.1".parse().unwrap()),
            IpAddr::V6("::ffff:169.254.169.254".parse().unwrap()),
        ];

        for ip in forbidden {
            assert!(is_forbidden_ip(ip), "Expected forbidden IP: {}", ip);
        }

        let allowed: Vec<IpAddr> = vec![
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
            IpAddr::V6("2606:4700:4700::1111".parse().unwrap()),
        ];

        for ip in allowed {
            assert!(!is_forbidden_ip(ip), "Expected public IP: {}", ip);
        }
    }

    #[test]
    fn test_validate_public_http_url_syntax_blocks_ssrf_targets() {
        let blocked = [
            "",
            "javascript:alert(1)",
            "file:///C:/Windows/win.ini",
            "ftp://example.com/file.png",
            "http://user:pass@example.com/a.png",
            "http://localhost:8080/a.png",
            "http://sub.localhost/a.png",
            "http://metadata.google.internal/computeMetadata/v1/",
            "http://127.0.0.1/admin",
            "http://169.254.169.254/latest/meta-data/",
            "https://[::1]/admin",
            "https://example.com/\r\nHost: evil",
        ];

        for raw in blocked {
            assert!(
                validate_public_http_url_syntax(raw).is_err(),
                "Expected URL to be blocked: {}",
                raw
            );
        }

        assert!(validate_public_http_url_syntax("https://cdn.example.com/emote.png").is_ok());
        assert!(validate_public_http_url_syntax("http://example.com/image.webp").is_ok());
    }

    #[test]
    fn test_validate_external_open_url() {
        assert!(validate_external_open_url("https://github.com/FrovaHappy/stickup").is_ok());
        assert!(validate_external_open_url("http://example.com/path?a=1&b=2").is_ok());
        assert!(validate_external_open_url("javascript:alert(1)").is_err());
        assert!(validate_external_open_url("file:///C:/Windows/System32/cmd.exe").is_err());
        assert!(validate_external_open_url("https://example.com/a b").is_err());
        assert!(validate_external_open_url("https://user:pass@example.com/").is_err());
    }

    #[test]
    fn test_normalize_sha256_hex_and_sums_parsing() {
        let expected = "a".repeat(64);
        assert_eq!(
            normalize_sha256_hex(&format!("sha256:{}", expected.to_uppercase())),
            Some(expected.clone())
        );
        assert_eq!(
            normalize_sha256_hex(&format!("\"{}\"", expected)),
            Some(expected.clone())
        );
        assert_eq!(normalize_sha256_hex("deadbeef"), None);

        let sums = format!(
            "# comment\n{}  ./release-assets/StickUp_1.0.0_x64-setup.exe\n{} *other.tar.gz\n",
            expected,
            "b".repeat(64)
        );
        assert_eq!(
            parse_sha256_sums_for_asset(&sums, "StickUp_1.0.0_x64-setup.exe"),
            Some(expected)
        );
        assert_eq!(
            parse_sha256_sums_for_asset(&sums, "missing.exe"),
            None
        );
    }

    #[test]
    fn test_validate_github_release_asset_url() {
        assert!(validate_github_release_asset_url(
            "https://github.com/FrovaHappy/stickup/releases/download/v0.1.0/StickUp_0.1.0_x64-setup.exe"
        )
        .is_ok());
        assert!(validate_github_release_asset_url(
            "http://github.com/FrovaHappy/stickup/releases/download/v0.1.0/StickUp.exe"
        )
        .is_err());
        assert!(validate_github_release_asset_url(
            "https://evil.com/FrovaHappy/stickup/releases/download/v0.1.0/StickUp.exe"
        )
        .is_err());
        assert!(validate_github_release_asset_url(
            "https://github.com/otheruser/stickup/releases/download/v0.1.0/StickUp.exe"
        )
        .is_err());
    }

    #[test]
    fn test_extract_header_sha256() {
        let mut headers = reqwest::header::HeaderMap::new();
        let hash = "c".repeat(64);
        headers.insert("x-linked-etag", format!("\"{}\"", hash).parse().unwrap());
        assert_eq!(extract_header_sha256(&headers), Some(hash));

        let mut weak_headers = reqwest::header::HeaderMap::new();
        weak_headers.insert("etag", "\"not-a-sha256\"".parse().unwrap());
        assert_eq!(extract_header_sha256(&weak_headers), None);
    }
}
