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

/// Represents a 5-card hand with an evaluation (hand type) and the card values used for tie-breaking.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Hand {
    hand_type: HandType,
    values: Vec<u8>,
}

/// Enumeration of poker hand types.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

/// Evaluates a 5-card hand.
fn evaluate_hand(cards: &[Card]) -> Hand {
    let mut values: Vec<u8> = cards.iter().map(|c| c.value()).collect();
    values.sort_unstable_by(|a, b| b.cmp(a));

    // Check flush (all cards have the same suit)
    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);

    // Check straight (sequential values)
    let mut is_straight = false;
    if values.windows(2).all(|w| w[0] == w[1] + 1) {
        is_straight = true;
    } else if values == vec![14, 5, 4, 3, 2] {
        // Special case for Ace-low straight
        is_straight = true;
        values = vec![5, 4, 3, 2, 1];
    }

    // Count frequencies of card values
    let mut freq = std::collections::HashMap::new();
    for &v in &values {
        *freq.entry(v).or_insert(0) += 1;
    }
    let mut freq_vec: Vec<_> = freq.into_iter().collect();
    freq_vec.sort_by_key(|&(v, count)| (-(count as i32), -(v as i32)));

    let hand_type = if is_flush && is_straight {
        HandType::StraightFlush
    } else if freq_vec[0].1 == 4 {
        HandType::FourOfAKind
    } else if freq_vec[0].1 == 3 && freq_vec.get(1).map_or(0, |&(_, c)| c) == 2 {
        HandType::FullHouse
    } else if is_flush {
        HandType::Flush
    } else if is_straight {
        HandType::Straight
    } else if freq_vec[0].1 == 3 {
        HandType::ThreeOfAKind
    } else if freq_vec[0].1 == 2 && freq_vec.get(1).map_or(0, |&(_, c)| c) == 2 {
        HandType::TwoPair
    } else if freq_vec[0].1 == 2 {
        HandType::Pair
    } else {
        HandType::HighCard
    };

    Hand { hand_type, values }
}

/// Generates all combinations of `k` items from a slice.
fn combinations<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.len() < k {
        return vec![];
    }

    let mut result = Vec::new();
    // For each index, combine the current item with all combinations of the remaining items.
    for (i, item) in items.iter().enumerate() {
        let rest_combos = combinations(&items[i + 1..], k - 1);
        for mut combo in rest_combos {
            let mut new_combo = vec![item.clone()];
            new_combo.append(&mut combo);
            result.push(new_combo);
        }
    }
    result
}

/// Evaluates the best possible 5-card hand out of a collection of cards.
fn evaluate_best_hand(cards: &[Card]) -> Hand {
    assert!(cards.len() >= 5, "At least 5 cards are required to evaluate a hand");
    if cards.len() == 5 {
        return evaluate_hand(cards);
    }
    combinations(cards, 5)
        .into_iter()
        .map(|combo| evaluate_hand(&combo))
        .max()
        .unwrap()
}

/// Simulates the win probability of a player's hand against opponents using Monte Carlo simulation.
/// It completes the community board with cards drawn from the remaining deck, then
/// evaluates every player's best hand and awards the win fraction when a tie occurs.
fn simulate_win_probability(
    player_cards: &[Card],
    other_players_cards: &[Vec<Card>],
    community_cards: &[Card],
    remaining_deck: &[Card],
    num_simulations: usize,
) -> f64 {
    // If there are no opponents, the win probability is 100%.
    if other_players_cards.is_empty() {
        return 1.0;
    }

    let total_needed = 5usize.saturating_sub(community_cards.len());
    if remaining_deck.len() < total_needed {
        return 1.0 / (other_players_cards.len() as f64 + 1.0);
    }

    let mut total_win = 0.0;
    let mut rng = rand::thread_rng();

    for _ in 0..num_simulations {
        let mut sim_deck = remaining_deck.to_vec();
        sim_deck.shuffle(&mut rng);

        // Complete the community board.
        let mut final_board = community_cards.to_vec();
        final_board.extend(sim_deck.into_iter().take(total_needed));

        // Evaluate best hand for the player.
        let player_best = evaluate_best_hand(&[player_cards, &final_board].concat());

        // Evaluate each opponent's best hand.
        let mut all_hands = vec![player_best.clone()];
        for other in other_players_cards {
            let other_best = evaluate_best_hand(&[other, &final_board].concat());
            all_hands.push(other_best);
        }

        // Identify the maximum hand and count how many players achieved it.
        if let Some(max_hand) = all_hands.iter().max() {
            let tie_count = all_hands.iter().filter(|&hand| hand == max_hand).count() as f64;
            if player_best == *max_hand {
                total_win += 1.0 / tie_count;
            }
        }
    }

    total_win / num_simulations as f64
}

impl GameState {
    /// Creates a new game with the specified number of players (between 2 and 8).
    fn new(num_players: usize) -> Self {
        if num_players < 2 || num_players > 8 {
            panic!("Number of players must be between 2 and 8");
        }

        // Generate a full 52-card deck.
        let mut deck = Vec::with_capacity(52);
        for &suit in &[Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
            for &rank in &[
                Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
                Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
            ] {
                deck.push(Card { suit, rank });
            }
        }

        let mut rng = rand::thread_rng();
        deck.shuffle(&mut rng);

        let mut players = Vec::with_capacity(num_players);
        // Deal 2 cards per player.
        for _ in 0..num_players {
            let card1 = deck.pop().expect("Deck should have enough cards");
            let card2 = deck.pop().expect("Deck should have enough cards");
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

    /// Updates win probabilities for all players based on the current state.
    fn update_probabilities(&mut self) {
        const NUM_SIMULATIONS: usize = 1000;
        // Use the current deck as the remaining deck.
        let remaining_deck = self.deck.clone();

        for (i, player) in self.players.iter_mut().enumerate() {
            if player.cards.len() != 2 {
                player.win_probability = 0.0;
                continue;
            }

            // Gather opponents' cards.
            let other_cards: Vec<Vec<Card>> = self.players
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, p)| p.cards.clone())
                .collect();

            let prob = simulate_win_probability(
                &player.cards,
                &other_cards,
                &self.community_cards,
                &remaining_deck,
                NUM_SIMULATIONS,
            );
            player.win_probability = prob;
        }
    }

    /// Deals the flop (3 community cards) and updates probabilities.
    fn deal_flop(&mut self) {
        for _ in 0..3 {
            if let Some(card) = self.deck.pop() {
                self.community_cards.push(card);
            }
        }
        self.update_probabilities();
    }

    /// Deals the turn (1 community card) and updates probabilities.
    fn deal_turn(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.community_cards.push(card);
        }
        self.update_probabilities();
    }

    /// Deals the river (1 community card) and updates probabilities.
    fn deal_river(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.community_cards.push(card);
        }
        self.update_probabilities();
    }
}

// Global game state wrapped in a Mutex for thread safety.
static GAME_STATE: Lazy<Mutex<Option<GameState>>> = Lazy::new(|| Mutex::new(None));

/// Endpoint to create a new game.
async fn new_game(Query(query): Query<NewGameQuery>) -> Json<GameState> {
    println!("Creating new game with {} players", query.num_players);
    let game = GameState::new(query.num_players);
    {
        let mut state = GAME_STATE.lock().unwrap();
        *state = Some(game.clone());
    }
    for (i, player) in game.players.iter().enumerate() {
        println!("Player {} cards: {:?} (win probability: {:.1}%)", 
            i + 1, player.cards, player.win_probability * 100.0);
    }
    Json(game)
}

/// Endpoint to deal the flop.
async fn deal_flop() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing flop");
        game.deal_flop();
        println!("Community cards: {:?}", game.community_cards);
        for (i, player) in game.players.iter().enumerate() {
            println!("Player {} win probability: {:.1}%", 
                i + 1, player.win_probability * 100.0);
        }
        return Json(game.clone());
    }
    Json(GameState::new(2))
}

/// Endpoint to deal the turn.
async fn deal_turn() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing turn");
        game.deal_turn();
        println!("Community cards: {:?}", game.community_cards);
        for (i, player) in game.players.iter().enumerate() {
            println!("Player {} win probability: {:.1}%", 
                i + 1, player.win_probability * 100.0);
        }
        return Json(game.clone());
    }
    Json(GameState::new(2))
}

/// Endpoint to deal the river.
async fn deal_river() -> Json<GameState> {
    let mut state = GAME_STATE.lock().unwrap();
    if let Some(ref mut game) = *state {
        println!("Dealing river");
        game.deal_river();
        println!("Community cards: {:?}", game.community_cards);
        for (i, player) in game.players.iter().enumerate() {
            println!("Player {} win probability: {:.1}%", 
                i + 1, player.win_probability * 100.0);
        }
        return Json(game.clone());
    }
    Json(GameState::new(2))
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
