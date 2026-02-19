use gradus_app::app;

pub fn main() {
    if let Err(err) = app::launch(){
        panic!("Error while launching the app: {:?}", err)
    }
}
