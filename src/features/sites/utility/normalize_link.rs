pub fn normalize_link(base_url: &str, link: &str) -> String {
    if link.is_empty() {
        return link.to_string();
    }

    // base without trailing slash
    let base = base_url.trim().trim_end_matches('/');

    // clean link
    let mut l = link.trim().replace('"', "");

    // remove trailing slash if present
    l = l.trim_end_matches('/').to_string();

    // if link starts with '/', remove leading slash and join
    if let Some(trimmed) = l.strip_prefix('/') {
        format!("{}/{}", base, trimmed)
    } else {
        l
    }
}
