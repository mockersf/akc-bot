use std::collections::HashMap;

use futures::Future;

use oauth2;
use akc;

use sami::Error;

use USER_CACHE;
use DEVICE_CACHE;

#[derive(Debug)]
pub struct FieldValueAndPath {
    pub path: Vec<String>,
    pub name: String,
    pub value: akc::snapshot::FieldValue,
    pub ts: Option<u64>,
}
impl Clone for FieldValueAndPath {
    fn clone(&self) -> FieldValueAndPath {
        FieldValueAndPath {
            path: self.path.clone(),
            name: self.name.clone(),
            value: self.value.clone(),
            ts: self.ts,
        }
    }
}

macro_rules! cache_get_or_set {
    ( $c:expr, $t:expr, $m:expr ) => {
        {
            let has_key = {
                let mut lock = $c.lock().unwrap();
                lock.contains_key(&$t)
            };
            if has_key {
                let mut lock = $c.lock().unwrap();
                lock.get(&$t).unwrap().clone()
            } else {
                match $m {
                    Ok(v) => {
                        let mut lock = $c.lock().unwrap();
                        lock.insert($t, v.clone());
                        v
                    },
                    Err(err) => {
                        warn!("Error: {:?}", err);
                        return Err(Error::AkcError);
                    }
                }
            }
        }
    }
}


pub fn find_user(akc_token: &oauth2::Token) -> Result<akc::user::User, Error> {
    Ok(cache_get_or_set!(USER_CACHE,
                         akc_token.access_token().to_string(),
                         akc::Akc::user_self(akc_token.clone()).wait()))
}

pub fn find_device_with(akc_token: &oauth2::Token, indications: &[String]) -> Result<akc::device::Device, Error> {
    let uid = find_user(akc_token)?.id;
    let mut devices = cache_get_or_set!(DEVICE_CACHE,
                                        akc_token.access_token().to_string(),
                                        akc::Akc::devices_parallel(akc_token.clone(), &uid).wait());
    for indication in indications {
        devices = devices
            .iter()
            .filter(|device| device.name.to_lowercase().contains(indication))
            .cloned()
            .collect::<Vec<akc::device::Device>>();
    }
    match devices.get(0).cloned() {
        Some(device) => Ok(device),
        None => Err(Error::NoMatch),
    }
}

pub fn find_field_value_with(akc_token: &oauth2::Token,
                             device_id: &str,
                             field_indication: &str)
                             -> Result<FieldValueAndPath, Error> {
    let snapshots = match akc::Akc::snapshots(akc_token.clone(), vec![device_id.to_string()]).wait() {
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
    if let akc::snapshot::FieldData::Group(root) = snapshot.data {
        let mut fields = recur_find_fields(&root, vec![], field_indication);
        match fields.len() {
            0 => Err(Error::NoMatch),
            _ => {
                info!("fields found: {:?}", fields);
                fields.sort_by(|a, b| a.path.len().cmp(&b.path.len()));
                Ok(fields[0].clone())
            }
        }
    } else {
        warn!("Error getting snapshot for device {:?}: no subfields",
              device_id);
        Err(Error::NoMatch)
    }
}

fn recur_find_fields(subfields: &HashMap<String, Box<akc::snapshot::FieldData>>,
                     path: Vec<String>,
                     field_indication: &str)
                     -> Vec<FieldValueAndPath> {
    let mut result = vec![];
    for (name, value) in subfields.iter() {
        info!("{:?} - {:?} : {:?}", path, name, value);
        match **value {
            akc::snapshot::FieldData::Field { ts, ref value } => {
                if name == field_indication {
                    result.push(FieldValueAndPath {
                                    path: path.clone(),
                                    name: name.to_owned(),
                                    value: value.to_owned(),
                                    ts,
                                });
                }
            }
            akc::snapshot::FieldData::Group(ref subfields) => {
                let mut new_path = path.clone();
                new_path.push(name.to_owned());
                result.extend(recur_find_fields(subfields, new_path, field_indication));
            }
        }
    }
    result
}

}
