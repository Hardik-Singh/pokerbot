<script lang="ts">
  import { onMount } from 'svelte';
  import Card from './lib/Card.svelte';
  import type { GameState } from './lib/types';

  let gameState: GameState | null = null;
  let gamePhase: 'preflop' | 'flop' | 'turn' | 'river' = 'preflop';
  let error: string | null = null;
  let numPlayers = 2;

  async function startNewGame() {
    try {
      error = null;
      const response = await fetch(`http://localhost:3000/new-game?num_players=${numPlayers}`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
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
  <h1>Poker Game</h1>

  <div class="setup">
    <label>
      Number of Players:
      <select bind:value={numPlayers} on:change={startNewGame}>
        {#each Array.from({length: 7}, (_, i) => i + 2) as num}
          <option value={num}>{num} Players</option>
        {/each}
      </select>
    </label>
  </div>

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
      <div class="game-header">
        <div class="game-info">
          <div class="player-count">
            <span class="count-label">Players:</span>
            <span class="count-value">{numPlayers}</span>
          </div>
          <div class="phase-indicator">
            <span class="phase-label">Phase:</span>
            <span class="phase-value">{gamePhase.charAt(0).toUpperCase() + gamePhase.slice(1)}</span>
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
      </div>

      <div class="players-grid">
        {#each gameState.players as player, i}
          <div class="player-section" class:current-player={i === 0}>
            <div class="player-header">
              <h3>Player {i + 1}</h3>
              <div class="win-probability">
                <div class="probability-value">{(player.win_probability * 100).toFixed(1)}%</div>
                <div class="probability-label">Win Probability</div>
              </div>
            </div>
            <div class="player-cards">
              {#each player.cards as card}
                <Card {card} />
              {/each}
            </div>
          </div>
        {/each}
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
    max-width: 1400px;
    margin: 0 auto;
    padding: 20px;
    text-align: center;
    background-color: #f8f9fa;
    min-height: 100vh;
  }

  h1 {
    color: #2c3e50;
    margin-bottom: 30px;
    font-size: 2.5em;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 2px;
  }

  .setup {
    margin-bottom: 30px;
    padding: 15px;
    display: inline-block;
    color: #2c3e50;
    font-weight: 600;
    font-size: 18px;
  }

  .setup select {
    margin-left: 10px;
    padding: 12px 20px;
    font-size: 18px;
    border-radius: 8px;
    border: 2px solid #4CAF50;
    background-color: white;
    cursor: pointer;
    transition: all 0.2s;
    color: #2c3e50;
    font-weight: 600;
  }

  .setup select:hover {
    border-color: #45a049;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .setup select:focus {
    outline: none;
    border-color: #2e7d32;
    box-shadow: 0 0 0 3px rgba(76, 175, 80, 0.2);
  }

  .controls {
    margin-bottom: 30px;
    display: flex;
    justify-content: center;
    gap: 15px;
  }

  button {
    padding: 12px 24px;
    font-size: 16px;
    cursor: pointer;
    background-color: #4CAF50;
    color: white;
    border: none;
    border-radius: 6px;
    transition: all 0.2s;
    text-transform: uppercase;
    letter-spacing: 1px;
    font-weight: 600;
  }

  button:hover {
    background-color: #45a049;
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  }

  button:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  .game-container {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 30px;
  }

  .game-header {
    display: flex;
    flex-direction: column;
    gap: 20px;
    align-items: center;
  }

  .game-info {
    display: flex;
    gap: 40px;
    justify-content: center;
    background: linear-gradient(145deg, #1b5e20, #2e7d32);
    padding: 25px;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    margin-bottom: 20px;
  }

  .player-count, .phase-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 15px 30px;
    min-width: 180px;
  }

  .count-label, .phase-label {
    font-weight: 600;
    color: rgba(255, 255, 255, 0.95);
    text-transform: uppercase;
    letter-spacing: 1px;
    font-size: 1em;
  }

  .count-value, .phase-value {
    font-size: 2.2em;
    color: white;
    font-weight: bold;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .community-cards {
    background: linear-gradient(145deg, #1b5e20, #2e7d32);
    padding: 25px;
    border-radius: 15px;
    width: 100%;
    max-width: 900px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .community-cards h2 {
    color: white;
    margin-bottom: 20px;
    font-size: 1.8em;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .players-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 25px;
    width: 100%;
    padding: 0 10px;
  }

  .player-section {
    background: white;
    padding: 20px;
    border-radius: 12px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    transition: transform 0.2s;
  }

  .player-section:hover {
    transform: translateY(-5px);
  }

  .current-player {
    border: 3px solid #4CAF50;
  }

  .player-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
    padding-bottom: 15px;
    border-bottom: 2px solid #f0f0f0;
  }

  h3 {
    color: #2c3e50;
    margin: 0;
    font-size: 1.4em;
    font-weight: bold;
  }

  .cards {
    display: flex;
    gap: 15px;
    flex-wrap: wrap;
    justify-content: center;
  }

  .player-cards {
    display: flex;
    gap: 15px;
    justify-content: center;
    min-height: 120px;
    padding: 10px;
    background-color: #f8f9fa;
    border-radius: 8px;
  }

  .win-probability {
    text-align: right;
  }

  .probability-value {
    font-size: 1.4em;
    font-weight: bold;
    color: #4CAF50;
  }

  .probability-label {
    font-size: 0.8em;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .start-prompt {
    font-size: 1.4em;
    color: #666;
    margin-top: 50px;
    padding: 30px;
    background-color: white;
    border-radius: 10px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .error {
    color: #d32f2f;
    background-color: #ffebee;
    padding: 15px;
    border-radius: 8px;
    margin: 15px 0;
    font-weight: 500;
    box-shadow: 0 2px 4px rgba(211, 47, 47, 0.1);
  }
</style>
