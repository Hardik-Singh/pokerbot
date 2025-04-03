use axum::{
    routing::get,
    Router, Json,
    http::Method,
};
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;
use tower_http::cors::{CorsLayer, Any};
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card {
    suit: Suit,
    rank: Rank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    deck: Vec<Card>,
    player_cards: Vec<Card>,
    opponent_cards: Vec<Card>,
    community_cards: Vec<Card>,
}

impl GameState {
    fn new() -> Self {
        let mut deck = Vec::with_capacity(52);
        for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades].iter() {
            for rank in [
                Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
                Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
            ].iter() {
                deck.push(Card {
                    suit: *suit,
                    rank: *rank,
                });
            }
        }
        
        let mut rng = rand::thread_rng();
        deck.shuffle(&mut rng);

        GameState {
            deck,
            player_cards: Vec::new(),
            opponent_cards: Vec::new(),
            community_cards: Vec::new(),
        }
    }

    fn deal_initial_hands(&mut self) {
        self.player_cards = vec![self.deck.pop().unwrap(), self.deck.pop().unwrap()];
        self.opponent_cards = vec![self.deck.pop().unwrap(), self.deck.pop().unwrap()];
    }

    fn deal_flop(&mut self) {
        for _ in 0..3 {
            self.community_cards.push(self.deck.pop().unwrap());
        }
    }

    fn deal_turn(&mut self) {
        self.community_cards.push(self.deck.pop().unwrap());
    }

    fn deal_river(&mut self) {
        self.community_cards.push(self.deck.pop().unwrap());
    }
}

static GAME_STATE: Lazy<Mutex<Option<GameState>>> = Lazy::new(|| Mutex::new(None));

async fn new_game() -> Json<GameState> {
    println!("Creating new game");
    let mut game = GameState::new();
    game.deal_initial_hands();
    
    let mut state = GAME_STATE.lock().unwrap();
    *state = Some(game.clone());
    println!("New game created with {} cards in deck", game.deck.len());
    println!("Player cards: {:?}", game.player_cards);
    println!("Opponent cards: {:?}", game.opponent_cards);
    
    Json(game)
}

async fn deal_flop() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing flop");
        game.deal_flop();
        println!("Community cards after flop: {:?}", game.community_cards);
        return Json(game.clone());
    }
    println!("No game state found for flop");
    Json(GameState::new())
}

async fn deal_turn() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing turn");
        game.deal_turn();
        println!("Community cards after turn: {:?}", game.community_cards);
        return Json(game.clone());
    }
    println!("No game state found for turn");
    Json(GameState::new())
}

async fn deal_river() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing river");
        game.deal_river();
        println!("Community cards after river: {:?}", game.community_cards);
        return Json(game.clone());
    }
    println!("No game state found for river");
    Json(GameState::new())
}

#[tokio::main]
async fn main() {
    println!("Starting poker server...");
    
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/new-game", get(new_game))
        .route("/deal-flop", get(deal_flop))
        .route("/deal-turn", get(deal_turn))
        .route("/deal-river", get(deal_river))
        .layer(cors);

    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
