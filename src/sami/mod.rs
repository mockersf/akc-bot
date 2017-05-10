pub mod input;
pub mod output;

mod akc_request;
mod process;

#[derive(Debug)]
pub enum Intent {
    SetField,
    GetField,
    FindDeviceType,
    Logout,
    ForcedLogout,
    GetSelf,
    Unknown,
}

impl Default for Intent {
    fn default() -> Intent {
        Intent::Unknown
    }
}

pub enum Error {
    AkcError,
    NoMatch,
}
