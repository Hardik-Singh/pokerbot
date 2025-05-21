use axum::{
    routing::{get, post},
    Router, Json,
    http::Method,
    extract::{Query, Json as JsonExtractor, State},
};
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;
use tower_http::cors::{CorsLayer, Any, AllowHeaders};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use chrono;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    #[serde(rename = "Hearts")]
    Hearts,
    #[serde(rename = "Diamonds")]
    Diamonds,
    #[serde(rename = "Clubs")]
    Clubs,
    #[serde(rename = "Spades")]
    Spades,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    #[serde(rename = "Two")]
    Two,
    #[serde(rename = "Three")]
    Three,
    #[serde(rename = "Four")]
    Four,
    #[serde(rename = "Five")]
    Five,
    #[serde(rename = "Six")]
    Six,
    #[serde(rename = "Seven")]
    Seven,
    #[serde(rename = "Eight")]
    Eight,
    #[serde(rename = "Nine")]
    Nine,
    #[serde(rename = "Ten")]
    Ten,
    #[serde(rename = "Jack")]
    Jack,
    #[serde(rename = "Queen")]
    Queen,
    #[serde(rename = "King")]
    King,
    #[serde(rename = "Ace")]
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
    chips: u32,
    is_robot: bool,
    name: String,
    current_bet: u32,  // Track current bet for this round
    personality: Option<RobotPersonality>,  // Only for robots
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotPersonality {
    name: String,
    emoji: String,
    style: String,
    description: String,
    aggression: f64,
    bluff_frequency: f64,
    patience: f64,
    risk_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    games_played: u32,
    games_won: u32,
    total_profit: i32,
    biggest_pot: u32,
    best_hand: String,
    favorite_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    players: Vec<PlayerStats>,
    total_hands: u32,
    average_pot: u32,
    biggest_pot: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    deck: Vec<Card>,
    players: Vec<Player>,
    community_cards: Vec<Card>,
    pot: u32,
    current_bet: u32,
    game_mode: GameMode,
    current_player: usize,
    last_action: Option<Action>,
    stats: GameStats,
    hand_history: Vec<HandHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameMode {
    #[serde(rename = "Simulation")]
    Simulation,
    #[serde(rename = "RobotPlay")]
    RobotPlay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    player_index: usize,
    action_type: ActionType,
    amount: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    #[serde(rename = "Fold")]
    Fold,
    #[serde(rename = "Check")]
    Check,
    #[serde(rename = "Call")]
    Call,
    #[serde(rename = "Bet")]
    Bet,
    #[serde(rename = "Raise")]
    Raise,
}

#[derive(Debug, Deserialize)]
pub struct NewGameQuery {
    num_players: usize,
    game_mode: GameMode,
    starting_chips: u32,
}

#[derive(Debug, Deserialize)]
pub struct PlayerAction {
    action_type: ActionType,
    amount: Option<u32>,
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
        let mut player_and_board = player_cards.to_vec();
        player_and_board.extend(final_board.iter().cloned());
        let player_best = evaluate_best_hand(&player_and_board);

        // Evaluate each opponent's best hand.
        let mut all_hands = vec![player_best.clone()];
        for other in other_players_cards {
            let mut other_and_board = other.clone();
            other_and_board.extend(final_board.iter().cloned());
            let other_best = evaluate_best_hand(&other_and_board);
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
    fn new(num_players: usize, game_mode: GameMode, starting_chips: u32) -> Self {
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

        let robot_personalities = vec![
            RobotPersonality {
                name: "PokerBot 3000".to_string(),
                emoji: "🤖".to_string(),
                style: "Calculating".to_string(),
                description: "A cold, calculating machine that plays by the numbers".to_string(),
                aggression: 0.7,
                bluff_frequency: 0.3,
                patience: 0.8,
                risk_tolerance: 0.6,
            },
            RobotPersonality {
                name: "Lucky Larry".to_string(),
                emoji: "🍀".to_string(),
                style: "Lucky".to_string(),
                description: "Always seems to get the cards he needs".to_string(),
                aggression: 0.5,
                bluff_frequency: 0.6,
                patience: 0.4,
                risk_tolerance: 0.8,
            },
            RobotPersonality {
                name: "Bluff Master".to_string(),
                emoji: "🎭".to_string(),
                style: "Deceptive".to_string(),
                description: "Loves to bluff and keep you guessing".to_string(),
                aggression: 0.8,
                bluff_frequency: 0.8,
                patience: 0.3,
                risk_tolerance: 0.9,
            },
            RobotPersonality {
                name: "Safe Sally".to_string(),
                emoji: "🛡️".to_string(),
                style: "Conservative".to_string(),
                description: "Plays it safe and waits for good hands".to_string(),
                aggression: 0.3,
                bluff_frequency: 0.2,
                patience: 0.9,
                risk_tolerance: 0.3,
            },
        ];

        let mut players = Vec::with_capacity(num_players);
        for i in 0..num_players {
            let card1 = deck.pop().expect("Deck should have enough cards");
            let card2 = deck.pop().expect("Deck should have enough cards");
            let is_robot = i > 0;
            
            let personality = if is_robot {
                Some(robot_personalities[i % robot_personalities.len()].clone())
            } else {
                None
            };

            players.push(Player {
                cards: vec![card1, card2],
                win_probability: 0.0,
                chips: starting_chips,
                is_robot,
                name: if is_robot {
                    format!("{} {}", personality.as_ref().unwrap().emoji, personality.as_ref().unwrap().name)
                } else {
                    "You".to_string()
                },
                current_bet: 0,
                personality,
            });
        }

        let mut game = GameState {
            deck,
            players,
            community_cards: Vec::new(),
            pot: 0,
            current_bet: 0,
            game_mode,
            current_player: 0,
            last_action: None,
            stats: GameStats {
                start_time: chrono::Utc::now(),
                end_time: None,
                players: vec![PlayerStats {
                    games_played: 0,
                    games_won: 0,
                    total_profit: 0,
                    biggest_pot: 0,
                    best_hand: String::new(),
                    favorite_action: String::new(),
                }; num_players],
                total_hands: 0,
                average_pot: 0,
                biggest_pot: 0,
            },
            hand_history: Vec::new(),
        };
        game.update_probabilities();
        game
    }

    /// Updates win probabilities for all players based on the current state.
    fn update_probabilities(&mut self) {
        const NUM_SIMULATIONS: usize = 1000;
        // Use the current deck as the remaining deck.
        let remaining_deck = self.deck.clone();

        // First collect all opponent cards for each player
        let opponent_cards: Vec<Vec<Vec<Card>>> = self.players
            .iter()
            .enumerate()
            .map(|(i, _)| {
                self.players
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, p)| p.cards.clone())
                    .collect()
            })
            .collect();

        // Then update probabilities
        for (i, player) in self.players.iter_mut().enumerate() {
            if player.cards.len() != 2 {
                player.win_probability = 0.0;
                continue;
            }

            let prob = simulate_win_probability(
                &player.cards,
                &opponent_cards[i],
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

    fn handle_action(&mut self, action: Action) -> Result<(), String> {
        let player = &mut self.players[action.player_index];
        
        match action.action_type {
            ActionType::Fold => {
                player.cards.clear();
            },
            ActionType::Check => {
                if self.current_bet > 0 {
                    return Err("Cannot check when there's a bet".to_string());
                }
            },
            ActionType::Call => {
                let call_amount = self.current_bet.saturating_sub(player.current_bet);
                if player.chips < call_amount {
                    return Err("Not enough chips to call".to_string());
                }
                player.chips -= call_amount;
                self.pot += call_amount;
                player.current_bet += call_amount;
            },
            ActionType::Bet => {
                let amount = action.amount.ok_or("Bet amount required")?;
                if amount <= self.current_bet {
                    return Err("Bet must be higher than current bet".to_string());
                }
                if player.chips < amount {
                    return Err("Not enough chips to bet".to_string());
                }
                player.chips -= amount;
                self.pot += amount;
                self.current_bet = amount;
                player.current_bet = amount;
            },
            ActionType::Raise => {
                let amount = action.amount.ok_or("Raise amount required")?;
                if amount <= self.current_bet {
                    return Err("Raise must be higher than current bet".to_string());
                }
                if player.chips < amount {
                    return Err("Not enough chips to raise".to_string());
                }
                player.chips -= amount;
                self.pot += amount;
                self.current_bet = amount;
                player.current_bet = amount;
            },
        }

        self.last_action = Some(action.clone());
        
        // Move to next player
        self.current_player = (self.current_player + 1) % self.players.len();
        
        // If it's a robot's turn, make them act
        if self.players[self.current_player].is_robot {
            self.handle_robot_action()?;
        }
        
        self.update_stats(&action);
        
        Ok(())
    }

    fn handle_robot_action(&mut self) -> Result<(), String> {
        let robot = &self.players[self.current_player];
        if !robot.is_robot {
            return Ok(());
        }

        let personality = self.get_robot_personality();
        let mut rng = rand::thread_rng();
        
        let action = if self.current_bet == 0 {
            if rng.gen::<f64>() < (1.0 - personality.aggression) {
                Action {
                    player_index: self.current_player,
                    action_type: ActionType::Check,
                    amount: None,
                }
            } else {
                let bet_amount = (self.pot as f64 * personality.aggression * 0.5) as u32;
                Action {
                    player_index: self.current_player,
                    action_type: ActionType::Bet,
                    amount: Some(bet_amount),
                }
            }
        } else {
            let r = rng.gen::<f64>();
            if r < (1.0 - personality.aggression) * 0.5 {
                Action {
                    player_index: self.current_player,
                    action_type: ActionType::Fold,
                    amount: None,
                }
            } else if r < (1.0 - personality.aggression) {
                Action {
                    player_index: self.current_player,
                    action_type: ActionType::Call,
                    amount: None,
                }
            } else {
                let raise_amount = (self.current_bet as f64 * (1.0 + personality.aggression)) as u32;
                Action {
                    player_index: self.current_player,
                    action_type: ActionType::Raise,
                    amount: Some(raise_amount),
                }
            }
        };

        self.handle_action(action)
    }

    fn get_robot_personality(&self) -> RobotPersonality {
        let personalities = vec![
            RobotPersonality {
                name: "PokerBot 3000".to_string(),
                emoji: "🤖".to_string(),
                style: "Calculating".to_string(),
                description: "A cold, calculating machine that plays by the numbers".to_string(),
                aggression: 0.7,
                bluff_frequency: 0.3,
                patience: 0.8,
                risk_tolerance: 0.6,
            },
            RobotPersonality {
                name: "Lucky Larry".to_string(),
                emoji: "🍀".to_string(),
                style: "Lucky".to_string(),
                description: "Always seems to get the cards he needs".to_string(),
                aggression: 0.5,
                bluff_frequency: 0.6,
                patience: 0.4,
                risk_tolerance: 0.8,
            },
            RobotPersonality {
                name: "Bluff Master".to_string(),
                emoji: "🎭".to_string(),
                style: "Deceptive".to_string(),
                description: "Loves to bluff and keep you guessing".to_string(),
                aggression: 0.8,
                bluff_frequency: 0.8,
                patience: 0.3,
                risk_tolerance: 0.9,
            },
            RobotPersonality {
                name: "Safe Sally".to_string(),
                emoji: "🛡️".to_string(),
                style: "Conservative".to_string(),
                description: "Plays it safe and waits for good hands".to_string(),
                aggression: 0.3,
                bluff_frequency: 0.2,
                patience: 0.9,
                risk_tolerance: 0.3,
            },
        ];

        personalities[self.current_player % personalities.len()].clone()
    }

    fn update_stats(&mut self, action: &Action) {
        if let Some(player) = self.players.get_mut(action.player_index) {
            let stats = &mut self.stats.players[action.player_index];
            stats.games_played += 1;
            
            match action.action_type {
                ActionType::Bet | ActionType::Raise => {
                    stats.favorite_action = "Aggressive".to_string();
                },
                ActionType::Check | ActionType::Call => {
                    stats.favorite_action = "Conservative".to_string();
                },
                _ => {}
            }
            
            if self.pot > stats.biggest_pot {
                stats.biggest_pot = self.pot;
            }
        }
    }

    fn record_action(&mut self, action: &Action) {
        if let Some(current_hand) = self.hand_history.last_mut() {
            current_hand.actions.push(action.clone());
            current_hand.pot_size = self.pot;
            current_hand.community_cards = self.community_cards.clone();
            current_hand.player_cards = self.players.iter()
                .map(|p| p.cards.clone())
                .collect();
        }
    }
    
    fn start_new_hand(&mut self) {
        self.hand_history.push(HandHistory {
            timestamp: chrono::Utc::now(),
            phase: GamePhase::PreFlop,
            actions: Vec::new(),
            pot_size: 0,
            community_cards: Vec::new(),
            player_cards: Vec::new(),
            winner: None,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandHistory {
    timestamp: chrono::DateTime<chrono::Utc>,
    phase: GamePhase,
    actions: Vec<Action>,
    pot_size: u32,
    community_cards: Vec<Card>,
    player_cards: Vec<Vec<Card>>,
    winner: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePhase {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
}

// Global game state wrapped in a Mutex for thread safety.
static GAME_STATE: Lazy<TokioMutex<Option<GameState>>> = Lazy::new(|| TokioMutex::new(None));

/// Endpoint to create a new game.
async fn new_game(Query(query): Query<NewGameQuery>) -> Json<GameState> {
    println!("Creating new game with {} players in {:?} mode", query.num_players, query.game_mode);
    let game = GameState::new(query.num_players, query.game_mode, query.starting_chips);
    {
        let mut state = GAME_STATE.lock().await;
        *state = Some(game.clone());
    }
    println!("Game created successfully");
    Json(game)
}

/// Endpoint to handle player actions
async fn player_action(
    JsonExtractor(action): JsonExtractor<PlayerAction>,
) -> Json<Result<GameState, String>> {
    println!("Received player action: {:?}", action);
    let mut state = GAME_STATE.lock().await;
    if let Some(ref mut game) = *state {
        let action = Action {
            player_index: 0, // Human player is always index 0
            action_type: action.action_type,
            amount: action.amount,
        };
        
        match game.handle_action(action) {
            Ok(_) => {
                println!("Action handled successfully");
                Json(Ok(game.clone()))
            },
            Err(e) => {
                println!("Error handling action: {}", e);
                Json(Err(e))
            },
        }
    } else {
        println!("No active game found");
        Json(Err("No active game".to_string()))
    }
}

/// Endpoint to deal the flop.
async fn deal_flop() -> Json<GameState> {
    let mut state = GAME_STATE.lock().await;
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
    Json(GameState::new(2, GameMode::Simulation, 1000))
}

/// Endpoint to deal the turn.
async fn deal_turn() -> Json<GameState> {
    let mut state = GAME_STATE.lock().await;
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
    Json(GameState::new(2, GameMode::Simulation, 1000))
}

/// Endpoint to deal the river.
async fn deal_river() -> Json<GameState> {
    let mut state = GAME_STATE.lock().await;
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
    Json(GameState::new(2, GameMode::Simulation, 1000))
}

#[tokio::main]
async fn main() {
    println!("Starting poker server...");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(AllowHeaders::any());

    let app = Router::new()
        .route("/new-game", get(new_game))
        .route("/player-action", post(player_action))
        .route("/deal-flop", get(deal_flop))
        .route("/deal-turn", get(deal_turn))
        .route("/deal-river", get(deal_river))
        .layer(cors);

    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
