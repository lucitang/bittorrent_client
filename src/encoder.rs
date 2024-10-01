pub fn url_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("%{:02x}", b))
        .collect::<String>()
}
