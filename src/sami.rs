use std::collections::HashMap;

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

fn find_device_with(akc_token: ::clients::oauth2::Token, indications: &[String]) -> Option<::clients::akc::device::Device> {
    let uid = match ::clients::akc::Akc::user_self(akc_token.clone()).wait() {
        Ok(user) => user.id,
        Err(err) => {
            warn!("Error getting user: {:?}", err);
            return None;
        }
    };
    let mut devices = match ::clients::akc::Akc::devices_parallel(akc_token.clone(), &uid).wait() {
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

fn find_field_value_with(akc_token: ::clients::oauth2::Token,
                         device_id: &str,
                         field_indication: &str)
                         -> Option<(String, Box<::clients::akc::snapshot::FieldData>)> {
    let snapshots = match ::clients::akc::Akc::snapshots(akc_token, vec![device_id.to_string()]).wait() {
        Ok(snapshots) => snapshots,
        Err(err) => {
            warn!("Error getting snapshot for device {:?}: {:?}",
                  device_id,
                  err);
            return None;
        }
    };
    let snapshot = match snapshots.get(0) {
        Some(snapshot) => snapshot.clone(),
        None => {
            warn!("Error getting snapshot for device {:?}: no result",
                  device_id);
            return None;
        }
    };
    let root = match snapshot.data.subfields {
        Some(subfields) => subfields,
        None => {
            warn!("Error getting snapshot for device {:?}: no subfields",
                  device_id);
            return None;
        }
    };
    recur_find_field(&root, vec![], field_indication)
}

fn recur_find_field(subfields: &HashMap<String, Box<::clients::akc::snapshot::FieldData>>,
                    path: Vec<String>,
                    field_indication: &str)
                    -> Option<(String, Box<::clients::akc::snapshot::FieldData>)> {
    for (name, value) in subfields.iter().filter(|entry| entry.1.is_leaf()) {
        info!("{:?} - {:?} : {:?}", path, name, value);
        if name == field_indication {
            return Some((name.to_owned(), value.to_owned()));
        }
    }
    for (name, values) in subfields.iter().filter(|entry| !entry.1.is_leaf()) {
        let mut new_path = path.clone();
        new_path.push(name.to_owned());
        return recur_find_field(&values.to_owned().subfields.unwrap(),
                                new_path,
                                field_indication);
    }
    None
}

pub fn generate_response(akc_token: ::clients::oauth2::Token, nlp_response: NlpResponse) -> MessageToUser {
    info!("{:?}", nlp_response);
    match nlp_response.intent {
        intent @ Intent::GetSelf => {
            MessageToUser {
                intent: intent,
                data: vec![::clients::akc::Akc::user_self(akc_token)
                               .wait()
                               .unwrap()
                               .full_name],
                status: Status::Info,
            }
        }
        intent @ Intent::Logout => {
            MessageToUser {
                intent: intent,
                data: vec![akc_token.access_token().to_string()],
                status: Status::Confirmation,
            }
        }
        intent @ Intent::GetField => {
            let device_indications = nlp_response
                .device
                .unwrap_or_else(|| vec!["no device specified".to_string()]);
            let field_indication = nlp_response
                .field
                .unwrap_or_else(|| "no field".to_string());
            match find_device_with(akc_token.clone(), &device_indications) {
                Some(device) => {
                    match find_field_value_with(akc_token.clone(), &device.id, &field_indication) {
                        Some((field, field_data)) => {
                            MessageToUser {
                                intent: intent,
                                data: vec![device.name, field, field_data.value.unwrap().to_string()],
                                status: Status::Info,
                            }
                        }
                        None => {
                            MessageToUser {
                                intent: intent,
                                data: vec![device.name, field_indication],
                                status: Status::Error,
                            }
                        }

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
