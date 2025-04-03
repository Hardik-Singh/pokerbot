<script lang="ts">
    import type { Card } from './types';
    
    export let card: Card;
    
    const getSuitSymbol = (suit: Card['suit']) => {
        switch (suit) {
            case 'Hearts': return '♥';
            case 'Diamonds': return '♦';
            case 'Clubs': return '♣';
            case 'Spades': return '♠';
            default: return '';
        }
    };
    
    const getRankDisplay = (rank: Card['rank']) => {
        switch (rank) {
            case 'Ace': return 'A';
            case 'King': return 'K';
            case 'Queen': return 'Q';
            case 'Jack': return 'J';
            case 'Ten': return '10';
            case 'Nine': return '9';
            case 'Eight': return '8';
            case 'Seven': return '7';
            case 'Six': return '6';
            case 'Five': return '5';
            case 'Four': return '4';
            case 'Three': return '3';
            case 'Two': return '2';
            default: return '';
        }
    };
    
    $: suitSymbol = getSuitSymbol(card.suit);
    $: rankDisplay = getRankDisplay(card.rank);
    $: isRed = card.suit === 'Hearts' || card.suit === 'Diamonds';
</script>

<div class="card" class:red={isRed}>
    <div class="top">
        <span class="rank">{rankDisplay}</span>
        <span class="suit small">{suitSymbol}</span>
    </div>
    <div class="center suit">{suitSymbol}</div>
    <div class="bottom">
        <span class="rank">{rankDisplay}</span>
        <span class="suit small">{suitSymbol}</span>
    </div>
</div>

<style>
    .card {
        width: 70px;
        height: 100px;
        background: white;
        border-radius: 8px;
        border: 2px solid #ccc;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        padding: 5px;
        font-family: Arial, sans-serif;
        margin: 5px;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        color: black;
        position: relative;
    }
    
    .red {
        color: #d40000;
    }
    
    .top {
        align-self: flex-start;
        display: flex;
        flex-direction: column;
        align-items: center;
        line-height: 1;
    }
    
    .bottom {
        align-self: flex-end;
        display: flex;
        flex-direction: column;
        align-items: center;
        transform: rotate(180deg);
        line-height: 1;
    }
    
    .center {
        font-size: 2em;
        align-self: center;
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }
    
    .rank {
        font-size: 1.2em;
        font-weight: bold;
    }
    
    .suit {
        font-size: 1.2em;
        line-height: 1;
    }
    
    .suit.small {
        font-size: 1em;
    }
</style> 