use eventsource::reqwest::Client;
use reqwest::Url;

fn main() {
    //let url = "https://clonkspot.org/league/game_events.php";
    let url = "http://league.openclonk.org/poll_game_events.php";
    let client = Client::new(Url::parse(url).unwrap());
    for event in client {
        println!("{}", event.unwrap());
    }
}
