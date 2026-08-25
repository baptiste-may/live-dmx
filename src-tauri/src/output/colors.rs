pub fn resolve_color_rgb(color_name_or_hex: &str) -> (u8, u8, u8) {
    let s = color_name_or_hex.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return (r, g, b);
            }
        }
    }
    match s.to_lowercase().as_str() {
        "white" => (255, 255, 255),
        "ctb" => (198, 230, 255),
        "cto" => (255, 199, 138),
        "red" => (255, 0, 0),
        "orange" => (255, 127, 0),
        "yellow" => (255, 255, 0),
        "lime" => (127, 255, 0),
        "green" => (0, 255, 0),
        "mint" => (0, 255, 127),
        "cyan" => (0, 255, 255),
        "azure" => (0, 127, 255),
        "blue" => (0, 0, 255),
        "violet" => (127, 0, 255),
        "magenta" => (255, 0, 255),
        "pink" => (255, 0, 127),
        _ => (255, 255, 255),
    }
}
