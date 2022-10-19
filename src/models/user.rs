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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct User {
    pub username: String,
    pub authority: UserAuthority,
}

impl User {
    pub async fn fetch(username: String, _password: String) -> Result<Self, AppErrorKind> {
        // fetch user from db or somewhere
        Ok(Self {
            username: username,
            authority: UserAuthority::User
        })
    }

    pub fn save(&self) -> Result<(), AppErrorKind> {
        // Save user changes to a db
        Ok(())
    }

    pub fn delete(&self) -> Result<(), AppErrorKind> {
        // Delete user from a db
        Ok(())
    }

    pub fn insert(&self, session: Session) -> Result<(), AppErrorKind> {
        if let Err(err) = session.insert(USER_SESSION_KEY, self.to_owned()) {
            Err(AppErrorKind::Session(err.to_string()))
        }
        else {
            Ok(())
        }
    }

    pub fn get(session: Session) -> Result<Self, AppErrorKind> {
        if let Some(user) = session.get(USER_SESSION_KEY)? {
            Ok(user)
        }
        else {
            Err(AppErrorKind::Session("user was not found in session".to_owned()))
        }
    }

    pub fn remove(session: Session) -> Result<(), AppErrorKind> {
        if let Some(_) = session.remove(USER_SESSION_KEY) {
            Ok(())
        }
        else {
            Err(AppErrorKind::Session("could not remove user from session".to_owned()))
        }
    }
}

// impl FromRequest for User {
//     // type Config = ();
//     type Error = actix_web::Error;
//     type Future = Pin<Box<dyn Future<Output = Result<User, actix_web::Error>>>>;

//     fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
//         log::info!("From_request start");
//         let fut = Identity::from_request(req, payload);
//         let sessions: Option<&actix_web::web::Data<RwLock<Sessions>>> = req.app_data();
//         if sessions.is_none() {
//             log::warn!("sessions is empty(none)!");
//             return Box::pin(async { Err(actix_web::error::ErrorUnauthorized("unauthorized")) });
//         }
//         let sessions = sessions.unwrap().clone();
//         Box::pin(async move {
//             if let Ok(id) = fut.await?.id() {
//                 if let Some(user) = sessions.read().unwrap().map.get(&id).map(|x| x.clone()) {
//                     return Ok(user);
//                 }
//             };

//             Err(actix_web::error::ErrorUnauthorized("unauthorized"))
//         })
//     }
// }
