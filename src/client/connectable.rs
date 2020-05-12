/// Can provide multiple URLs
pub trait Connectable {
    /// provide urls
    fn get_urls(self) -> Vec<String>;
}

impl Connectable for String {
    fn get_urls(self) -> Vec<String> {
        vec![self]
    }
}

impl Connectable for Vec<String> {
    fn get_urls(self) -> Vec<String> {
        self
    }
}

impl Connectable for &str {
    fn get_urls(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl Connectable for Vec<&str> {
    fn get_urls(self) -> Vec<String> {
        let mut urls = vec![];
        for url in self {
            urls.push(url.to_string());
        }
        urls
    }
}
