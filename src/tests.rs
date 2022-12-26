pub(crate) use super::rocket;
use rocket::local::blocking::Client;
use rocket::http::{Status, ContentType};

#[test]
fn create_site_minimal() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.post("/site")
        .header(ContentType::JSON)
        .body(r#"{ "name": "1000 Washington Street" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
}

#[test]
fn create_site_maximal() {

}
