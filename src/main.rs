extern crate eventsource;
extern crate hyper;

use eventsource::*;
use hyper::Url;

fn main() {
    //let url = Url::parse("https://clonkspot.org/league/game_events.php").unwrap();
    let url = Url::parse("http://league.openclonk.org/poll_game_events.php").unwrap();
    let client = Client::new(url);
    for event in client {
        println!("{:?}", event);
    }
}
