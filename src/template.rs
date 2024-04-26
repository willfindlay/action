use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct APMTemplate<'a> {
    pub css: &'a str,
}

#[derive(serde::Serialize)]
pub struct APMData {
    pub apm: u64,
}
