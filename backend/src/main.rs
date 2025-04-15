use axum::{
    routing::{get, post},
    Router, Json,
    http::Method,
    extract::Query,
};
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;
use tower_http::cors::{CorsLayer, Any};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::cmp::Ordering;

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
pub struct Player {
    cards: Vec<Card>,
    win_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    deck: Vec<Card>,
    players: Vec<Player>,
    community_cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
pub struct NewGameQuery {
    num_players: usize,
}

impl Card {
    fn value(&self) -> u8 {
        match self.rank {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
            Rank::Ace => 14,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hand {
    hand_type: HandType,
    values: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

fn evaluate_hand(cards: &[Card]) -> Hand {
    let mut values: Vec<u8> = cards.iter().map(|c| c.value()).collect();
    values.sort_unstable_by(|a, b| b.cmp(a));
    
    // Check for flush
    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);
    
    // Check for straight
    let mut is_straight = false;
    if values.windows(2).all(|w| w[0] == w[1] + 1) {
        is_straight = true;
    } else if values == vec![14, 5, 4, 3, 2] {
        // Special case: Ace-low straight
        is_straight = true;
        values = vec![5, 4, 3, 2, 1];
    }
    
    // Count frequencies
    let mut freq = std::collections::HashMap::new();
    for &v in &values {
        *freq.entry(v).or_insert(0) += 1;
    }
    let mut freq_vec: Vec<_> = freq.into_iter().collect();
    freq_vec.sort_by_key(|&(v, c)| (-(c as i32), -(v as i32)));
    
    let hand_type = if is_flush && is_straight {
        HandType::StraightFlush
    } else if freq_vec[0].1 == 4 {
        HandType::FourOfAKind
    } else if freq_vec[0].1 == 3 && freq_vec[1].1 == 2 {
        HandType::FullHouse
    } else if is_flush {
        HandType::Flush
    } else if is_straight {
        HandType::Straight
    } else if freq_vec[0].1 == 3 {
        HandType::ThreeOfAKind
    } else if freq_vec[0].1 == 2 && freq_vec[1].1 == 2 {
        HandType::TwoPair
    } else if freq_vec[0].1 == 2 {
        HandType::Pair
    } else {
        HandType::HighCard
    };
    
    Hand { hand_type, values }
}

fn simulate_win_probability(player_cards: &[Card], other_players_cards: &[Vec<Card>], community_cards: &[Card], remaining_deck: &[Card], num_simulations: usize) -> f64 {
    let mut wins = 0;
    let mut ties = 0;
    let mut rng = rand::thread_rng();
    
    // If no other players, return 100% win probability
    if other_players_cards.is_empty() {
        return 1.0;
    }
    
    // If we don't have enough cards in the deck for simulation, return equal probability
    let total_needed_cards = 5 - community_cards.len(); // cards needed to complete board
    if remaining_deck.len() < total_needed_cards {
        return 1.0 / (other_players_cards.len() as f64 + 1.0);
    }
    
    for _ in 0..num_simulations {
        let mut simulation_deck = remaining_deck.to_vec();
        simulation_deck.shuffle(&mut rng);
        
        // Complete the community cards
        let mut final_community = community_cards.to_vec();
        while final_community.len() < 5 {
            if let Some(card) = simulation_deck.pop() {
                final_community.push(card);
            } else {
                // If we run out of cards, return equal probability
                return 1.0 / (other_players_cards.len() as f64 + 1.0);
            }
        }
        
        // Evaluate all hands
        let mut all_hands: Vec<Hand> = Vec::new();
        
        // Player's hand
        let mut player_all_cards = player_cards.to_vec();
        player_all_cards.extend(final_community.iter().cloned());
        if player_all_cards.len() >= 5 {
            all_hands.push(evaluate_hand(&player_all_cards));
        } else {
            // Not enough cards, return equal probability
            return 1.0 / (other_players_cards.len() as f64 + 1.0);
        }
        
        // Other players' hands
        for other_cards in other_players_cards {
            let mut other_all_cards = other_cards.to_vec();
            other_all_cards.extend(final_community.iter().cloned());
            if other_all_cards.len() >= 5 {
                all_hands.push(evaluate_hand(&other_all_cards));
            } else {
                // Not enough cards, return equal probability
                return 1.0 / (other_players_cards.len() as f64 + 1.0);
            }
        }
        
        // Check if player wins or ties
        let player_hand = &all_hands[0];
        let mut is_win = true;
        let mut is_tie = true;
        
        for other_hand in &all_hands[1..] {
            match other_hand.cmp(player_hand) {
                Ordering::Greater => {
                    is_win = false;
                    is_tie = false;
                    break;
                }
                Ordering::Less => {
                    is_tie = false;
                }
                Ordering::Equal => {
                    is_win = false;
                }
            }
        }
        
        if is_win {
            wins += 1;
        } else if is_tie {
            ties += 1;
        }
    }
    
    if num_simulations > 0 {
        (wins as f64 + (ties as f64 * 0.5)) / num_simulations as f64
    } else {
        1.0 / (other_players_cards.len() as f64 + 1.0) // Equal probability if no simulations
    }
}

impl GameState {
    fn new(num_players: usize) -> Self {
        if num_players < 2 || num_players > 8 {
            panic!("Number of players must be between 2 and 8");
        }

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

        let mut players = Vec::with_capacity(num_players);
        let mut used_cards = Vec::new();
        
        for _ in 0..num_players {
            let card1 = deck.pop().unwrap();
            let card2 = deck.pop().unwrap();
            used_cards.push(card1);
            used_cards.push(card2);
            players.push(Player {
                cards: vec![card1, card2],
                win_probability: 0.0,
            });
        }

        let mut game = GameState {
            deck,
            players,
            community_cards: Vec::new(),
        };
        game.update_probabilities();
        game
    }

    fn update_probabilities(&mut self) {
        const NUM_SIMULATIONS: usize = 1000;
        
        // Early return if no players
        if self.players.is_empty() {
            return;
        }
        
        // Calculate remaining deck (exclude all known cards)
        let mut all_used_cards = Vec::new();
        for player in &self.players {
            all_used_cards.extend(player.cards.iter().cloned());
        }
        all_used_cards.extend(self.community_cards.iter().cloned());
        
        let mut remaining_deck = Vec::new();
        for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades].iter() {
            for rank in [
                Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
                Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
            ].iter() {
                let card = Card { suit: *suit, rank: *rank };
                if !all_used_cards.contains(&card) {
                    remaining_deck.push(card);
                }
            }
        }
        
        // Calculate probabilities for each player
        for i in 0..self.players.len() {
            let player_cards = &self.players[i].cards;
            
            // Skip if player doesn't have exactly 2 cards
            if player_cards.len() != 2 {
                self.players[i].win_probability = 0.0;
                continue;
            }
            
            let mut other_players_cards = Vec::new();
            for (j, player) in self.players.iter().enumerate() {
                if i != j && player.cards.len() == 2 {
                    other_players_cards.push(player.cards.clone());
                }
            }
            
            let prob = simulate_win_probability(
                player_cards,
                &other_players_cards,
                &self.community_cards,
                &remaining_deck,
                NUM_SIMULATIONS
            );
            self.players[i].win_probability = prob;
        }
    }

    fn deal_flop(&mut self) {
        for _ in 0..3 {
            self.community_cards.push(self.deck.pop().unwrap());
        }
        self.update_probabilities();
    }

    fn deal_turn(&mut self) {
        self.community_cards.push(self.deck.pop().unwrap());
        self.update_probabilities();
    }

    fn deal_river(&mut self) {
        self.community_cards.push(self.deck.pop().unwrap());
        self.update_probabilities();
    }
}

static GAME_STATE: Lazy<Mutex<Option<GameState>>> = Lazy::new(|| Mutex::new(None));

async fn new_game(Query(query): Query<NewGameQuery>) -> Json<GameState> {
    println!("Creating new game with {} players", query.num_players);
    let game = GameState::new(query.num_players);
    
    let mut state = GAME_STATE.lock().unwrap();
    *state = Some(game.clone());
    println!("New game created with {} cards in deck", game.deck.len());
    for (i, player) in game.players.iter().enumerate() {
        println!("Player {} cards: {:?} (win probability: {:.1}%)", 
            i + 1, player.cards, player.win_probability * 100.0);
    }
    
    Json(game)
}

async fn deal_flop() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing flop");
        game.deal_flop();
        println!("Community cards after flop: {:?}", game.community_cards);
        for (i, player) in game.players.iter().enumerate() {
            println!("Player {} win probability: {:.1}%", 
                i + 1, player.win_probability * 100.0);
        }
        return Json(game.clone());
    }
    println!("No game state found for flop");
    Json(GameState::new(2))
}

async fn deal_turn() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing turn");
        game.deal_turn();
        println!("Community cards after turn: {:?}", game.community_cards);
        for (i, player) in game.players.iter().enumerate() {
            println!("Player {} win probability: {:.1}%", 
                i + 1, player.win_probability * 100.0);
        }
        return Json(game.clone());
    }
    println!("No game state found for turn");
    Json(GameState::new(2))
}

async fn deal_river() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing river");
        game.deal_river();
        println!("Community cards after river: {:?}", game.community_cards);
        for (i, player) in game.players.iter().enumerate() {
            println!("Player {} win probability: {:.1}%", 
                i + 1, player.win_probability * 100.0);
        }
        return Json(game.clone());
    }
    println!("No game state found for river");
    Json(GameState::new(2))
}

#[tokio::main]
async fn main() {
    println!("Starting poker server...");
    
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/new-game", get(new_game
        ))
        .route("/deal-flop", get(deal_flop))
        .route("/deal-turn", get(deal_turn))
        .route("/deal-river", get(deal_river))
        .layer(cors);

    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
