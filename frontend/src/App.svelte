<script lang="ts">
  import { onMount } from 'svelte';
  import Card from './lib/Card.svelte';
  import type { GameState } from './lib/types';

  let gameState: GameState | null = null;
  let gamePhase: 'preflop' | 'flop' | 'turn' | 'river' = 'preflop';
  let error: string | null = null;

  async function startNewGame() {
    try {
      error = null;
      console.log('Starting new game...');
      const response = await fetch('http://localhost:3000/new-game');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('Received game state:', data);
      gameState = data;
      gamePhase = 'preflop';
    } catch (e) {
      console.error('Error starting new game:', e);
      error = e instanceof Error ? e.message : 'Failed to start new game';
    }
  }

  async function dealFlop() {
    try {
      if (gamePhase !== 'preflop') return;
      error = null;
      const response = await fetch('http://localhost:3000/deal-flop');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('Received flop:', data);
      gameState = data;
      gamePhase = 'flop';
    } catch (e) {
      console.error('Error dealing flop:', e);
      error = e instanceof Error ? e.message : 'Failed to deal flop';
    }
  }

  async function dealTurn() {
    try {
      if (gamePhase !== 'flop') return;
      error = null;
      const response = await fetch('http://localhost:3000/deal-turn');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('Received turn:', data);
      gameState = data;
      gamePhase = 'turn';
    } catch (e) {
      console.error('Error dealing turn:', e);
      error = e instanceof Error ? e.message : 'Failed to deal turn';
    }
  }

  async function dealRiver() {
    try {
      if (gamePhase !== 'turn') return;
      error = null;
      const response = await fetch('http://localhost:3000/deal-river');
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('Received river:', data);
      gameState = data;
      gamePhase = 'river';
    } catch (e) {
      console.error('Error dealing river:', e);
      error = e instanceof Error ? e.message : 'Failed to deal river';
    }
  }

  onMount(() => {
    startNewGame();
  });
</script>

<main>
  <h1>Poker Learning Tool</h1>

  <div class="controls">
    <button on:click={startNewGame}>New Game</button>
    <button on:click={dealFlop} disabled={!gameState || gamePhase !== 'preflop'}>Deal Flop</button>
    <button on:click={dealTurn} disabled={!gameState || gamePhase !== 'flop'}>Deal Turn</button>
    <button on:click={dealRiver} disabled={!gameState || gamePhase !== 'turn'}>Deal River</button>
  </div>

  {#if error}
    <div class="error">
      {error}
    </div>
  {/if}

  {#if gameState}
    <div class="game-container">
      <div class="player-cards">
        <h2>Your Cards</h2>
        <div class="cards">
          {#each gameState.player_cards as card}
            <Card {card} />
          {/each}
        </div>
      </div>

      <div class="community-cards">
        <h2>Community Cards</h2>
        <div class="cards">
          {#each gameState.community_cards as card}
            <Card {card} />
          {/each}
        </div>
      </div>

      <div class="opponent-cards">
        <h2>Opponent's Cards</h2>
        <div class="cards">
          {#each gameState.opponent_cards as card}
            <Card {card} />
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <div class="start-prompt">
      Click "New Game" to begin!
    </div>
  {/if}
</main>

<style>
  main {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    text-align: center;
  }

  h1 {
    color: #333;
    margin-bottom: 30px;
  }

  .controls {
    margin-bottom: 30px;
  }

  button {
    margin: 0 10px;
    padding: 10px 20px;
    font-size: 16px;
    cursor: pointer;
    background-color: #4CAF50;
    color: white;
    border: none;
    border-radius: 4px;
  }

  button:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
  }

  .game-container {
    padding: 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
  }

  h2 {
    color: #333;
    margin-bottom: 10px;
  }

  .cards {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    justify-content: center;
  }

  .start-prompt {
    font-size: 1.2em;
    color: #666;
    margin-top: 50px;
  }

  .error {
    color: #d32f2f;
    background-color: #ffebee;
    padding: 10px;
    border-radius: 4px;
    margin: 10px 0;
  }
</style>
