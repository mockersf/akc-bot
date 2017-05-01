macro_rules! create_handler {
    ( $n:ident, [ $( $t:ident ),* ], $body:expr ) => {
        pub struct $n {}
        $(impl $t for $n {})*
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
