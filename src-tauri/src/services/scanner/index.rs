use crate::commands::to_pascal_case;
use crate::models::{RawScannedItem, ScannedImage};
use std::collections::HashSet;
use std::path::Path;

pub async fn download_html(url: &str) -> Result<String, String> {
    crate::services::net::fetch_text_limited(
        url.trim(),
        crate::services::net::MAX_HTML_DOWNLOAD_BYTES,
        25,
    )
    .await
}

/// Extrae el origen (esquema + host + puerto opcional) de una URL base
fn extract_origin(base: &str) -> Option<String> {
    if let Some(idx) = base.find("://") {
        let rest = &base[idx + 3..];
        let end_idx = rest.find('/').unwrap_or(rest.len());
        Some(format!("{}{}", &base[..idx + 3], &rest[..end_idx]))
    } else {
        None
    }
}

/// Extrae el directorio base de una URL
fn extract_base_directory(base: &str) -> String {
    let without_query = base.split('?').next().unwrap_or(base);
    let without_fragment = without_query.split('#').next().unwrap_or(without_query);
    if let Some(slash_idx) = without_fragment.rfind('/') {
        if slash_idx > 7 {
            // después de https://
            return without_fragment[..slash_idx + 1].to_string();
        }
    }
    format!("{}/", without_fragment.trim_end_matches('/'))
}

/// Decodifica entidades HTML comunes en cadenas y URLs
pub fn unescape_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&#38;", "&")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn is_safe_raster_data_image(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    lower.starts_with("data:image/png")
        || lower.starts_with("data:image/jpeg")
        || lower.starts_with("data:image/jpg")
        || lower.starts_with("data:image/gif")
        || lower.starts_with("data:image/webp")
}

/// Resuelve una URL candidata relativa contra la URL base
pub fn resolve_url(candidate: &str, base_url: Option<&str>) -> Option<String> {
    let unescaped = unescape_html_entities(candidate);
    let clean = unescaped.trim().trim_matches(|c| c == '\'' || c == '"');
    if clean.is_empty() || clean.starts_with('#') || clean.chars().any(|c| c.is_control()) {
        return None;
    }

    if is_safe_raster_data_image(clean) {
        return Some(clean.to_string());
    }

    let lower = clean.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return Some(clean.to_string());
    }

    // Rechazar cualquier pseudo-esquema explícito (javascript:, vbscript:, data:image/svg+xml, file:, etc.)
    if let Some(colon_idx) = clean.find(':') {
        let before_colon = &clean[..colon_idx];
        if !before_colon.contains('/') && !before_colon.contains('?') && !before_colon.contains('#')
        {
            return None;
        }
    }

    if clean.starts_with("//") {
        let scheme = base_url
            .and_then(|b| b.trim().split("://").next())
            .map(|s| s.to_ascii_lowercase())
            .filter(|s| s == "http" || s == "https")
            .unwrap_or_else(|| "https".to_string());
        return Some(format!("{}:{}", scheme, clean));
    }

    let base = base_url?.trim();
    let base_lower = base.to_ascii_lowercase();
    if base.is_empty() || (!base_lower.starts_with("http://") && !base_lower.starts_with("https://")) {
        return None;
    }

    if clean.starts_with('/') {
        if let Some(origin) = extract_origin(base) {
            return Some(format!("{}{}", origin.trim_end_matches('/'), clean));
        }
    }

    let base_dir = extract_base_directory(base);
    let mut clean_rel = clean;
    while clean_rel.starts_with("./") {
        clean_rel = &clean_rel[2..];
    }

    let mut parts: Vec<&str> = base_dir
        .split('/')
        .filter(|p| !p.is_empty())
        .collect();

    let rel_segments: Vec<&str> = clean_rel.split('/').collect();
    while let Some(&"parent") = rel_segments.first() {
        break;
    }

    let mut rel_iter = rel_segments.into_iter();
    while let Some(segment) = rel_iter.next() {
        if segment == ".." {
            if parts.len() > 2 {
                // No remover el scheme://host
                parts.pop();
            }
        } else if segment == "." || segment.is_empty() {
            continue;
        } else {
            parts.push(segment);
            for remaining in rel_iter {
                if remaining == ".." {
                    if parts.len() > 2 {
                        parts.pop();
                    }
                } else if remaining != "." {
                    parts.push(remaining);
                }
            }
            break;
        }
    }

    if parts.len() >= 2 && parts[0].ends_with(':') {
        let scheme = parts[0];
        let rest = parts[1..].join("/");
        Some(format!("//{}", rest).replacen("//", &format!("{}//", scheme), 1))
    } else {
        Some(format!("{}{}", base_dir, clean_rel))
    }
}

/// Comprueba si una URL pertenece a los dominios CDN de Discord
pub fn is_discord_cdn_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("cdn.discordapp.com")
        || lower.contains("media.discordapp.net")
        || lower.contains(".discordapp.net")
        || lower.contains(".discordapp.com")
}

/// Determina si un recurso de Discord es animado según sus reglas oficiales:
/// 1. Parámetro query `animated=true` (ej. emojis WebP animados, tolerando variantes con amp;)
/// 2. Extensión `.gif`
/// 3. Hash / nombre de archivo que comienza con el prefijo `a_` (avatares, iconos y banners animados de Discord)
pub fn is_discord_animated(url: &str) -> bool {
    if !is_discord_cdn_url(url) {
        return false;
    }

    let unescaped = unescape_html_entities(url);
    let lower = unescaped.to_lowercase();

    // Regla 1: Parámetro query animated=true (soportando limpieza de entidades amp;)
    if let Some((_, query)) = lower.split_once('?') {
        let without_hash = query.split('#').next().unwrap_or(query);
        for param in without_hash.split('&') {
            let key_val = param.trim().trim_start_matches("amp;").trim();
            if key_val == "animated=true"
                || key_val.starts_with("animated=true&")
                || key_val.ends_with("&animated=true")
                || key_val.contains("&animated=true&")
            {
                return true;
            }
        }
    }

    // Regla 2: Extensión .gif
    let without_query = lower.split('?').next().unwrap_or(&lower);
    let without_hash = without_query.split('#').next().unwrap_or(without_query);
    if without_hash.ends_with(".gif") {
        return true;
    }

    // Regla 3: Prefijo `a_` en el identificador / hash del archivo (avatares, iconos, etc.)
    let last_segment = without_hash.trim_end_matches('/').rsplit('/').next().unwrap_or("");
    if last_segment.starts_with("a_") {
        return true;
    }

    false
}

/// Normaliza una URL de Discord:
/// - Si es animada y termina en .webp, convierte a .gif para descarga y previsualización nativa directa
/// - Remueve parámetros superfluos como `size=...` y entidades accidentales como `amp;`
pub fn normalize_discord_url(url: &str) -> String {
    if !is_discord_cdn_url(url) {
        return unescape_html_entities(url);
    }

    let unescaped = unescape_html_entities(url);
    let is_anim = is_discord_animated(&unescaped);
    let without_fragment = unescaped.split('#').next().unwrap_or(&unescaped).trim();
    let (base, query_opt) = match without_fragment.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (without_fragment, None),
    };

    let mut final_base = base.to_string();
    if is_anim && final_base.ends_with(".webp") {
        final_base = format!("{}.gif", &final_base[..final_base.len() - 5]);
    }

    let mut kept_params = Vec::new();
    if let Some(query) = query_opt {
        for param in query.split('&') {
            let clean_param = param.trim().trim_start_matches("amp;").trim();
            let key = clean_param.split('=').next().unwrap_or("").trim();
            if !key.is_empty() && !key.eq_ignore_ascii_case("size") {
                if !is_anim || !clean_param.starts_with("animated=") {
                    kept_params.push(clean_param);
                }
            }
        }
    }

    if kept_params.is_empty() {
        final_base
    } else {
        format!("{}?{}", final_base, kept_params.join("&"))
    }
}

/// Detecta si una URL apunta directamente a un archivo de imagen o endpoint de imagen conocido
pub fn is_direct_image_url(url: &str) -> bool {
    let clean = url.trim();
    if !clean.starts_with("http://") && !clean.starts_with("https://") {
        return false;
    }

    if is_discord_cdn_url(clean) {
        let without_query = clean.split('?').next().unwrap_or(clean).split('#').next().unwrap_or(clean);
        if without_query.contains("/emojis/")
            || without_query.contains("/avatars/")
            || without_query.contains("/stickers/")
            || without_query.contains("/icons/")
            || without_query.contains("/banners/")
            || without_query.contains("/app-icons/")
        {
            return true;
        }
    }

    let without_query = clean.split('?').next().unwrap_or(clean).split('#').next().unwrap_or(clean);
    if let Some(dot_idx) = without_query.rfind('.') {
        let ext = without_query[dot_idx + 1..].to_lowercase();
        return matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "avif" | "bmp");
    }

    false
}

/// Extrae el formato probable a partir de una URL o Data URI
pub fn infer_format(url: &str) -> Option<String> {
    if url.starts_with("data:image/") {
        let mime = &url[11..];
        let end_idx = mime.find(';').or_else(|| mime.find(','))?;
        let sub = &mime[..end_idx].to_lowercase();
        let fmt = match sub.as_str() {
            "jpeg" | "jpg" => "jpg",
            "png" => "png",
            "gif" => "gif",
            "webp" => "webp",
            "svg+xml" | "svg" => "svg",
            other => other,
        };
        return Some(fmt.to_string());
    }

    if is_discord_cdn_url(url) {
        if is_discord_animated(url) {
            return Some("gif".to_string());
        } else {
            let clean = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);
            if clean.ends_with(".png") {
                return Some("png".to_string());
            } else if clean.ends_with(".jpg") || clean.ends_with(".jpeg") {
                return Some("jpg".to_string());
            }
            return Some("webp".to_string());
        }
    }

    let clean = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);
    if let Some(dot_idx) = clean.rfind('.') {
        let ext = clean[dot_idx + 1..].to_lowercase();
        if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "avif" | "bmp") {
            return Some(if ext == "jpeg" { "jpg".to_string() } else { ext });
        }
    }
    None
}

/// Genera un nombre PascalCase adecuado para la imagen
pub fn suggest_name(url: &str, alt: Option<&str>) -> String {
    if let Some(a) = alt {
        let trimmed = a.trim().trim_matches(':').trim();
        if !trimmed.is_empty() && trimmed.len() <= 60 {
            let pascal = to_pascal_case(trimmed);
            if !pascal.is_empty() {
                return pascal;
            }
        }
    }

    if url.starts_with("data:image/") {
        return "ScannedDataImage".to_string();
    }

    let clean = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);
    let last_seg = clean.trim_end_matches('/').rsplit('/').next().unwrap_or("emote");
    let stem = Path::new(last_seg)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Emote")
        .trim();

    if stem.is_empty() {
        "ScannedEmote".to_string()
    } else if stem.chars().all(|c| c.is_ascii_digit()) && is_discord_cdn_url(url) {
        format!("DiscordEmote{}", stem)
    } else {
        let pascal = to_pascal_case(stem);
        if pascal.is_empty() {
            "ScannedEmote".to_string()
        } else {
            pascal
        }
    }
}

/// Extrae el valor de un atributo HTML dado (ej. src="foo" o src='foo')
fn extract_attribute<'a>(tag_content: &'a str, attr_name: &str) -> Option<&'a str> {
    let mut search_idx = 0;
    while let Some(pos) = tag_content[search_idx..].to_lowercase().find(attr_name) {
        let start = search_idx + pos;
        let after_attr = &tag_content[start + attr_name.len()..];
        let trimmed_after = after_attr.trim_start();
        if trimmed_after.starts_with('=') {
            let after_eq = trimmed_after[1..].trim_start();
            if let Some(quote) = after_eq.chars().next() {
                if quote == '"' || quote == '\'' {
                    let val_content = &after_eq[1..];
                    if let Some(end_quote) = val_content.find(quote) {
                        return Some(&val_content[..end_quote]);
                    }
                } else {
                    // Valor sin comillas
                    let end_idx = after_eq.find(|c: char| c.is_whitespace() || c == '>').unwrap_or(after_eq.len());
                    return Some(&after_eq[..end_idx]);
                }
            }
        }
        search_idx = start + attr_name.len();
    }
    None
}

/// Extrae URLs desde un atributo srcset
fn parse_srcset(srcset: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for entry in srcset.split(',') {
        let trimmed = entry.trim();
        if let Some(url) = trimmed.split_whitespace().next() {
            if !url.is_empty() {
                urls.push(url.to_string());
            }
        }
    }
    urls
}

/// Extrae URLs desde declaraciones CSS `url(...)`
fn extract_css_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let lower = text.to_lowercase();
    let mut start_idx = 0;

    while let Some(pos) = lower[start_idx..].find("url(") {
        let content_start = start_idx + pos + 4;
        if let Some(end_paren) = text[content_start..].find(')') {
            let raw_url = text[content_start..content_start + end_paren].trim();
            let clean = raw_url.trim_matches(|c| c == '\'' || c == '"').trim();
            if !clean.is_empty() {
                urls.push(clean.to_string());
            }
            start_idx = content_start + end_paren + 1;
        } else {
            break;
        }
    }
    urls
}

/// Analiza el HTML y extrae todas las imágenes candidatas
pub fn extract_images_from_html(html: &str, base_url: Option<&str>) -> Vec<ScannedImage> {
    let mut results = Vec::new();
    let mut seen_urls = HashSet::new();

    // 1. Detectar si existe etiqueta <base href="...">
    let mut effective_base = base_url.map(|s| s.to_string());
    let lower_html = html.to_lowercase();
    if let Some(base_pos) = lower_html.find("<base ") {
        if let Some(base_end) = html[base_pos..].find('>') {
            let tag_content = &html[base_pos..base_pos + base_end];
            if let Some(href) = extract_attribute(tag_content, "href") {
                if let Some(resolved) = resolve_url(href, base_url) {
                    effective_base = Some(resolved);
                }
            }
        }
    }

    let mut add_candidate = |raw_url: &str, alt: Option<&str>| {
        let Some(resolved) = resolve_url(raw_url, effective_base.as_deref()) else {
            return;
        };

        let target_url = if is_discord_cdn_url(&resolved) {
            normalize_discord_url(&resolved)
        } else {
            resolved
        };

        if seen_urls.insert(target_url.clone()) {
            let fmt = infer_format(&target_url);
            let name = suggest_name(&target_url, alt);
            results.push(ScannedImage {
                url: target_url,
                alt: alt.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                suggested_name: name,
                format_hint: fmt,
            });
        }
    };

    // 2. Extraer emojis personalizados de Discord: <a:name:id> (animado) o <:name:id> (estático)
    let mut discord_search_idx = 0;
    while let Some(start_bracket) = html[discord_search_idx..].find('<') {
        let abs_start = discord_search_idx + start_bracket;
        let rest = &html[abs_start..];

        let is_anim = rest.starts_with("<a:");
        let is_static = rest.starts_with("<:");

        if is_anim || is_static {
            if let Some(end_bracket) = rest.find('>') {
                let inner = &rest[1..end_bracket]; // "a:name:id" o ":name:id"
                let parts: Vec<&str> = inner.split(':').collect();
                if is_anim && parts.len() == 3 {
                    let name = parts[1].trim();
                    let id = parts[2].trim();
                    if !name.is_empty() && id.chars().all(|c| c.is_ascii_digit()) && id.len() >= 15 {
                        let emoji_url = format!("https://cdn.discordapp.com/emojis/{}.gif", id);
                        add_candidate(&emoji_url, Some(name));
                    }
                } else if is_static && parts.len() == 3 {
                    let name = parts[1].trim();
                    let id = parts[2].trim();
                    if !name.is_empty() && id.chars().all(|c| c.is_ascii_digit()) && id.len() >= 15 {
                        let emoji_url = format!("https://cdn.discordapp.com/emojis/{}.webp", id);
                        add_candidate(&emoji_url, Some(name));
                    }
                }
                discord_search_idx = abs_start + end_bracket + 1;
                continue;
            }
        }
        discord_search_idx = abs_start + 1;
    }

    // 3. Extraer etiquetas <img> y <source>
    let mut cursor = 0;
    while let Some(tag_start) = html[cursor..].find('<') {
        let abs_start = cursor + tag_start;
        let Some(tag_len) = html[abs_start..].find('>') else {
            break;
        };
        let tag_str = &html[abs_start..abs_start + tag_len + 1];
        cursor = abs_start + tag_len + 1;

        let lower_tag = tag_str.to_lowercase();
        if lower_tag.starts_with("<img") {
            let alt = extract_attribute(tag_str, "alt").or_else(|| extract_attribute(tag_str, "title"));
            
            // Atributos de origen prioritarios
            if let Some(src) = extract_attribute(tag_str, "src") {
                add_candidate(src, alt);
            }
            if let Some(data_src) = extract_attribute(tag_str, "data-src") {
                add_candidate(data_src, alt);
            }
            if let Some(data_orig) = extract_attribute(tag_str, "data-original") {
                add_candidate(data_orig, alt);
            }
            if let Some(lazy) = extract_attribute(tag_str, "data-lazy-src") {
                add_candidate(lazy, alt);
            }
            if let Some(srcset) = extract_attribute(tag_str, "srcset").or_else(|| extract_attribute(tag_str, "data-srcset")) {
                for u in parse_srcset(srcset) {
                    add_candidate(&u, alt);
                }
            }
        } else if lower_tag.starts_with("<source") {
            if let Some(srcset) = extract_attribute(tag_str, "srcset") {
                for u in parse_srcset(srcset) {
                    add_candidate(&u, None);
                }
            }
            if let Some(src) = extract_attribute(tag_str, "src") {
                add_candidate(src, None);
            }
        } else if lower_tag.starts_with("<link") {
            let rel = extract_attribute(tag_str, "rel").unwrap_or("").to_lowercase();
            if rel.contains("icon") || rel == "preload" {
                if let Some(href) = extract_attribute(tag_str, "href") {
                    add_candidate(href, None);
                }
            }
        } else if lower_tag.starts_with("<a") {
            if let Some(href) = extract_attribute(tag_str, "href") {
                if infer_format(href).is_some() {
                    let title = extract_attribute(tag_str, "title");
                    add_candidate(href, title);
                }
            }
        }

        // Estilos inline style="...url(...)..."
        if let Some(style_val) = extract_attribute(tag_str, "style") {
            for css_url in extract_css_urls(style_val) {
                add_candidate(&css_url, None);
            }
        }
    }

    // 3. Estilos en bloques <style>...</style>
    let mut style_cursor = 0;
    while let Some(start_style) = lower_html[style_cursor..].find("<style") {
        let abs_start = style_cursor + start_style;
        let Some(tag_close) = html[abs_start..].find('>') else {
            break;
        };
        let content_start = abs_start + tag_close + 1;
        let Some(end_style) = lower_html[content_start..].find("</style>") else {
            break;
        };
        let css_block = &html[content_start..content_start + end_style];
        for css_url in extract_css_urls(css_block) {
            add_candidate(&css_url, None);
        }
        style_cursor = content_start + end_style + 8;
    }

    results
}

/// Procesa una lista de elementos recolectados durante la navegación en vivo,
/// resolviendo rutas relativas contra `base_url`, normalizando CDNs (como Discord),
/// deduplicando y generando nombres sugeridos en PascalCase.
pub fn process_scanned_urls(items: &[RawScannedItem], base_url: Option<&str>) -> Vec<ScannedImage> {
    let mut seen = HashSet::new();
    let mut results = Vec::new();

    for item in items {
        let clean = item.url.trim();
        if clean.is_empty() {
            continue;
        }

        let resolved = match resolve_url(clean, base_url) {
            Some(u) => u,
            None => continue,
        };

        if resolved.is_empty() {
            continue;
        }

        let final_url = if is_discord_cdn_url(&resolved) {
            normalize_discord_url(&resolved)
        } else {
            resolved
        };

        if seen.insert(final_url.clone()) {
            let fmt = infer_format(&final_url);
            let name = suggest_name(&final_url, item.alt.as_deref());
            results.push(ScannedImage {
                url: final_url,
                alt: item.alt.clone(),
                suggested_name: name,
                format_hint: fmt,
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url_absolute() {
        assert_eq!(
            resolve_url("https://example.com/cat.png", Some("https://other.com/")),
            Some("https://example.com/cat.png".to_string())
        );
    }

    #[test]
    fn test_resolve_url_protocol_relative() {
        assert_eq!(
            resolve_url("//cdn.example.com/cat.png", Some("https://example.com/gallery/")),
            Some("https://cdn.example.com/cat.png".to_string())
        );
    }

    #[test]
    fn test_resolve_url_root_relative() {
        assert_eq!(
            resolve_url("/assets/dog.webp", Some("https://example.com/gallery/page1")),
            Some("https://example.com/assets/dog.webp".to_string())
        );
    }

    #[test]
    fn test_resolve_url_data_uri() {
        let data = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUg==";
        assert_eq!(resolve_url(data, None), Some(data.to_string()));
    }

    #[test]
    fn test_infer_format() {
        assert_eq!(infer_format("https://example.com/img.png?size=large"), Some("png".to_string()));
        assert_eq!(infer_format("https://example.com/banner.JPEG"), Some("jpg".to_string()));
        assert_eq!(infer_format("data:image/webp;base64,123"), Some("webp".to_string()));
    }

    #[test]
    fn test_suggest_name_from_alt() {
        assert_eq!(suggest_name("https://example.com/12345.png", Some("super cool cat")), "SuperCoolCat");
    }

    #[test]
    fn test_suggest_name_from_url_stem() {
        assert_eq!(suggest_name("https://example.com/happy-pepe_frog.gif", None), "HappyPepeFrog");
    }

    #[test]
    fn test_extract_images_from_html_img_tags() {
        let html = r#"
            <div>
                <img src="/img/cat.png" alt="Cute Cat" />
                <img data-src="https://cdn.site.com/dog.webp" title="Lazy Dog" />
                <img src="data:image/gif;base64,R0lGODlh" alt="inline gif" />
            </div>
        "#;
        let images = extract_images_from_html(html, Some("https://example.com/gallery/"));
        assert_eq!(images.len(), 3);
        assert_eq!(images[0].url, "https://example.com/img/cat.png");
        assert_eq!(images[0].suggested_name, "CuteCat");
        assert_eq!(images[1].url, "https://cdn.site.com/dog.webp");
        assert_eq!(images[1].suggested_name, "LazyDog");
        assert_eq!(images[2].url, "data:image/gif;base64,R0lGODlh");
    }

    #[test]
    fn test_extract_images_from_srcset_and_css() {
        let html = r#"
            <style>
                .hero { background-image: url('/bg/hero.jpg'); }
            </style>
            <picture>
                <source srcset="/img/small.webp 1x, /img/large.webp 2x" />
                <img src="/img/fallback.png" />
            </picture>
            <div style="background: url('https://example.com/avatar.png')"></div>
        "#;
        let images = extract_images_from_html(html, Some("https://example.com/"));
        let urls: Vec<&str> = images.iter().map(|i| i.url.as_str()).collect();
        assert!(urls.contains(&"https://example.com/bg/hero.jpg"));
        assert!(urls.contains(&"https://example.com/img/small.webp"));
        assert!(urls.contains(&"https://example.com/img/large.webp"));
        assert!(urls.contains(&"https://example.com/img/fallback.png"));
        assert!(urls.contains(&"https://example.com/avatar.png"));
    }

    #[test]
    fn test_extract_deduplicates_urls() {
        let html = r#"
            <img src="/img/same.png" alt="first" />
            <img src="/img/same.png" alt="second" />
            <a href="/img/same.png">Link</a>
        "#;
        let images = extract_images_from_html(html, Some("https://example.com/"));
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].url, "https://example.com/img/same.png");
    }

    #[test]
    fn test_is_discord_animated_rules() {
        // animated=true en query (incluyendo variantes con amp; y &amp;)
        assert!(is_discord_animated(
            "https://cdn.discordapp.com/emojis/123.webp?size=48&animated=true"
        ));
        assert!(is_discord_animated(
            "https://cdn.discordapp.com/emojis/123.webp?animated=true"
        ));
        assert!(is_discord_animated(
            "https://cdn.discordapp.com/emojis/808565687815372840.webp?amp;animated=true"
        ));
        assert!(is_discord_animated(
            "https://cdn.discordapp.com/emojis/808565687815372840.webp?size=96&amp;animated=true"
        ));
        assert!(is_discord_animated(
            "https://media.discordapp.net/emojis/123.webp?quality=lossless&animated=true&size=64"
        ));

        // Estático (sin animated=true)
        assert!(!is_discord_animated(
            "https://cdn.discordapp.com/emojis/123.webp?size=48"
        ));
        assert!(!is_discord_animated(
            "https://cdn.discordapp.com/emojis/123.webp"
        ));
        assert!(!is_discord_animated(
            "https://cdn.discordapp.com/emojis/123.png"
        ));

        // Extensión .gif
        assert!(is_discord_animated(
            "https://cdn.discordapp.com/emojis/123.gif"
        ));
        assert!(is_discord_animated(
            "https://cdn.discordapp.com/emojis/123.gif?size=64"
        ));

        // Prefijo a_ en hash de avatar/icono animado
        assert!(is_discord_animated(
            "https://cdn.discordapp.com/avatars/456/a_abcdef123456.webp"
        ));
        assert!(!is_discord_animated(
            "https://cdn.discordapp.com/avatars/456/abcdef123456.webp"
        ));
    }

    #[test]
    fn test_normalize_discord_url() {
        assert_eq!(
            normalize_discord_url(
                "https://cdn.discordapp.com/emojis/123.webp?size=32&animated=true"
            ),
            "https://cdn.discordapp.com/emojis/123.gif"
        );
        assert_eq!(
            normalize_discord_url(
                "https://cdn.discordapp.com/emojis/808565687815372840.webp?amp;animated=true"
            ),
            "https://cdn.discordapp.com/emojis/808565687815372840.gif"
        );
        assert_eq!(
            normalize_discord_url(
                "https://cdn.discordapp.com/emojis/808565687815372840.webp?size=96&amp;animated=true"
            ),
            "https://cdn.discordapp.com/emojis/808565687815372840.gif"
        );
        assert_eq!(
            normalize_discord_url("https://cdn.discordapp.com/emojis/123.webp?size=64"),
            "https://cdn.discordapp.com/emojis/123.webp"
        );
        assert_eq!(
            normalize_discord_url("https://cdn.discordapp.com/avatars/1/a_hash.webp?size=128"),
            "https://cdn.discordapp.com/avatars/1/a_hash.gif"
        );
    }

    #[test]
    fn test_infer_format_discord() {
        assert_eq!(
            infer_format("https://cdn.discordapp.com/emojis/123.webp?size=48&animated=true"),
            Some("gif".to_string())
        );
        assert_eq!(
            infer_format("https://cdn.discordapp.com/emojis/123.webp?size=48"),
            Some("webp".to_string())
        );
        assert_eq!(
            infer_format("https://cdn.discordapp.com/emojis/123.gif"),
            Some("gif".to_string())
        );
        assert_eq!(
            infer_format("https://cdn.discordapp.com/emojis/123.png"),
            Some("png".to_string())
        );
        assert_eq!(
            infer_format("https://cdn.discordapp.com/avatars/1/a_hash.webp"),
            Some("gif".to_string())
        );
        assert_eq!(
            infer_format("https://cdn.discordapp.com/avatars/1/hash.webp"),
            Some("webp".to_string())
        );
    }

    #[test]
    fn test_extract_discord_custom_emojis_from_markup() {
        let html = r#"
            <div>
                Mensaje de Discord con emotes:
                <a:pepe_dance:1049285028472918016>
                <:pepe_happy:1049285028472918017>
                <img class="emoji" src="https://cdn.discordapp.com/emojis/1049285028472918018.webp?size=48&animated=true" alt=":cat_vibe:" />
                <img class="emoji" src="https://cdn.discordapp.com/emojis/1049285028472918019.webp?size=48" alt=":cat_chill:" />
            </div>
        "#;
        let images = extract_images_from_html(html, None);
        assert_eq!(images.len(), 4);

        // 1. <a:pepe_dance:1049...>
        assert_eq!(images[0].url, "https://cdn.discordapp.com/emojis/1049285028472918016.gif");
        assert_eq!(images[0].suggested_name, "PepeDance");
        assert_eq!(images[0].format_hint, Some("gif".to_string()));

        // 2. <:pepe_happy:1049...>
        assert_eq!(images[1].url, "https://cdn.discordapp.com/emojis/1049285028472918017.webp");
        assert_eq!(images[1].suggested_name, "PepeHappy");
        assert_eq!(images[1].format_hint, Some("webp".to_string()));

        // 3. img con animated=true normalizada a .gif directo
        assert_eq!(images[2].url, "https://cdn.discordapp.com/emojis/1049285028472918018.gif");
        assert_eq!(images[2].suggested_name, "CatVibe");
        assert_eq!(images[2].format_hint, Some("gif".to_string()));

        // 4. img estática
        assert_eq!(images[3].url, "https://cdn.discordapp.com/emojis/1049285028472918019.webp");
        assert_eq!(images[3].suggested_name, "CatChill");
        assert_eq!(images[3].format_hint, Some("webp".to_string()));
    }

    #[test]
    fn test_is_direct_image_url() {
        assert!(is_direct_image_url("https://cdn.discordapp.com/emojis/123.webp?size=96&animated=true"));
        assert!(is_direct_image_url("https://cdn.discordapp.com/emojis/123.webp?size=96"));
        assert!(is_direct_image_url("https://example.com/photo.jpg"));
        assert!(is_direct_image_url("https://example.com/animation.gif?v=2"));
        assert!(!is_direct_image_url("https://example.com/gallery"));
        assert!(!is_direct_image_url("https://discord.com/channels/123/456"));
    }

    #[test]
    fn test_process_scanned_urls() {
        let items = vec![
            RawScannedItem {
                url: "https://example.com/cat.png".to_string(),
                alt: Some("funny cat".to_string()),
            },
            RawScannedItem {
                url: "https://example.com/cat.png".to_string(), // Duplicado exacto
                alt: None,
            },
            RawScannedItem {
                url: "/emotes/pepe.gif".to_string(), // Relativo
                alt: None,
            },
            RawScannedItem {
                url: "https://cdn.discordapp.com/emojis/999888.webp?size=64&animated=true".to_string(),
                alt: Some(":vibe_party:".to_string()),
            },
        ];

        let results = process_scanned_urls(&items, Some("https://example.com/stream/"));
        assert_eq!(results.len(), 3);

        assert_eq!(results[0].url, "https://example.com/cat.png");
        assert_eq!(results[0].suggested_name, "FunnyCat");
        assert_eq!(results[0].format_hint, Some("png".to_string()));

        assert_eq!(results[1].url, "https://example.com/emotes/pepe.gif");
        assert_eq!(results[1].suggested_name, "Pepe");
        assert_eq!(results[1].format_hint, Some("gif".to_string()));

        assert_eq!(results[2].url, "https://cdn.discordapp.com/emojis/999888.gif");
        assert_eq!(results[2].suggested_name, "VibeParty");
        assert_eq!(results[2].format_hint, Some("gif".to_string()));
    }

    #[test]
    fn test_process_scanned_urls_blocks_unsafe_schemes() {
        let items = vec![
            RawScannedItem {
                url: "javascript:alert(1)".to_string(),
                alt: None,
            },
            RawScannedItem {
                url: "JaVaScRiPt:alert(document.domain)".to_string(),
                alt: None,
            },
            RawScannedItem {
                url: "vbscript:msgbox(1)".to_string(),
                alt: None,
            },
            RawScannedItem {
                url: "data:text/html,<script>alert(1)</script>".to_string(),
                alt: None,
            },
            RawScannedItem {
                url: "data:image/svg+xml;base64,PHN2Zy8+".to_string(),
                alt: None,
            },
            RawScannedItem {
                url: "file:///C:/Windows/win.ini".to_string(),
                alt: None,
            },
            RawScannedItem {
                url: "/relative/only.png".to_string(),
                alt: None,
            },
            RawScannedItem {
                url: "data:image/png;base64,iVBORw0KGgo=".to_string(),
                alt: Some("safe".to_string()),
            },
        ];

        // Sin base_url: las rutas relativas y pseudo-esquemas deben descartarse sin hacer fallback inseguro
        let results = process_scanned_urls(&items, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "data:image/png;base64,iVBORw0KGgo=");
    }
}
