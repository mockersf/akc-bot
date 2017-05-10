use std::collections::HashMap;

use futures::Future;

use sami::Error;

pub fn find_user(akc_token: &::clients::oauth2::Token) -> Result<::clients::akc::user::User, Error> {
    match ::clients::akc::Akc::user_self(akc_token.clone()).wait() {
        Ok(user) => Ok(user),
        Err(err) => {
            warn!("Error getting user: {:?}", err);
            return Err(Error::AkcError);
        }
    }
}

pub fn find_device_with(akc_token: &::clients::oauth2::Token, indications: &[String]) -> Result<::clients::akc::device::Device, Error> {
    let uid = find_user(akc_token)?.id;
    let mut devices = match ::clients::akc::Akc::devices_parallel(akc_token.clone(), &uid).wait() {
        Ok(devices) => devices,
        Err(err) => {
            warn!("Error getting user: {:?}", err);
            return Err(Error::AkcError);
        }
    };
    for indication in indications {
        devices = devices
            .iter()
            .filter(|device| device.name.to_lowercase().contains(indication))
            .cloned()
            .collect::<Vec<::clients::akc::device::Device>>();
    }
    match devices.get(0).cloned() {
        Some(device) => Ok(device),
        None => Err(Error::NoMatch)
    }
}

pub fn find_field_value_with(akc_token: &::clients::oauth2::Token,
                         device_id: &str,
                         field_indication: &str)
                         -> Result<(String, ::clients::akc::snapshot::FieldValue, Option<u64>), Error> {
    let snapshots = match ::clients::akc::Akc::snapshots(akc_token.clone(), vec![device_id.to_string()]).wait() {
        Ok(snapshots) => snapshots,
        Err(err) => {
            warn!("Error getting snapshot for device {:?}: {:?}",
                  device_id,
                  err);
            return Err(Error::AkcError);
        }
    };
    let snapshot = match snapshots.get(0) {
        Some(snapshot) => snapshot.clone(),
        None => {
            warn!("Error getting snapshot for device {:?}: no result",
                  device_id);
            return Err(Error::AkcError);
        }
    };
    if let ::clients::akc::snapshot::FieldData::Node(root) = snapshot.data {
        recur_find_field(&root, vec![], field_indication)
    } else {
        warn!("Error getting snapshot for device {:?}: no subfields", device_id);
        return Err(Error::NoMatch);
    }
}

fn recur_find_field(subfields: &HashMap<String, Box<::clients::akc::snapshot::FieldData>>,
                    path: Vec<String>,
                    field_indication: &str)
                    -> Result<(String, ::clients::akc::snapshot::FieldValue, Option<u64>), Error> {
    for (name, value) in subfields.iter() {
        info!("{:?} - {:?} : {:?}", path, name, value);
        match **value {
            ::clients::akc::snapshot::FieldData::Leaf {ts, ref value} => {
                if name == field_indication {
                    return Ok((name.to_owned(), value.to_owned(), ts));
                }
            }
            ::clients::akc::snapshot::FieldData::Node(ref subfields) => {
                let mut new_path = path.clone();
                new_path.push(name.to_owned());
                match recur_find_field(subfields, new_path, field_indication) {
                    Ok(result) => return Ok(result),
                    Err(_) => ()
                };
            }
        }
    }
    return Err(Error::NoMatch);
}