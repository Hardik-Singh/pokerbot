export enum Suit {
    Hearts = 'Hearts',
    Diamonds = 'Diamonds',
    Clubs = 'Clubs',
    Spades = 'Spades',
}

export enum Rank {
    Two = 'Two',
    Three = 'Three',
    Four = 'Four',
    Five = 'Five',
    Six = 'Six',
    Seven = 'Seven',
    Eight = 'Eight',
    Nine = 'Nine',
    Ten = 'Ten',
    Jack = 'Jack',
    Queen = 'Queen',
    King = 'King',
    Ace = 'Ace',
}

export interface Card {
    suit: 'Hearts' | 'Diamonds' | 'Clubs' | 'Spades';
    rank: 'Two' | 'Three' | 'Four' | 'Five' | 'Six' | 'Seven' | 'Eight' | 'Nine' | 'Ten' | 'Jack' | 'Queen' | 'King' | 'Ace';
}

export interface Player {
    cards: Card[];
    win_probability: number;
    chips: number;
    is_robot: boolean;
    name: string;
}

export interface GameState {
    deck: Card[];
    players: Player[];
    community_cards: Card[];
    pot: number;
    current_bet: number;
    game_mode: 'Simulation' | 'RobotPlay';
    current_player: number;
    last_action: Action | null;
}

export interface Action {
    player_index: number;
    action_type: 'Fold' | 'Check' | 'Call' | 'Bet' | 'Raise';
    amount?: number;
}

export interface NewGameQuery {
    num_players: number;
    game_mode: 'Simulation' | 'RobotPlay';
    starting_chips: number;
} 