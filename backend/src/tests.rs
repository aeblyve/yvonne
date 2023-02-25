use crate::container::Container;
use crate::item::Item;
use crate::item_location::ItemLocation;

pub(crate) use super::rocket;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::{Client, LocalResponse};
use rocket::Response;
use std::io::Cursor;

#[test]
fn test_container() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client
        .post("/container")
        .header(ContentType::JSON)
        .body(r#"{ "name": "1000 Washington Street" }"#)
        .dispatch();

    let response = client
        .post("/container")
        .header(ContentType::JSON)
        .body(r#"{ "parent_container_id": 1, "name": "Breadboarding Bin" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let container: Container = response.into_json().expect("Valid response");
    assert_eq!(container.id, None);
    assert_eq!(container.parent_container_id, Some(1));
    assert_eq!(container.name, "Breadboarding Bin");
    assert_eq!(container.note, None);
    assert_eq!(container.photo, None);

    let response = client.get("/container/2").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let container: Container = response.into_json().expect("Valid response");
    assert_eq!(container.id, None);
    assert_eq!(container.parent_container_id, Some(1));
    assert_eq!(container.name, "Breadboarding Bin");
    assert_eq!(container.note, None);
    assert_eq!(container.photo, None);

    let response = client.put("/container/1").header(ContentType::JSON).body(r#"{ "parent_container_id": 1, "name": "1000 Washington Street", "note": "foobar", "photo": null}"#).dispatch();

    // TODO may change later
    assert_eq!(response.status(), Status::Created);

    let response = client.get("/container/1").dispatch();
    let container: Container = response.into_json().expect("Valid response");
    assert_eq!(container.note, Some("foobar".to_string()));

    let response = client.delete("/container/1").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let response = client.get("/container/1").dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let response = client.delete("/container/2").dispatch(); // cascade delete should take out container 2

    assert_eq!(response.status(), Status::NotFound);

    let response = client
        .post("/container")
        .header(ContentType::JSON)
        .body(r#"{ "parent_container_id": 1, "name": "Breadboarding Bin" }"#)
        .dispatch(); //parent no longer exists - this should fail

    assert_eq!(response.status(), Status::InternalServerError)
}

#[test]
fn test_item() {
    // test simple request
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client
        .post("/item")
        .header(ContentType::JSON)
        .body(r#"{ "name": "M3 Bolt, 20mm" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let item: Item = response.into_json().expect("Valid response");
    assert_eq!(item.id, None);
    assert_eq!(item.name, "M3 Bolt, 20mm");
    assert_eq!(item.note, None);
    assert_eq!(item.photo, None);

    let response = client.get("/item/1").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let item: Item = response.into_json().expect("Valid response");
    assert_eq!(item.id, None);
    assert_eq!(item.name, "M3 Bolt, 20mm");
    assert_eq!(item.note, None);
    assert_eq!(item.photo, None);

    let response = client.delete("/item/1").dispatch();
    assert_eq!(response.status(), Status::Ok);

    let response = client.get("/item/1").dispatch();
    assert_eq!(response.status(), Status::NotFound);

    // test optional fields
    let response = client
        .post("/item")
        .header(ContentType::JSON)
        .body(r#"{ "name": "M3 Bolt, 20mm", "note": "Titanium", "photo": [0, 1, 2, 3, 4] }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let item: Item = response.into_json().expect("Valid response");
    assert_eq!(item.id, None);
    assert_eq!(item.name, "M3 Bolt, 20mm");
    assert_eq!(item.note, Some("Titanium".to_string()));
    assert_eq!(item.photo, Some([0, 1, 2, 3, 4].into()));

    let response = client.get("/item/2").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let item: Item = response.into_json().expect("Valid response");
    assert_eq!(item.id, None);
    assert_eq!(item.name, "M3 Bolt, 20mm");
    assert_eq!(item.note, Some("Titanium".to_string()));
    assert_eq!(item.photo, Some([0, 1, 2, 3, 4].into()));
}

#[test]
fn test_item_location() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");

    let response = client
        .post("/site")
        .header(ContentType::JSON)
        .body(r#"{ "name": "1000 Washington Street" }"#)
        .dispatch();

    let response = client
        .post("/container")
        .header(ContentType::JSON)
        .body(r#"{ "site_id": 1, "name": "Breadboarding Bin" }"#)
        .dispatch();

    let response = client
        .post("/item")
        .header(ContentType::JSON)
        .body(r#"{ "name": "M3 Bolt, 20mm" }"#)
        .dispatch();

    let response = client
        .post("/itemloc")
        .header(ContentType::JSON)
        .body(r#"{ "item_id": 1, "container_id": 1}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let itemloc: ItemLocation = response.into_json().expect("Valid response");
    assert_eq!(itemloc.id, None);
    assert_eq!(itemloc.item_id, 1);
    assert_eq!(itemloc.container_id, 1);
    assert_eq!(itemloc.quantity, None);

    let response = client.get("/itemloc/1").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    let itemloc: ItemLocation = response.into_json().expect("Valid response");
    assert_eq!(itemloc.id, None);
    assert_eq!(itemloc.item_id, 1);
    assert_eq!(itemloc.container_id, 1);
    assert_eq!(itemloc.quantity, None);

    let response = client.delete("/itemloc/1").dispatch();
    assert_eq!(response.status(), Status::Ok);

    let response = client.get("/itemloc/1").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}
