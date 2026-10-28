pub const PROVIDER_NAME: &str = "Moly";
pub const PROVIDER_VERSION: &str = env!("CARGO_PKG_VERSION");
// don't ever change this, it is used to identify the provider in the database
// existing books will use this slug to identify the provider
pub const PROVIDER_ID: &str = "moly";
pub const PROVIDER_FAVICON_URL: &str = "https://moly.hu/favicon.ico";
pub const PROVIDER_COLOR: &str = "#b7cfe7";
pub const PROVIDER_BOOK_TEMPLATE_URL: &str = "https://moly.hu/konyvek/{id}";
