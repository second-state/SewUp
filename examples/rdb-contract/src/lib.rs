use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use sewup::primitives::{Contract, EwasmAny};
use sewup::rdb::{errors::Error as LibError, Db, Feature};
use sewup_derive::{
    ewasm_fn, ewasm_fn_sig, ewasm_input_from, ewasm_main, ewasm_output_from, ewasm_test,
};

mod errors;
use errors::RDBError;

mod modules;
use modules::{person, Person, PERSON};

#[ewasm_fn]
fn init_db_with_tables() -> Result<EwasmAny> {
    let mut db = Db::new()?;
    db.create_table::<Person>();
    db.commit()?;
    Ok(().into())
}

#[ewasm_fn]
fn check_version_and_features(version: u8, features: Vec<Feature>) -> Result<EwasmAny> {
    let db = Db::load(None)?;
    if db.version() != version {
        return Err(RDBError::UnexpectVersion(db.version()).into());
    };
    let current_features = db.features();
    if current_features != features {
        return Err(RDBError::IncompatibleFeatures(current_features).into());
    };

    Ok(().into())
}

#[ewasm_fn]
fn check_tables() -> Result<EwasmAny> {
    let mut db = Db::load(None)?;
    let info = db.table_info::<Person>().unwrap();
    if info.record_raw_size != 1 {
        return Err(RDBError::SimpleError("Person record_raw_size not correct".into()).into());
    }
    if info.range.start != 2 {
        return Err(RDBError::SimpleError("Person range start not correct".into()).into());
    }
    if info.range.end != 2 {
        return Err(RDBError::SimpleError("Person range end not correct".into()).into());
    }
    Ok(().into())
}

#[ewasm_fn]
fn drop_table() -> Result<EwasmAny> {
    let mut db = Db::load(None)?;
    db.drop_table::<Person>();
    db.commit()?;
    Ok(().into())
}

#[ewasm_fn]
fn check_tables_again() -> Result<EwasmAny> {
    let mut db = Db::load(None)?;
    if db.table_info::<Person>().is_some() {
        return Err(RDBError::SimpleError("Person table should be deleted".into()).into());
    }
    Ok(().into())
}

#[ewasm_fn]
fn get_childern() -> Result<EwasmAny> {
    use sewup::primitives::IntoEwasmAny;

    let table = sewup::rdb::Db::load(None)?.table::<Person>()?;
    let people = table.filter_records(&|p: &Person| p.age < 12)?;

    // you can do much complicate filter logic here as you like

    let protocol: person::Protocol = people.into();
    Ok(EwasmAny::from(protocol))
}

#[ewasm_main(auto)]
fn main() -> Result<EwasmAny> {
    let mut contract = Contract::new()?;

    match contract.get_function_selector()? {
        ewasm_fn_sig!(person::get) => ewasm_input_from!(contract move person::get),
        ewasm_fn_sig!(person::create) => ewasm_input_from!(contract move person::create),
        ewasm_fn_sig!(person::update) => ewasm_input_from!(contract move person::update),
        ewasm_fn_sig!(person::delete) => ewasm_input_from!(contract move person::delete),
        ewasm_fn_sig!(check_version_and_features) => {
            check_version_and_features(0, vec![Feature::Default])
        }
        ewasm_fn_sig!(get_childern) => get_childern(),
        ewasm_fn_sig!(init_db_with_tables) => init_db_with_tables(),
        ewasm_fn_sig!(check_tables) => check_tables(),
        ewasm_fn_sig!(drop_table) => drop_table(),
        ewasm_fn_sig!(check_tables_again) => check_tables_again(),
        _ => return Err(RDBError::UnknownHandle.into()),
    }
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_auto_assert_eq, ewasm_err_output};

    #[ewasm_test]
    fn test_execute_crud_handler() {
        ewasm_assert_ok!(init_db_with_tables());

        let person = Person {
            trusted: true,
            age: 18,
        };

        let mut create_input = person::protocol(person.clone());
        let mut expect_output = create_input.clone();
        expect_output.set_id(1);
        ewasm_auto_assert_eq!(person::create(create_input), expect_output);

        let mut get_input: person::Protocol = Person::default().into();
        get_input.set_id(1);
        ewasm_auto_assert_eq!(person::get(get_input), expect_output);

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

        // Get the childern by the customized handler
        expect_output = vec![child].into();
        ewasm_auto_assert_eq!(get_childern(), expect_output);

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
    fn test_table_management() {
        ewasm_assert_ok!(init_db_with_tables());
        ewasm_assert_ok!(check_version_and_features());
        ewasm_assert_ok!(check_tables());
        ewasm_assert_ok!(drop_table());
        ewasm_assert_ok!(check_tables_again());
    }
}
