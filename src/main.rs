// Luna project is a web hunting app
// SAoirse

mod cmd;
mod model;
mod mylog;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
