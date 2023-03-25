use regex::Regex;

static youtube_regex: &str = r"^((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?$";
// group 5 is the video id

pub fn url_checker(url: &str) -> Option<String> {
    let youtube_pattern = Regex::new(youtube_regex).unwrap();
    if youtube_pattern.is_match(url) {
        youtube_pattern.captures(url).unwrap().get(5).map(|m| m.as_str().to_string())
    } else {
        None
    }
}