 // src/certifications.rs
 
 use crate::rest::templates;
 
 pub fn init_certifications() {
    templates::certs::certify_not_allowed_templates_responses();
    templates::certs::certify_not_found_response();
 }
 
 pub fn get_certified_response(request: &ic_http_certification::HttpRequest) -> ic_http_certification::HttpResponse<'static> {
    templates::certs::get_certified_response(request)
 }
 