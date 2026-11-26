//! CrawlerEngine Feature Slice

#[allow(non_snake_case)]
pub mod Common;
#[allow(non_snake_case)]
pub mod Controller;
#[allow(non_snake_case)]
pub mod Model;
#[allow(non_snake_case)]
pub mod Services;

pub use Controller::crawler_router;
pub use Services::{CkeyCrawlerService, CkeyParser};
