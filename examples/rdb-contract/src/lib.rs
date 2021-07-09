use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup::rdb::{errors::Error as LibError, Db, Feature};
use sewup::types::{Raw, Row};
use sewup_derive::{
    ewasm_fn, ewasm_fn_sig, ewasm_input_from, ewasm_main, ewasm_output_from, ewasm_test, Table,
};

mod errors;
use errors::RDBError;

// Table derive provides the handers for CRUD,
// to communicate with these handler, you will need protocol.
// The protocol is easy to build by the `{struct_name}::protocol`, `{struct_name}::Protocol`,
// please check out the test case in the end of this document
#[derive(Table, Default, Serialize, Deserialize)]
pub struct Person {
    trusted: bool,
    age: u8,
}

#[ewasm_fn]
fn init_db_with_tables() -> Result<()> {
    let mut db = Db::new()?;
    db.create_table::<Person>();
    db.commit()?;
    Ok(())
}

#[ewasm_fn]
fn check_version_and_features(version: u8, features: Vec<Feature>) -> Result<()> {
    let db = Db::load(None)?;
    if db.version() != version {
        return Err(RDBError::UnexpectVersion(db.version()).into());
    };
    let current_features = db.features();
    if current_features != features {
        return Err(RDBError::IncompatibleFeatures(current_features).into());
    };

    Ok(())
}

#[ewasm_fn]
fn check_tables() -> Result<()> {
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
    Ok(())
}

#[ewasm_fn]
fn drop_table() -> Result<()> {
    let mut db = Db::load(None)?;
    db.drop_table::<Person>();
    db.commit()?;
    Ok(())
}

#[ewasm_fn]
fn check_tables_again() -> Result<()> {
    let mut db = Db::load(None)?;
    if db.table_info::<Person>().is_some() {
        return Err(RDBError::SimpleError("Person table should be deleted".into()).into());
    }
    Ok(())
}
#[ewasm_main]
fn main() -> Result<()> {
    let mut contract = Contract::new()?;

    match contract.get_function_selector()? {
        ewasm_fn_sig!(person::get) => ewasm_input_from!(contract, person::get)?,
        ewasm_fn_sig!(person::create) => ewasm_input_from!(contract, person::create)?,
        ewasm_fn_sig!(person::update) => ewasm_input_from!(contract, person::update)?,
        ewasm_fn_sig!(person::delete) => ewasm_input_from!(contract, person::delete)?,
        ewasm_fn_sig!(check_version_and_features) => {
            check_version_and_features(0, vec![Feature::Default])?
        }
        ewasm_fn_sig!(init_db_with_tables) => init_db_with_tables()?,
        ewasm_fn_sig!(check_tables) => check_tables()?,
        ewasm_fn_sig!(drop_table) => drop_table()?,
        ewasm_fn_sig!(check_tables_again) => check_tables_again()?,
        _ => return Err(RDBError::UnknownHandle.into()),
    }

    Ok(())
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{ewasm_assert_eq, ewasm_assert_ok, ewasm_err_output};

    #[ewasm_test]
    fn test_execute_crud_handler() {
        ewasm_assert_ok!(init_db_with_tables());

        let person = Person {
            trusted: true,
            age: 18,
        };

        let create_input = person::protocol(person);
        let mut expect_output = create_input.clone();
        expect_output.set_id(1);
        ewasm_assert_eq!(
            person::create(create_input),
            ewasm_output_from!(expect_output)
        );

        let mut get_input: person::Protocol = Person::default().into();
        get_input.set_id(1);
        ewasm_assert_eq!(person::get(get_input), ewasm_output_from!(expect_output));

        let older_person = Person {
            trusted: true,
            age: 20,
        };
        let mut update_input = person::protocol(older_person);
        update_input.set_id(1);
        ewasm_assert_eq!(
            person::update(update_input),
            ewasm_output_from!(update_input)
        );
        ewasm_assert_eq!(person::get(get_input), ewasm_output_from!(update_input));

        // Please Notice that protocol from the default instance may not be empty,
        // this dependents on the default implementation of the struct.
        // You always can use {struct name}::Protocol::default() to get a empty one,
        // then set the id to delete the object.
        let mut default_person_protocol = person::protocol(Person::default());
        assert!(!default_person_protocol.is_empty());

        let mut delete_input = person::Protocol::default();
        assert!(delete_input.is_empty());

        delete_input.id = Some(1);
        ewasm_assert_eq!(
            person::delete(delete_input),
            ewasm_output_from!(delete_input)
        );

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
