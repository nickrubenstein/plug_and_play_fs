use std::{collections::HashMap, cmp::Ordering};

use actix_session::Session;
use serde::{Deserialize, Serialize};

use crate::util::error::AppErrorKind;

const USER_SESSION_KEY: &str = "user";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UserAuthority {
    Guest,
    User,
    Admin,
}

impl Default for UserAuthority {
    fn default() -> Self {
        UserAuthority::Guest
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub authority: UserAuthority,
}

impl Default for User {
    fn default() -> Self { 
        Self {
            username: "Guest".to_string(),
            authority: UserAuthority::Guest
        }
    }
}

impl User {
    pub async fn fetch(username: &str, password: &str) -> Result<Self, AppErrorKind> {
        // TODO Fetch user from db with username and password
        Self::test_only_get_user(username, password)
    }

    pub fn save(&self) -> Result<(), AppErrorKind> {
        // TODO Save user changes to a db
        Ok(())
    }

    pub fn delete(&self) -> Result<(), AppErrorKind> {
        // TODO Delete user from a db
        Ok(())
    }

    pub fn insert(&self, session: Session) -> Result<(), AppErrorKind> {
        if let Err(err) = session.insert(USER_SESSION_KEY, self.to_owned()) {
            Err(AppErrorKind::Session(err.to_string(), Some(session)))
        }
        else {
            Ok(())
        }
    }

    pub fn get(session: Session) -> Result<Self, AppErrorKind> {
        if let Ok(Some(user)) = session.get(USER_SESSION_KEY) {
            Ok(user)
        }
        else {
            Err(AppErrorKind::Session("user must login again".to_owned(), Some(session)))
        }
    }

    pub fn remove(session: Session) -> Result<(), AppErrorKind> {
        if let Some(_) = session.remove(USER_SESSION_KEY) {
            Ok(())
        }
        else {
            Err(AppErrorKind::Session("could not logout user from session".to_owned(), Some(session)))
        }
    }

    fn test_only_get_user(username: &str, password: &str) -> Result<User, AppErrorKind> {
        let mut test_users: HashMap<&str, (&str, User)> = HashMap::new();
        test_users.insert("admin", ("admin123", User {
            username: "admin".to_string(),
            authority: UserAuthority::Admin
        }));
        test_users.insert("nick", ("testing", User {
            username: "nick".to_string(),
            authority: UserAuthority::User
        }));
        test_users.insert("guest", ("guest", User {
            username: "guest".to_string(),
            authority: UserAuthority::Guest
        }));
        if let Some((real_password, user)) = test_users.get(username) {
            if real_password.cmp(&password) != Ordering::Equal {
                Err(AppErrorKind::InvalidUserCredentials)
            }
            else {
                Ok(user.to_owned())
            }
        }
        else {
            Err(AppErrorKind::InvalidUserCredentials)
        }
    }
}
