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
    suit: Suit;
    rank: Rank;
}

export interface GameState {
    deck: Card[];
    player_cards: Card[];
    opponent_cards: Card[];
    community_cards: Card[];
} 