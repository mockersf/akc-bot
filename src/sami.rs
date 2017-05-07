use futures::Future;

#[derive(Debug)]
pub enum Status {
    Info,
    Confirmation,
    Error,
    ActionRequired,
}

#[derive(Debug)]
pub struct MessageToUser {
    pub intent: Intent,
    pub data: Vec<String>,
    pub status: Status,
}

fn find_device_with(from: &str, indications: &[String]) -> Option<::clients::akc::device::Device> {
    let uid = match ::clients::akc::Akc::user_self(from.to_string()).wait() {
        Ok(user) => user.id,
        Err(err) => {
            warn!("Error getting user: {:?}", err);
            return None;
        }
    };
    let mut devices = match ::clients::akc::Akc::devices_parallel(from.to_string(), &uid).wait() {
        Ok(devices) => devices,
        Err(err) => {
            warn!("Error getting user: {:?}", err);
            return None;
        }
    };
    for indication in indications {
        devices = devices
            .iter()
            .filter(|device| device.name.to_lowercase().contains(indication))
            .cloned()
            .collect::<Vec<::clients::akc::device::Device>>();
    }
    devices.get(0).cloned()
}

pub fn generate_response(from: String, nlp_response: NlpResponse) -> MessageToUser {
    info!("{:?}", nlp_response);
    match nlp_response.intent {
        intent @ Intent::GetSelf => {
            MessageToUser {
                intent: intent,
                data: vec![::clients::akc::Akc::user_self(from)
                               .wait()
                               .unwrap()
                               .full_name],
                status: Status::Info,
            }
        }
        intent @ Intent::GetField => {
            let device_indications =
                nlp_response
                    .device
                    .unwrap_or_else(|| vec!["no device specified".to_string()]);
            let field_indication = nlp_response
                .field
                .unwrap_or_else(|| "no field".to_string());
            match find_device_with(&from, &device_indications) {
                Some(device) => {
                    MessageToUser {
                        intent: intent,
                        data: vec![device.name, field_indication],
                        status: Status::Error,
                    }
                }
                None => {
                    MessageToUser {
                        intent: intent,
                        data: vec![device_indications.join(" ")],
                        status: Status::Error,
                    }
                }
            }
        }
        intent => {
            MessageToUser {
                intent,
                data: nlp_response.meta.unwrap_or_else(|| vec![]),
                status: Status::Error,
            }
        }
    }
}

#[derive(Debug)]
pub enum Intent {
    SetField,
    GetField,
    FindDeviceType,
    Logout,
    GetSelf,
    Unknown,
}

impl Default for Intent {
    fn default() -> Intent {
        Intent::Unknown
    }
}

#[derive(Default, Debug)]
pub struct NlpResponse {
    pub intent: Intent,
    pub device: Option<Vec<String>>,
    pub value: Option<String>,
    pub field: Option<String>,
    pub meta: Option<Vec<String>>,
}
