use std::fmt::Display;

/// Implemented for types that can be the routes of a web application
pub trait Routes {
    type UrlDisplay: Display;
    fn url(&self) -> &Self::UrlDisplay;
}
