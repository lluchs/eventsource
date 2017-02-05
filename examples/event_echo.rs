extern crate eventsource;

use eventsource::curl::Client;

fn main() {
    let url = "https://clonkspot.org/league/game_events.php";
    //let url = "http://league.openclonk.org/poll_game_events.php";
    let client = Client::new(url);
    for event in client {
        println!("{}", event.unwrap());
    }
}
