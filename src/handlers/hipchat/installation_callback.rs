use iron::{Handler, status, IronResult, Response, Request};
use bodyparser;
use iron::prelude::*;
use router::Router;

use clients::hipchat::installation::Installation;

use handlers::hipchat::HC_DATABASE;

create_handler!(PostInstallation,
                |_: &PostInstallation, req: &mut Request| {
    let installation = req.get::<bodyparser::Struct<Installation>>();
    match installation {
        Ok(Some(installation)) => {
            let mut locked = HC_DATABASE.lock().unwrap();
            locked.add_installation(installation);
        }
        Ok(None) => info!("No body"),
        Err(err) => info!("Error: {:?}", err),
    }
    Ok(Response::with((status::Ok, "{}".to_string())))
});

create_handler!(DeleteInstallation,
                |_: &DeleteInstallation, req: &mut Request| {
    let installation = get_path_param!(req, "installation").to_string();
    info!("removing {:?}", installation);
    let body = req.get::<bodyparser::Raw>();
    match body {
        Ok(Some(body)) => info!("Read body: {}", body),
        Ok(None) => info!("No body"),
        Err(err) => info!("Error: {:?}", err),
    }
    Ok(Response::with((status::Ok, "".to_string())))
});
