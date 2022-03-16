// Luna
// Developed by SAoirse
// xaoirse.github.com

pub mod run;

use log::debug;

use run::run;

fn main() {
    env_logger::init();
    debug!("Luna Begins.");
    run();
    debug!("Luna finished.");
}
