use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use sewup::primitives::Contract;
use sewup::rdb::{Db, Feature};
use sewup::types::{Raw, Row};
use sewup_derive::{ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_output_from, ewasm_test, Table};

mod errors;
use errors::RDBError;

#[derive(Table)]
struct Person {
    trusted: bool,
    age: u8,
}

#[ewasm_fn]
fn init_db_with_tables() -> Result<()> {
    let mut db = Db::new()?;
    db.create_table("Table1", 1);
    db.create_table("Table2", 2);
    db.create_table("Person", 3); // TODO: fix this when implementing table
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
    let mut info = db.table_info("Table1").unwrap();
    if info.record_size != 1 {
        return Err(RDBError::SimpleError("Table1 record_size not correct".into()).into());
    }

    info = db.table_info("Table2").unwrap();
    if info.record_size != 2 {
        return Err(RDBError::SimpleError("Table2 record_size not correct".into()).into());
    }

    info = db.table_info("Person").unwrap();
    if info.record_size != 3 {
        return Err(RDBError::SimpleError("Person record_size not correct".into()).into());
    }
    Ok(())
}

#[ewasm_fn]
fn drop_table() -> Result<()> {
    let mut db = Db::load(None)?;
    db.drop_table("Table2");
    db.commit()?;
    Ok(())
}

#[ewasm_fn]
fn check_tables_again() -> Result<()> {
    let mut db = Db::load(None)?;
    let mut info = db.table_info("Table1").unwrap();
    if info.record_size != 1 {
        return Err(RDBError::SimpleError("Table1 record_size not correct".into()).into());
    }
    info = db.table_info("Person").unwrap();
    if info.record_size != 3 {
        return Err(RDBError::SimpleError("Person record_size not correct".into()).into());
    }
    Ok(())
}
#[ewasm_main]
fn main() -> Result<()> {
    let mut contract = Contract::new()?;

    match contract.get_function_selector()? {
        ewasm_fn_sig!(person::get) => person::get()?,
        ewasm_fn_sig!(person::create) => person::create()?,
        ewasm_fn_sig!(person::update) => person::update()?,
        ewasm_fn_sig!(person::delete) => person::delete()?,
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
        // TODO: correctly implement the handler
        ewasm_assert_eq!(person::get(), vec![]);
        ewasm_assert_eq!(person::create(), vec![]);
        ewasm_assert_eq!(person::update(), vec![]);
        ewasm_assert_eq!(person::delete(), vec![]);
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
