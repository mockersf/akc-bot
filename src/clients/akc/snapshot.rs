use std::collections::HashMap;
use std::fmt;

use hyper::Url;
use futures::future::*;

use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};

use clients::akc::Akc;
use clients::akc::error::AkcClientError;
use clients::akc::helpers;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DataSnapshot {
    data: Vec<Snapshot>,
}

impl helpers::DataWrapper for DataSnapshot {
    type Data = Vec<Snapshot>;
    fn data(self: Self) -> Self::Data {
        self.data
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Snapshot {
    pub sdid: String,
    pub data: FieldData,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct FieldValue {
    #[serde(skip_serializing_if="Option::is_none")]
    pub float: Option<f64>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub int: Option<i64>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub string: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub boolean: Option<bool>,
}
impl FieldValue {
    pub fn float(f: f64) -> FieldValue {
        FieldValue {
            float: Some(f),
            ..Default::default()
        }
    }
    pub fn int(i: i64) -> FieldValue {
        FieldValue {
            int: Some(i),
            ..Default::default()
        }
    }
    fn string(s: String) -> FieldValue {
        FieldValue {
            string: Some(s),
            ..Default::default()
        }
    }
    fn boolean(b: bool) -> FieldValue {
        FieldValue {
            boolean: Some(b),
            ..Default::default()
        }
    }
}
impl fmt::Display for FieldValue {
    fn fmt(&self, fm: &mut fmt::Formatter) -> fmt::Result {
        match (self.float, self.int, self.string.clone(), self.boolean) {
            (Some(f), _, _, _) => write!(fm, "{:?}", f),
            (_, Some(i), _, _) => write!(fm, "{:?}", i),
            (_, _, Some(s), _) => write!(fm, "{:?}", s),
            (_, _, _, Some(b)) => write!(fm, "{:?}", b),
            (_, _, _, _) => write!(fm, "none"),
        }

    }
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct FieldData {
    #[serde(skip_serializing_if="Option::is_none")]
    pub ts: Option<u64>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub value: Option<FieldValue>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub subfields: Option<HashMap<String, Box<FieldData>>>,
}
impl FieldData {
    fn leaf(value: FieldValue, ts: u64) -> FieldData {
        FieldData {
            ts: Some(ts),
            value: Some(value),
            ..Default::default()
        }
    }
    fn node(subfields: HashMap<String, Box<FieldData>>) -> FieldData {
        FieldData {
            subfields: Some(subfields),
            ..Default::default()
        }
    }
    pub fn is_leaf(self: &Self) -> bool {
        self.ts.is_some() && self.value.is_some()
    }
}

impl<'de> Deserialize<'de> for FieldData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        #[derive(Debug)]
        enum Field {
            Ts,
            Value,
            Subfield(String),
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where D: Deserializer<'de>
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("ts and value, or map of subfields")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where E: de::Error
                    {
                        match value {
                            "ts" => Ok(Field::Ts),
                            "value" => Ok(Field::Value),
                            subfield => Ok(Field::Subfield(subfield.to_string())),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DetailOrSubfield {
            detail: Option<FieldValue>,
            subfield: Option<FieldData>,
        }
        impl DetailOrSubfield {
            fn detail(fv: FieldValue) -> DetailOrSubfield {
                DetailOrSubfield {
                    detail: Some(fv),
                    subfield: None,
                }
            }
            fn subfield(fd: FieldData) -> DetailOrSubfield {
                DetailOrSubfield {
                    detail: None,
                    subfield: Some(fd),
                }
            }
        }
        impl<'de> Deserialize<'de> for DetailOrSubfield {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>
            {
                struct DetailOrSubfieldVisitor;

                impl<'de> Visitor<'de> for DetailOrSubfieldVisitor {
                    type Value = DetailOrSubfield;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("detail of a field, or a subfield")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<DetailOrSubfield, E>
                        where E: de::Error
                    {
                        Ok(DetailOrSubfield::detail(FieldValue::string(value.to_string())))
                    }
                    fn visit_bool<E>(self, value: bool) -> Result<DetailOrSubfield, E>
                        where E: de::Error
                    {
                        Ok(DetailOrSubfield::detail(FieldValue::boolean(value)))
                    }
                    fn visit_f64<E>(self, value: f64) -> Result<DetailOrSubfield, E>
                        where E: de::Error
                    {
                        Ok(DetailOrSubfield::detail(FieldValue::float(value)))
                    }
                    fn visit_i64<E>(self, value: i64) -> Result<DetailOrSubfield, E>
                        where E: de::Error
                    {
                        Ok(DetailOrSubfield::detail(FieldValue::int(value as i64)))
                    }
                    fn visit_u64<E>(self, value: u64) -> Result<DetailOrSubfield, E>
                        where E: de::Error
                    {
                        Ok(DetailOrSubfield::detail(FieldValue::int(value as i64)))
                    }
                    fn visit_map<M>(self, visitor: M) -> Result<DetailOrSubfield, M::Error>
                        where M: de::MapAccess<'de>
                    {
                        let map = de::value::MapAccessDeserializer::new(visitor);
                        let demap: Result<FieldData, M::Error> = Deserialize::deserialize(map);
                        Ok(DetailOrSubfield::subfield(demap?))
                    }
                }

                deserializer.deserialize_any(DetailOrSubfieldVisitor)
            }
        }

        struct FieldDataVisitor;

        impl<'de> Visitor<'de> for FieldDataVisitor {
            type Value = FieldData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Snapshot")
            }

            fn visit_map<V>(self, mut map: V) -> Result<FieldData, V::Error>
                where V: MapAccess<'de>
            {
                let mut ts: Option<u64> = None;
                let mut value: Option<FieldValue> = None;
                let mut values: HashMap<String, Box<FieldData>> = HashMap::new();
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Ts => {
                            if ts.is_some() {
                                return Err(de::Error::duplicate_field("ts"));
                            }
                            let tmp = map.next_value::<DetailOrSubfield>()?;
                            match (tmp.detail, tmp.subfield) {
                                (Some(ref detail), None) if detail.int.is_some() => ts = Some(detail.int.unwrap() as u64),
                                (None, Some(subfield)) => {
                                    values.insert("ts".to_string(), Box::new(subfield));
                                    ()
                                }
                                (_, _) => (),
                            }
                        }
                        Field::Value => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }
                            let tmp = map.next_value::<DetailOrSubfield>()?;
                            match (tmp.detail, tmp.subfield) {
                                (Some(detail), None) => value = Some(detail),
                                (None, Some(subfield)) => {
                                    values.insert("value".to_string(), Box::new(subfield));
                                    ()
                                }
                                (_, _) => (),
                            }
                        }
                        Field::Subfield(subfield) => {
                            if values.contains_key(&subfield) {
                                return Err(de::Error::duplicate_field("subfield"));
                            }
                            values.insert(subfield, Box::new(map.next_value()?));
                        }
                    }
                }
                if ts.is_some() && value.is_some() {
                    Ok(FieldData::leaf(value.unwrap(), ts.unwrap()))
                } else {
                    Ok(FieldData::node(values))
                }
            }
        }

        const FIELDS: &'static [&'static str] = &["ts", "value"];
        deserializer.deserialize_struct("FieldData", FIELDS, FieldDataVisitor)
    }
}



impl Akc {
    pub fn snapshots(token: ::clients::oauth2::Token, sdid: Vec<String>) -> Box<Future<Item = Vec<Snapshot>, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/messages/snapshots", Self::base_url::<'static>())).unwrap();

        Self::get_with_params::<DataSnapshot>(token, url, vec![("sdids".to_string(), sdid.join(","))])
    }
}
