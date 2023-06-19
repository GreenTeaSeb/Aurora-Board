pub fn sanitize_html(input: &str) -> String {
    return input.replace('<', "&lt;").replace('>', "&gt;");
}

pub fn bb_to_html(input: &str) -> String {
    todo!()
}
