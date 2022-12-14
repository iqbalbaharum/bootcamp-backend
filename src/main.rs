use marine_rs_sdk::{marine, module_manifest, WasmLoggerBuilder};
use marine_sqlite_connector::{Error, Result};

mod auth;
mod db;

use auth::*;
use db::*;

module_manifest!();

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

#[marine]
pub struct IFResult {
    pub success: bool,
    pub err_msg: String,
}

impl IFResult {
    pub fn from_res(res: Result<()>) -> IFResult {
        match res {
            Ok(_v) => IFResult {
                success: true,
                err_msg: "".into(),
            },
            Err(e) => IFResult {
                success: false,
                err_msg: e.to_string(),
            },
        }
    }

    pub fn from_err_str(e: &str) -> IFResult {
        IFResult {
            success: false,
            err_msg: e.to_string(),
        }
    }
}

#[marine]
pub fn init_service() -> IFResult {
    if !am_i_owner() {
        return IFResult::from_err_str("You are not the owner!");
    }

    let conn = db::get_connection();
    let res = db::create_tables(&conn);
    IFResult::from_res(res)
}

#[marine]
pub fn reset_service() -> IFResult {
    if !am_i_owner() {
        return IFResult::from_err_str("You are not the owner!");
    }

    let conn = db::get_connection();
    let res = db::delete_tables(&conn);
    IFResult::from_res(res)
}

#[marine]
pub fn register_user(near_address: String, email: String) -> User {
    let conn = db::get_connection();
    let res = db::add_user(&conn, near_address, email);
    User::from_res(res)
}

#[marine]
pub fn update_user(
    near_address: String,
    first_name: String,
    last_name: String,
    is_student: u8,
    country: String,
    git: String,
    linkedin: String,
    twitter: String,
) -> User {
    let conn = db::get_connection();
    let res = db::update_user(
        &conn,
        near_address,
        first_name,
        last_name,
        is_student,
        country,
        git,
        linkedin,
        twitter,
    );
    User::from_res(res)
}

#[marine]
pub fn get_user(near_address: String) -> User {
    let conn = db::get_connection();
    let user = db::get_user(&conn, near_address);

    User::from_res(user)
}

#[marine]
pub fn draft(
    event_id: i64,
    name: String,
    description: String,
    thumbnail: String,
    git: String,
    live_url: String,
    video_url: String,
    submit_by: String,
) -> Submission {
    let conn = db::get_connection();

    // check user
    match db::get_user(&conn, submit_by.clone()) {
        Ok(_) => {
            match db::get_event(&conn, event_id) {
                Ok(_) => {
                    // check if user already submitted
                    let user_submission =
                        db::get_user_submission_for_event(&conn, submit_by.clone(), event_id);

                    if !user_submission.unwrap().success {
                        let submission = db::add_submission(
                            &conn,
                            event_id,
                            name,
                            description,
                            thumbnail,
                            git,
                            live_url,
                            video_url,
                            submit_by,
                        );

                        Submission::from_res(submission)
                    } else {
                        Submission::from_res(Err(Error {
                            code: None,
                            message: Some("User have submitted project".to_string()),
                        }))
                    }
                }
                Err(e) => Submission::from_res(Err(Error {
                    code: None,
                    message: Some(e.to_string().to_string()),
                })),
            }
        }
        Err(err) => Submission::from_res(Err(Error {
            code: None,
            message: Some(err.to_string().to_string()),
        })),
    }
}

#[marine]
pub fn update_submission(
    id: i64,
    name: String,
    description: String,
    thumbnail: String,
    git: String,
    live_url: String,
    video_url: String,
) {
    let conn = db::get_connection();
    let submission = db::get_submission(&conn, id).expect("No submission record");

    if submission.status == 1 {
        db::update_submission(
            &conn,
            id,
            name,
            description,
            thumbnail,
            git,
            live_url,
            video_url,
        )
        .unwrap_or_default();
    }
}

#[marine]
pub fn submit(id: i64) -> Submission {
    let conn = db::get_connection();
    let submission = db::submit_submission(&conn, id);

    Submission::from_res(submission)
}

#[marine]
pub fn get_submission(id: i64) -> Submission {
    let conn = db::get_connection();
    let submission = db::get_submission(&conn, id);

    Submission::from_res(submission)
}

#[marine]
pub fn get_user_event_submission(address: String, event_id: i64) -> Submission {
    let conn = db::get_connection();
    let submission = db::get_user_submission_for_event(&conn, address, event_id);

    Submission::from_res(submission)
}

#[marine]
pub fn get_submissions() -> Vec<Submission> {
    let conn = db::get_connection();
    let res = db::get_submissions(&conn);
    res.unwrap_or_default()
}

#[marine]
pub fn get_event_submissions(event_id: i64) -> Vec<Submission> {
    let conn = db::get_connection();
    let res = db::get_submissions_by_event(&conn, event_id);
    res.unwrap_or_default()
}

// event
#[marine]
pub fn add_event(
    title: String,
    event_type: String,
    start_date: String,
    end_date: String,
    logo: String,
) -> Event {
    let conn = db::get_connection();
    let res = db::add_event(&conn, title, event_type, start_date, end_date, logo);
    Event::from_res(res)
}

#[marine]
pub fn update_event(
    id: i64,
    title: String,
    event_type: String,
    start_date: String,
    end_date: String,
    logo: String,
) -> Event {
    let conn = db::get_connection();
    let res = db::update_event(&conn, id, title, event_type, start_date, end_date, logo);
    Event::from_res(res)
}

#[marine]
pub fn close_event(id: i64) -> Event {
    let conn = db::get_connection();
    let res = db::close_event(&conn, id);
    Event::from_res(res)
}

#[marine]
pub fn get_event(id: i64) -> Event {
    let conn = db::get_connection();
    let res = db::get_event(&conn, id);

    Event::from_res(res)
}

#[marine]
pub fn get_events() -> Vec<Event> {
    let conn = db::get_connection();
    let res = db::get_events(&conn);
    res.unwrap_or_default()
}

#[marine]
pub fn get_live_events() -> Vec<Event> {
    let conn = db::get_connection();
    let res = db::get_live_events(&conn);
    res.unwrap_or_default()
}
