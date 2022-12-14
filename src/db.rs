use marine_rs_sdk::marine;
use marine_sqlite_connector::{Connection, Error, Result, Value};

const DB_PATH: &str = "/tmp/submission_service_db.sqlite";

pub fn get_none_error() -> Error {
    Error {
        code: None,
        message: Some("Value doesn't exist".to_string()),
    }
}

pub fn get_connection() -> Connection {
    Connection::open(DB_PATH).unwrap()
}

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "
      create table if not exists users (
        near_address TEXT unique not null primary key,
        email TEXT unique not null,
        first_name TEXT default null,
        last_name TEXT default null,
        is_student INTEGER,
        country TEXT default null,
        git_handler TEXT default null,
        linkedin_handler TEXT default null,
        twitter_handler TEXT default null
      ) without rowid;
      ",
    )?;

    conn.execute(
        "
      create table if not exists events (
        id INTEGER not null primary key AUTOINCREMENT, 
        type TEXT not null,
        title TEXT not null,
        start_date TEXT not null,
        end_date TEXT default null,
        logo TEXT not null,
        status INTEGER not null
      );
      ",
    )?;

    conn.execute(
        "
      create table if not exists submissions (
        uuid INTEGER not null primary key AUTOINCREMENT, 
        event_id INTEGER not null,
        project_name TEXT not null,
        description TEXT not null,
        thumbnail TEXT default null,
        git_url TEXT not null,
        live_demo_url TEXT default null,
        video_demo_url TEXT not null,
        submit_by TEXT not null,
        status INTEGER not null,
        created_at DATETIME default CURRENT_TIMESTAMP,
        FOREIGN KEY (submit_by) REFERENCES users,
        FOREIGN KEY (event_id) REFERENCES events
      );
      ",
    )?;

    conn.execute(
        "
      create table if not exists submission_team (
        uuid INTEGER not null primary key AUTOINCREMENT, 
        near_address TEXT not null,
        created_at DATETIME default CURRENT_TIMESTAMP
      );
      ",
    )?;

    Ok(())
}

pub fn delete_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "
      drop table if exists submissions;
      drop table if exists events;
      drop table if exists users;
      ",
    )?;

    Ok(())
}

pub fn add_user(conn: &Connection, near_address: String, email: String) -> Result<User> {
    conn.execute(format!(
        "
      insert into users (near_address, email)
      values ('{}', '{}');
      ",
        near_address, email
    ))?;

    get_user(conn, near_address)
}

pub fn get_user(conn: &Connection, near_address: String) -> Result<User> {
    let mut cursor = conn
        .prepare(format!(
            "select * from users where near_address = '{}';",
            near_address
        ))?
        .cursor();

    let row = cursor.next()?;
    let found_user = User::from_row(row.ok_or(get_none_error())?);
    Ok(found_user?)
}

pub fn update_user(
    conn: &Connection,
    near_address: String,
    first_name: String,
    last_name: String,
    is_student: u8,
    country: String,
    git: String,
    linkedin: String,
    twitter: String,
) -> Result<User> {
    let _ = conn.execute(format!(
        "
        UPDATE users
        SET first_name = '{}',
            last_name = '{}',
            is_student = '{}',
            country = '{}',
            git_handler = '{}',
            linkedin_handler = '{}',
            twitter_handler = '{}'
        WHERE 
            near_address = '{}';
        ",
        first_name, last_name, is_student, country, git, linkedin, twitter, near_address
    ));

    get_user(conn, near_address)
}

#[marine]
#[derive(Default)]
pub struct User {
    pub near_address: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_student: i64,
    pub country: String,
    pub git_handler: String,
    pub linkedin_handler: String,
    pub twitter_handler: String,
    pub err_msg: String,
    pub success: bool,
}

impl User {
    pub fn from_row(row: &[Value]) -> Result<User> {
        let user = User {
            near_address: row[0].as_string().ok_or(get_none_error())?.to_string(),
            email: row[1].as_string().ok_or(get_none_error())?.to_string(),
            first_name: row[2].as_string().unwrap_or_default().to_string(),
            last_name: row[3].as_string().unwrap_or_default().to_string(),
            is_student: row[4].as_integer().unwrap_or_default(),
            country: row[5].as_string().unwrap_or_default().to_string(),
            git_handler: row[6].as_string().unwrap_or_default().to_string(),
            linkedin_handler: row[7].as_string().unwrap_or_default().to_string(),
            twitter_handler: row[8].as_string().unwrap_or_default().to_string(),
            err_msg: "".to_string(),
            success: true,
            ..Default::default()
        };

        Ok(user)
    }

    pub fn from_res(res: Result<User>) -> User {
        match res {
            Ok(v) => v,
            Err(e) => {
                let mut res_user: User = Default::default();
                res_user.err_msg = e.to_string();
                res_user.success = false;
                res_user
            }
        }
    }
}

// SUBMISSION
#[marine]
#[derive(Default)]
pub struct Submission {
    pub uuid: i64,
    pub event_id: i64,
    pub project_name: String,
    pub description: String,
    pub thumbnail: String,
    pub git_url: String,
    pub live_demo_url: String,
    pub video_demo_url: String,
    pub submit_by: String,
    pub status: i64,
    pub created_by: String,
    pub err_msg: String,
    pub success: bool,
}

impl Submission {
    pub fn from_row(row: &[Value]) -> Result<Submission> {
        let submission = Submission {
            uuid: row[0].as_integer().ok_or(get_none_error())?,
            event_id: row[1].as_integer().ok_or(get_none_error())?,
            project_name: row[2].as_string().ok_or(get_none_error())?.to_string(),
            description: row[3].as_string().ok_or(get_none_error())?.to_string(),
            thumbnail: row[4].as_string().unwrap_or_default().to_string(),
            git_url: row[5].as_string().unwrap_or_default().to_string(),
            live_demo_url: row[6].as_string().unwrap_or_default().to_string(),
            video_demo_url: row[7].as_string().unwrap_or_default().to_string(),
            submit_by: row[8].as_string().unwrap_or_default().to_string(),
            status: row[9].as_integer().unwrap_or_default(),
            err_msg: "".to_string(),
            success: true,
            ..Default::default()
        };

        Ok(submission)
    }

    pub fn from_res(res: Result<Submission>) -> Submission {
        match res {
            Ok(v) => v,
            Err(e) => {
                let mut res_submit: Submission = Default::default();
                res_submit.err_msg = e.to_string();
                res_submit.success = false;
                res_submit
            }
        }
    }
}

pub fn add_submission(
    conn: &Connection,
    event_id: i64,
    name: String,
    description: String,
    thumbnail: String,
    git: String,
    live_url: String,
    video_url: String,
    submit_by: String,
) -> Result<Submission> {
    conn.execute(format!(
        "
      insert into submissions (event_id, project_name, description, thumbnail, git_url, live_demo_url, video_demo_url, submit_by, status)
      values ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', 1);
      ",
        event_id, name, description, thumbnail, git, live_url, video_url, submit_by
    ))?;

    let new_row_id = conn
        .prepare("select last_insert_rowid();")?
        .cursor()
        .next()?
        .ok_or(get_none_error())?[0]
        .as_integer()
        .ok_or(get_none_error())?;

    get_submission(conn, new_row_id)
}

pub fn update_submission(
    conn: &Connection,
    id: i64,
    name: String,
    description: String,
    thumbnail: String,
    git_url: String,
    demo_url: String,
    video_url: String,
) -> Result<Submission> {
    let _ = conn.execute(format!(
        "
        UPDATE submissions
        SET
            project_name = '{}',
            description = '{}',
            thumbnail = '{}',
            git_url = '{}',
            live_demo_url = '{}',
            video_demo_url = '{}'
        WHERE 
            uuid = '{}';
        ",
        name, description, thumbnail, git_url, demo_url, video_url, id
    ));

    get_submission(conn, id)
}

pub fn submit_submission(conn: &Connection, id: i64) -> Result<Submission> {
    let _ = conn.execute(format!(
        "
        UPDATE submissions
        SET status = 2
        WHERE 
            uuid = '{}';
        ",
        id
    ));

    get_submission(conn, id)
}

pub fn get_submission(conn: &Connection, uuid: i64) -> Result<Submission> {
    let mut cursor = conn
        .prepare(format!("select * from submissions where uuid = {};", uuid))?
        .cursor();

    let row = cursor.next()?;
    let found_user = Submission::from_row(row.ok_or(get_none_error())?);
    Ok(found_user?)
}

pub fn get_user_submission_for_event(
    conn: &Connection,
    address: String,
    event_id: i64,
) -> Result<Submission> {
    let mut cursor = conn
        .prepare(format!(
            "select * from submissions where submit_by = '{}' AND event_id = {};",
            address, event_id
        ))?
        .cursor();

    let row = cursor.next()?;
    let found_user = Submission::from_row(row.ok_or(get_none_error())?);
    Ok(found_user?)
}

pub fn get_submissions(conn: &Connection) -> Result<Vec<Submission>> {
    let mut cursor = conn.prepare("select * from submissions;")?.cursor();

    let mut submissions = Vec::new();
    while let Some(row) = cursor.next()? {
        submissions.push(Submission::from_row(row)?);
    }

    Ok(submissions)
}

pub fn get_submissions_by_event(conn: &Connection, event_id: i64) -> Result<Vec<Submission>> {
    let mut cursor = conn
        .prepare(format!(
            "select * from submissions where event_id = {};",
            event_id
        ))?
        .cursor();

    let mut submissions = Vec::new();
    while let Some(row) = cursor.next()? {
        submissions.push(Submission::from_row(row)?);
    }

    Ok(submissions)
}

// EVENTS
#[marine]
#[derive(Default)]
pub struct Event {
    pub id: i64,
    pub title: String,
    pub event_type: String,
    pub start_date: String,
    pub end_date: String,
    pub logo: String,
    pub status: i64,
    pub err_msg: String,
    pub success: bool,
}

impl Event {
    pub fn from_row(row: &[Value]) -> Result<Event> {
        let event = Event {
            id: row[0].as_integer().ok_or(get_none_error())?,
            title: row[1].as_string().ok_or(get_none_error())?.to_string(),
            event_type: row[2].as_string().ok_or(get_none_error())?.to_string(),
            start_date: row[3].as_string().unwrap_or_default().to_string(),
            end_date: row[4].as_string().unwrap_or_default().to_string(),
            logo: row[5].as_string().unwrap_or_default().to_string(),
            status: row[6].as_integer().ok_or(get_none_error())?,
            err_msg: "".to_string(),
            success: true,
            ..Default::default()
        };

        Ok(event)
    }

    pub fn from_res(res: Result<Event>) -> Event {
        match res {
            Ok(v) => v,
            Err(e) => {
                let mut res_event: Event = Default::default();
                res_event.err_msg = e.to_string();
                res_event.success = false;
                res_event
            }
        }
    }
}

pub fn add_event(
    conn: &Connection,
    title: String,
    event_type: String,
    start_date: String,
    end_date: String,
    logo: String,
) -> Result<Event> {
    conn.execute(format!(
        "
      insert into events (title, type, start_date, end_date, logo, status)
      values ('{}', '{}', '{}', '{}', '{}', 1);
      ",
        title, event_type, start_date, end_date, logo
    ))?;

    log::info!(
        "
    insert into events (title, type, start_date, end_date, logo, status)
    values ('{}', '{}', '{}', '{}', '{}', 1);
    ",
        title,
        event_type,
        start_date,
        end_date,
        logo
    );

    let new_row_id = conn
        .prepare("select last_insert_rowid();")?
        .cursor()
        .next()?
        .ok_or(get_none_error())?[0]
        .as_integer()
        .ok_or(get_none_error())?;

    get_event(conn, new_row_id)
}

pub fn update_event(
    conn: &Connection,
    id: i64,
    title: String,
    event_type: String,
    start_date: String,
    end_date: String,
    logo: String,
) -> Result<Event> {
    let _ = conn.execute(format!(
        "
        UPDATE events
        SET
            title = '{}',
            type = '{}',
            start_date = '{}',
            end_date = '{}',
            logo = '{}'
        WHERE 
            id = '{}';
        ",
        title, event_type, start_date, end_date, logo, id
    ));

    get_event(conn, id)
}

pub fn close_event(conn: &Connection, id: i64) -> Result<Event> {
    let _ = conn.execute(format!(
        "
        UPDATE events
        SET
            status = 2
        WHERE 
            id = '{}';
        ",
        id
    ));

    get_event(conn, id)
}

pub fn get_event(conn: &Connection, id: i64) -> Result<Event> {
    let mut cursor = conn
        .prepare(format!("select * from events where id = {};", id))?
        .cursor();

    let row = cursor.next()?;
    let found_user = Event::from_row(row.ok_or(get_none_error())?);
    Ok(found_user?)
}

pub fn get_live_events(conn: &Connection) -> Result<Vec<Event>> {
    let mut cursor = conn
        .prepare("select * from events where status = 1;")?
        .cursor();

    let mut events = Vec::new();
    while let Some(row) = cursor.next()? {
        events.push(Event::from_row(row)?);
    }

    Ok(events)
}

pub fn get_events(conn: &Connection) -> Result<Vec<Event>> {
    let mut cursor = conn.prepare("select * from events;")?.cursor();

    let mut events = Vec::new();
    while let Some(row) = cursor.next()? {
        events.push(Event::from_row(row)?);
    }

    Ok(events)
}
