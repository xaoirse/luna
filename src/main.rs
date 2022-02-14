// Luna
// Developed by SAoirse
// xaoirse.github.com
//
//        ┌─────────────┐
//        │             │
//        │    LUNA     │
//        │             │
//        └──────┬──────┘
//               │
//        ┌──────▼──────┐
//        │             │
//        │   program   │
//        │             │
//        └──────┬──────┘
//               │
//        ┌──────▼──────┐
//        │  scope:     │
//        │   - domain  │
//        │   - cidr    │
//        └──────┬──────┘
//               │
//        ┌──────▼──────┐
//        │             │
//        │     sub     │
//        │             │
//        └─┬──────────┬┘
//          │          │
//   ┌──────▼─┐     ┌──▼──────┐
//   │   url  │     │  host   │
//   └────┬───┘     └────┬────┘
//        │              │
//        │              │
//   ┌────▼───┐     ┌────▼────┐
//   │  tech  │     │ service │
//   └────────┘     └─────────┘

use log::debug;
mod cmd;
mod model;

fn main() {
    env_logger::init();
    debug!("Luna Begins.");
    cmd::run::run();
    debug!("Luna finished.");
}
