use serde_derive::{Deserialize, Serialize};

use sewup::rdb::errors::Error as LibError;
use sewup_derive::{ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test};

mod errors;

pub mod modules;
use modules::{location, person, post, Location, Person, Post, LOCATION, PERSON, POST};

#[derive(Serialize, Deserialize)]
pub struct Input {
    id: usize,
}

#[ewasm_constructor]
fn constructor() {
    let mut db = sewup::rdb::Db::new().expect("there is no return for constructor currently");
    db.create_table::<Person>();
    db.create_table::<Post>();
    db.commit()
        .expect("there is no return for constructor currently");
}

#[ewasm_fn]
fn add_address_table() -> anyhow::Result<sewup::primitives::EwasmAny> {
    let mut db = sewup::rdb::Db::load(None)?;
    db.create_table::<Location>();
    db.commit()?;
    Ok(().into())
}

#[ewasm_fn]
fn check_version_and_features(
    version: u8,
    features: Vec<sewup::rdb::Feature>,
) -> anyhow::Result<sewup::primitives::EwasmAny> {
    let db = sewup::rdb::Db::load(None)?;
    if db.version() != version {
        return Err(errors::RDBError::UnexpectedVersion(db.version()).into());
    };
    let current_features = db.features();
    if current_features != features {
        return Err(errors::RDBError::IncompatibleFeatures(current_features).into());
    };

    Ok(().into())
}

#[ewasm_fn]
fn check_tables() -> anyhow::Result<sewup::primitives::EwasmAny> {
    let db = sewup::rdb::Db::load(None)?;
    let info = db.table_info::<Person>().unwrap();
    if info.record_raw_size != 1 {
        return Err(
            errors::RDBError::SimpleError("Person record_raw_size not correct".into()).into(),
        );
    }
    if info.range.start != 2 {
        return Err(errors::RDBError::SimpleError("Person range start not correct".into()).into());
    }
    if info.range.end != 2 {
        return Err(errors::RDBError::SimpleError("Person range end not correct".into()).into());
    }
    Ok(().into())
}

#[ewasm_fn]
fn drop_person_table() -> anyhow::Result<sewup::primitives::EwasmAny> {
    let mut db = sewup::rdb::Db::load(None)?;
    db.drop_table::<Person>();
    db.commit()?;
    Ok(().into())
}

#[ewasm_fn]
fn check_tables_again() -> anyhow::Result<sewup::primitives::EwasmAny> {
    let db = sewup::rdb::Db::load(None)?;
    if db.table_info::<Person>().is_some() {
        return Err(errors::RDBError::SimpleError("Person table should be deleted".into()).into());
    }
    Ok(().into())
}

#[ewasm_fn]
fn get_children() -> anyhow::Result<sewup::primitives::EwasmAny> {
    let table = sewup::rdb::Db::load(None)?.table::<Person>()?;
    let people = table.filter_records(&|p: &Person| p.age < 12)?;

    // you can do much complicate filter logic here as you like

    let protocol: person::Protocol = people.into();
    Ok(sewup::primitives::EwasmAny::from(protocol))
}

#[ewasm_fn]
fn get_post_author(input: Input) -> anyhow::Result<sewup::primitives::EwasmAny> {
    let table = sewup::rdb::Db::load(None)?.table::<Post>()?;
    let post = table.get_record(input.id)?;

    // ( Person <- 1 --- many -> Post )
    // use relationship to get the post owner
    let owner = post.person()?;

    // This is an example show output not wrapped into protocol,
    // just return instance itself
    Ok(sewup::primitives::EwasmAny::from(owner))
}

#[ewasm_main(auto)]
fn main() -> anyhow::Result<sewup::primitives::EwasmAny> {
    use sewup_derive::ewasm_input_from;
    let contract = sewup::primitives::Contract::new()?;

    match contract.get_function_selector()? {
        ewasm_fn_sig!(person::get) => ewasm_input_from!(contract move person::get),
        ewasm_fn_sig!(person::create) => ewasm_input_from!(contract move person::create),
        ewasm_fn_sig!(person::update) => ewasm_input_from!(contract move person::update),
        ewasm_fn_sig!(person::delete) => ewasm_input_from!(contract move person::delete),
        ewasm_fn_sig!(post::get) => ewasm_input_from!(contract move post::get),
        ewasm_fn_sig!(post::create) => ewasm_input_from!(contract move post::create),
        ewasm_fn_sig!(post::update) => ewasm_input_from!(contract move post::update),
        ewasm_fn_sig!(post::delete) => ewasm_input_from!(contract move post::delete),
        ewasm_fn_sig!(location::get) => ewasm_input_from!(contract move location::get),
        ewasm_fn_sig!(location::create) => ewasm_input_from!(contract move location::create),
        ewasm_fn_sig!(location::update) => ewasm_input_from!(contract move location::update),
        ewasm_fn_sig!(location::delete) => ewasm_input_from!(contract move location::delete),
        ewasm_fn_sig!(check_version_and_features) => {
            check_version_and_features(0, vec![sewup::rdb::Feature::Default])
        }
        ewasm_fn_sig!(get_post_author) => ewasm_input_from!(contract move get_post_author),
        ewasm_fn_sig!(get_children) => get_children(),
        ewasm_fn_sig!(check_tables) => check_tables(),
        ewasm_fn_sig!(drop_person_table) => drop_person_table(),
        ewasm_fn_sig!(check_tables_again) => check_tables_again(),
        ewasm_fn_sig!(add_address_table) => add_address_table(),
        _ => return Err(errors::RDBError::UnknownHandle.into()),
    }
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup::types::Raw;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_auto_assert_eq, ewasm_err_output};

    #[ewasm_test]
    fn test_execute_crud_handler() {
        let person = Person {
            trusted: true,
            age: 18,
        };

        let mut create_input = person::protocol(person.clone());
        let mut expect_output = create_input.clone();
        expect_output.set_id(1);
        ewasm_auto_assert_eq!(person::create(create_input), expect_output);

        let post = Post {
            content: [
                sewup::types::Raw::from("No Day but today"),
                sewup::types::Raw::from("Embrace who you are"),
            ],
            person_id: 1,
        };
        let mut create_post_input = post::protocol(post);
        let mut expect_post_output = create_post_input.clone();
        expect_post_output.set_id(1);
        ewasm_auto_assert_eq!(post::create(create_post_input), expect_post_output);

        let mut get_input: person::Protocol = Person::default().into();
        get_input.set_id(1);
        ewasm_auto_assert_eq!(person::get(get_input), expect_output);

        ewasm_auto_assert_eq!(get_post_author(Input { id: 1 }), person);

        let child = Person {
            trusted: false,
            age: 9,
        };

        create_input = person::protocol(child.clone());
        expect_output = create_input.clone();
        expect_output.set_id(2);
        ewasm_auto_assert_eq!(person::create(create_input), expect_output);

        get_input.set_id(2);
        ewasm_auto_assert_eq!(person::get(get_input), expect_output);

        let older_person = Person {
            trusted: true,
            age: 20,
        };
        let mut update_input = person::protocol(older_person.clone());
        update_input.set_id(1);
        ewasm_auto_assert_eq!(person::update(update_input), update_input);
        get_input.set_id(1);
        ewasm_auto_assert_eq!(person::get(get_input), update_input);

        // Here is the advance query with filter and selector
        // In the example, the query only want to get the age of trusted person
        let mut person_query = person::Query::default();
        person_query.trusted = Some(true);
        let mut person_query_protocol: person::Protocol = person_query.into();
        assert!(person_query_protocol.filter);
        person_query_protocol.set_select_fields(vec!["age".to_string()]);

        // The expected result only return the age of the trusted person,
        // and other fields will be None
        expect_output = vec![older_person].into();
        expect_output.records[0].trusted = None;
        ewasm_auto_assert_eq!(person::get(person_query_protocol), expect_output);

        // Get the children by the customized handler
        expect_output = vec![child].into();
        ewasm_auto_assert_eq!(get_children(), expect_output);

        // Please Notice that protocol from the default instance may not be empty,
        // this dependents on the default implementation of the struct.
        // You always can use {struct name}::Protocol::default() to get a empty one,
        // then set the id to delete the object.
        let mut default_person_protocol = person::protocol(Person::default());
        assert!(!default_person_protocol.is_empty());

        let mut delete_input = person::Protocol::default();
        assert!(delete_input.is_empty());

        delete_input.set_id(1);
        ewasm_auto_assert_eq!(person::delete(delete_input), delete_input);

        ewasm_assert_eq!(
            person::get(get_input),
            ewasm_err_output!(LibError::RecordDeleted)
        );
    }

    #[ewasm_test]
    fn test_insert_large_records() {
        for i in 1..100 {
            let person = Person {
                trusted: true,
                age: i as u8,
            };

            let create_input = person::protocol(person.clone());
            let mut expect_output = create_input.clone();
            expect_output.set_id(i);
            ewasm_auto_assert_eq!(person::create(create_input), expect_output);

            let mut get_input: person::Protocol = Person::default().into();
            get_input.set_id(i);
            ewasm_auto_assert_eq!(person::get(get_input), expect_output);
        }
    }

    #[ewasm_test]
    fn test_table_management() {
        ewasm_assert_ok!(check_version_and_features());
        ewasm_assert_ok!(check_tables());
        ewasm_assert_ok!(drop_person_table());
        ewasm_assert_ok!(check_tables_again());
    }

    #[ewasm_test]
    fn test_create_tables_after_used() {
        let person = Person {
            trusted: true,
            age: 1,
        };

        let create_input = person::protocol(person.clone());
        let mut expect_output = create_input.clone();
        expect_output.set_id(1);
        ewasm_auto_assert_eq!(person::create(create_input), expect_output);

        let mut get_input: person::Protocol = Person::default().into();
        get_input.set_id(1);
        ewasm_auto_assert_eq!(person::get(get_input), expect_output);

        ewasm_assert_ok!(add_address_table());

        let location = Location {
            address: Raw::from("Utopia"),
        };

        let create_input = location::protocol(location.clone());
        let mut expect_output = create_input.clone();
        expect_output.set_id(1);
        ewasm_auto_assert_eq!(location::create(create_input), expect_output);

        let mut get_input: location::Protocol = Location::default().into();
        get_input.set_id(1);
        ewasm_auto_assert_eq!(location::get(get_input), expect_output);
    }

    #[ewasm_test]
    fn test_record_extended_and_causing_migration() {
        ewasm_assert_ok!(add_address_table());

        let location = Location {
            address: Raw::from("Utopia"),
        };

        let create_input = location::protocol(location.clone());
        let mut location_expect_output = create_input.clone();
        location_expect_output.set_id(1);
        ewasm_auto_assert_eq!(location::create(create_input), location_expect_output);

        let mut location_get_input: location::Protocol = Location::default().into();
        location_get_input.set_id(1);
        ewasm_auto_assert_eq!(location::get(location_get_input), location_expect_output);

        let person = Person {
            trusted: true,
            age: 1,
        };

        let create_input = person::protocol(person.clone());
        let mut expect_output = create_input.clone();
        expect_output.set_id(1);
        ewasm_auto_assert_eq!(person::create(create_input), expect_output);

        let mut get_input: person::Protocol = Person::default().into();
        get_input.set_id(1);
        ewasm_auto_assert_eq!(person::get(get_input), expect_output);

        ewasm_auto_assert_eq!(location::get(location_get_input), location_expect_output);
    }

    #[ewasm_test]
    fn test_table_deleting_causing_migration() {
        let person = Person {
            trusted: true,
            age: 1,
        };

        let create_input = person::protocol(person.clone());
        let mut expect_output = create_input.clone();
        expect_output.set_id(1);
        ewasm_auto_assert_eq!(person::create(create_input), expect_output);

        ewasm_assert_ok!(add_address_table());

        let location = Location {
            address: Raw::from("Utopia"),
        };

        let create_input = location::protocol(location.clone());
        let mut expect_output = create_input.clone();
        expect_output.set_id(1);
        ewasm_auto_assert_eq!(location::create(create_input), expect_output);

        let mut get_input: location::Protocol = Location::default().into();
        get_input.set_id(1);
        ewasm_auto_assert_eq!(location::get(get_input), expect_output.clone());

        ewasm_assert_ok!(drop_person_table());

        ewasm_auto_assert_eq!(location::get(get_input), expect_output);
    }
}
