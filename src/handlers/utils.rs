pub fn sanitize_html(input: &String) -> String {
    return input.replace('<', "&lt;").replace('>', "&gt;");
}
