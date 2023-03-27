use regex::Regex;

static YOUTUBE_REGEX: &str = r"^((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?$";
static YOUTUBE_PREFIX: &str = "https://www.youtube.com/watch?v=";
// group 5 is the video id

pub enum UrlType {
    ID,
    URL,
}

pub fn url_checker(url: &str, url_type: UrlType) -> Option<String> {
    let youtube_pattern = Regex::new(YOUTUBE_REGEX).unwrap();
    if youtube_pattern.is_match(url) {
        let id = youtube_pattern.captures(url).unwrap().get(5).map(|m| m.as_str().to_string());
        match url_type {
            UrlType::ID => id,
            UrlType::URL => Some(format!("{}{}", YOUTUBE_PREFIX, id.unwrap())),
        }
        
    } else {
        None
    }
}