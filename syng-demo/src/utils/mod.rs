use random_string::generate;

pub fn path_to_string(path: &Vec<usize>) -> String {
    path.iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join("/")
}

const REQ_CONTENT_CHARSET: &str = "0123456789abcdef";

pub fn get_random_request_content() -> String {
    generate(6, REQ_CONTENT_CHARSET)
}
