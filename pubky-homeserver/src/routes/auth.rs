use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use bytes::Bytes;
use heed::BytesEncode;
use postcard::to_allocvec;
use tower_cookies::{Cookie, Cookies};

use pubky_common::{
    crypto::{random_bytes, random_hash},
    session::Session,
    timestamp::Timestamp,
};

use crate::{
    database::tables::{
        sessions::{SessionsTable, SESSIONS_TABLE},
        users::{User, UsersTable, USERS_TABLE},
    },
    error::{Error, Result},
    extractors::Pubky,
    server::AppState,
};

pub async fn signup(
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    cookies: Cookies,
    pubky: Pubky,
    body: Bytes,
) -> Result<impl IntoResponse> {
    // TODO: Verify invitation link.
    // TODO: add errors in case of already axisting user.
    signin(State(state), TypedHeader(user_agent), cookies, pubky, body).await
}

pub async fn session(
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    cookies: Cookies,
    pubky: Pubky,
) -> Result<impl IntoResponse> {
    if let Some(cookie) = cookies.get(&pubky.public_key().to_string()) {
        let rtxn = state.db.env.read_txn()?;

        let sessions: SessionsTable = state
            .db
            .env
            .open_database(&rtxn, Some(SESSIONS_TABLE))?
            .expect("Session table already created");

        if let Some(session) = sessions.get(&rtxn, cookie.value())? {
            let session = session.to_owned();
            rtxn.commit()?;

            return Ok(session);
        };

        rtxn.commit()?;
    };

    Err(Error::with_status(StatusCode::NOT_FOUND))
}

pub async fn signout(
    State(state): State<AppState>,
    cookies: Cookies,
    pubky: Pubky,
) -> Result<impl IntoResponse> {
    if let Some(cookie) = cookies.get(&pubky.public_key().to_string()) {
        let mut wtxn = state.db.env.write_txn()?;

        let sessions: SessionsTable = state
            .db
            .env
            .open_database(&wtxn, Some(SESSIONS_TABLE))?
            .expect("Session table already created");

        let _ = sessions.delete(&mut wtxn, cookie.value());

        wtxn.commit()?;

        return Ok(());
    };

    Err(Error::with_status(StatusCode::UNAUTHORIZED))
}

pub async fn signin(
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    cookies: Cookies,
    pubky: Pubky,
    body: Bytes,
) -> Result<impl IntoResponse> {
    let public_key = pubky.public_key();

    state.verifier.verify(&body, public_key)?;

    let mut wtxn = state.db.env.write_txn()?;
    let users: UsersTable = state
        .db
        .env
        .open_database(&wtxn, Some(USERS_TABLE))?
        .expect("Users table already created");

    if let Some(existing) = users.get(&wtxn, public_key)? {
        users.put(&mut wtxn, public_key, &existing)?;
    } else {
        users.put(
            &mut wtxn,
            public_key,
            &User {
                created_at: Timestamp::now().into_inner(),
            },
        )?;
    }

    let session_secret = base32::encode(base32::Alphabet::Crockford, &random_bytes::<16>());

    let sessions: SessionsTable = state
        .db
        .env
        .open_database(&wtxn, Some(SESSIONS_TABLE))?
        .expect("Sessions table already created");

    // TODO: handle not having a user agent?
    let mut session = Session::new();

    session.set_user_agent(user_agent.to_string());

    sessions.put(&mut wtxn, &session_secret, &session.serialize())?;

    cookies.add(Cookie::new(public_key.to_string(), session_secret));

    wtxn.commit()?;

    Ok(())
}
