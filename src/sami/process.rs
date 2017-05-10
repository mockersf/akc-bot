use sami::{Intent, output, input, akc_request, Error};

pub fn generate_response(akc_token: ::clients::oauth2::Token, nlp_response: input::NlpResponse) -> output::MessageToUser {
    info!("{:?}", nlp_response);
    match nlp_response.intent {
        
        intent @ Intent::GetSelf => {
            match akc_request::find_user(&akc_token) {
                Ok(user) => output::MessageToUser {
                    intent: intent,
                    data: vec![user.full_name],
                    status: output::Status::Info,
                },
                Err(_) => {
                    output::MessageToUser {
                        intent: Intent::ForcedLogout,
                        data: vec![akc_token.access_token().to_string()],
                        status: output::Status::Error,
                    }                            
                }
            }
        }

        intent @ Intent::Logout => {
            output::MessageToUser {
                intent: intent,
                data: vec![akc_token.access_token().to_string()],
                status: output::Status::Confirmation,
            }
        }
        
        intent @ Intent::GetField => {
            let device_indications = nlp_response
                .device
                .unwrap_or_else(|| vec!["no device specified".to_string()]);
            let field_indication = nlp_response
                .field
                .unwrap_or_else(|| "no field".to_string());
            match akc_request::find_device_with(&akc_token, &device_indications) {
                Ok(device) => {
                    match akc_request::find_field_value_with(&akc_token, &device.id, &field_indication) {
                        Ok((field, field_value, _)) => {
                            output::MessageToUser {
                                intent: intent,
                                data: vec![device.name, field, field_value.to_string()],
                                status: output::Status::Info,
                            }
                        },
                        Err(Error::NoMatch) => {
                            output::MessageToUser {
                                intent: intent,
                                data: vec![device.name, field_indication],
                                status: output::Status::Error,
                            }
                        },
                        Err(Error::AkcError) => {
                            output::MessageToUser {
                                intent: Intent::ForcedLogout,
                                data: vec![akc_token.access_token().to_string()],
                                status: output::Status::Error,
                            }                            
                        }
                    }
                },
                Err(Error::NoMatch) => {
                    output::MessageToUser {
                        intent: intent,
                        data: vec![device_indications.join(" ")],
                        status: output::Status::Error,
                    }
                },
                Err(Error::AkcError) => {
                    output::MessageToUser {
                        intent: Intent::ForcedLogout,
                        data: vec![akc_token.access_token().to_string()],
                        status: output::Status::Error,
                    }                            
                }
            }
        }

        intent => {
            output::MessageToUser {
                intent,
                data: nlp_response.meta.unwrap_or_else(|| vec![]),
                status: output::Status::Error,
            }
        }
    }
}