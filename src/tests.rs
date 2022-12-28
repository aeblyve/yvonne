use crate::site::Site;

pub(crate) use super::rocket;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use rocket::Response;
use std::io::Cursor;

#[test]
fn test_site() {
    // test simple request
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client
        .post("/site")
        .header(ContentType::JSON)
        .body(r#"{ "name": "1000 Washington Street" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let site: Site = response.into_json().expect("Valid response");
    assert_eq!(site.id, None);
    assert_eq!(site.name, "1000 Washington Street");
    assert_eq!(site.note, None);
    assert_eq!(site.photo, None);

    let response = client.get("/site/1").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let site: Site = response.into_json().expect("Valid response");
    assert_eq!(site.id, None);
    assert_eq!(site.name, "1000 Washington Street");
    assert_eq!(site.note, None);
    assert_eq!(site.photo, None);

    let response = client.delete("/site/1").dispatch();
    assert_eq!(response.status(), Status::Ok);

    let response = client.get("/site/1").dispatch();
    assert_eq!(response.status(), Status::NotFound);

    // test optional fields
    let response = client
        .post("/site")
        .header(ContentType::JSON)
        .body(r#"{ "name": "1001 Washington Street", "note": "fish", "photo": [0, 1, 2, 3, 4] }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let site: Site = response.into_json().expect("Valid response");
    assert_eq!(site.id, None);
    assert_eq!(site.name, "1001 Washington Street");
    assert_eq!(site.note, Some("fish".to_string()));
    assert_eq!(site.photo, Some([0, 1, 2, 3, 4].into()));

    let response = client.get("/site/2").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let site: Site = response.into_json().expect("Valid response");
    assert_eq!(site.id, None);
    assert_eq!(site.name, "1001 Washington Street");
    assert_eq!(site.note, Some("fish".to_string()));
    assert_eq!(site.photo, Some([0, 1, 2, 3, 4].into()));
}
