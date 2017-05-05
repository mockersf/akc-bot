macro_rules! create_handler {
    ( $n:ident, $body:expr ) => {
        pub struct $n {}
        impl $n {
            pub fn new() -> $n {
                $n {}
            }
        }
        impl Handler for $n {
            fn handle(&self, req: &mut Request) -> IronResult<Response> {
                $body(self, req)
            }
        }
    }
}

pub mod my_error;

macro_rules! get_path_param {
    ( $r:expr, $e:expr ) => {
        match $r.extensions.get::<Router>() {
            Some(router) => {
                match router.find($e) {
                    Some(val) => val,
                    None => return Ok(Response::with(status::BadRequest)),
                }
            }
            None => return Ok(Response::with(status::InternalServerError)),
        }
    }
}

macro_rules! get_query_param {
    ( $r:expr, $e:expr ) => {
        match $r.get_ref::<UrlEncodedQuery>() {
            Ok(ref hashmap) => hashmap.get($e),
            Err(_) => None
        }
    }
}

macro_rules! get_query_params {
    ( $r:expr$(, $e:expr )+ ) => {
        match $r.get::<UrlEncodedQuery>() {
            Ok(hashmap) => hashmap,
            Err(_) => HashMap::new()
        }
    }
}
