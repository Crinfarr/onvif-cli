use reqwest::{header::HeaderMap, Body, Client};
const CREATEPROFILE_XML: &str = include_str!("./fm_xml/CreateProfile.xml");
pub fn create_profile(url:String, auth:Option<String>) {
    let req = Client::default().get(url);
    let headers = HeaderMap::new();
}
