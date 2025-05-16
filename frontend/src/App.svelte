<script lang="ts">
  import { onMount } from 'svelte';
  import Card from './lib/Card.svelte';
  import type { GameState, Action } from './lib/types';

  let gameState: GameState | null = null;
  let gamePhase: 'preflop' | 'flop' | 'turn' | 'river' | 'showdown' = 'preflop';
  let error: string | null = null;
  let numPlayers = 2;
  let gameMode: 'Simulation' | 'RobotPlay' = 'RobotPlay';
  let startingChips = 1000;
  let betAmount = 0;
  let minRaise = 0;
  let maxBet = 0;
  let showRobotCards = false;

  function updateBetLimits() {
    if (gameState && gameState.players && gameState.players.length > 0) {
      const player = gameState.players[0]; // Human player
      console.log('Updating bet limits:', {
        currentBet: gameState.current_bet,
        playerChips: player.chips,
        pot: gameState.pot
      });
      
      minRaise = Math.max(gameState.current_bet * 2, 10); // Minimum raise of 10
      maxBet = player.chips;
      
      // Ensure betAmount is within valid range
      if (betAmount < minRaise) {
        betAmount = minRaise;
      }
      if (betAmount > maxBet) {
        betAmount = maxBet;
      }
      
      // If betAmount is 0, set it to minRaise
      if (betAmount === 0) {
        betAmount = minRaise;
      }
    } else {
      // Set default values if game state is not ready
      minRaise = 0;
      maxBet = 0;
      betAmount = 0;
    }
  }

  function setQuickBet(percentage: number) {
    if (gameState) {
      betAmount = Math.min(Math.floor(gameState.pot * percentage), maxBet);
    }
  }

  async function startNewGame() {
    try {
      error = null;
      const players = gameMode === 'RobotPlay' ? 2 : numPlayers;
      console.log('Starting new game:', { players, gameMode, startingChips });
      
      const response = await fetch(
        `http://localhost:3000/new-game?num_players=${players}&game_mode=${gameMode}&starting_chips=${startingChips}`
      );
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('New game data:', data);
      
      gameState = data;
      gamePhase = 'preflop';
      updateBetLimits();
      showRobotCards = false;
    } catch (e) {
      console.error('Error starting new game:', e);
      error = e instanceof Error ? e.message : 'Failed to start new game';
    }
  }

  async function handleAction(actionType: Action['action_type'], amount?: number) {
    try {
      error = null;
      console.log('Sending action:', { actionType, amount, currentGameState: gameState });
      
      const response = await fetch('http://localhost:3000/player-action', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ action_type: actionType, amount }),
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const result = await response.json();
      console.log('Received response:', result);
      
      if (result.Ok) {
        console.log('Updating game state with:', result.Ok);
        gameState = result.Ok;
        updateBetLimits();

        // If the round is complete, move to the next phase
        if (gameState.last_action?.action_type === 'Call' || 
            gameState.last_action?.action_type === 'Check') {
          if (gamePhase === 'preflop') {
            await dealFlop();
          } else if (gamePhase === 'flop') {
            await dealTurn();
          } else if (gamePhase === 'turn') {
            await dealRiver();
          } else if (gamePhase === 'river') {
            gamePhase = 'showdown';
            showRobotCards = true;
          }
        }
      } else if (result.Err) {
        throw new Error(result.Err);
      } else {
        throw new Error('Invalid response from server');
      }
    } catch (e) {
      console.error('Error handling action:', e);
      error = e instanceof Error ? e.message : 'Failed to handle action';
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
      Game Mode:
      <select bind:value={gameMode} on:change={startNewGame}>
        <option value="Simulation">Simulation</option>
        <option value="RobotPlay">Play with Robots</option>
      </select>
    </label>

    {#if gameMode === 'Simulation'}
      <label>
        Number of Players:
        <select bind:value={numPlayers} on:change={startNewGame}>
          {#each Array.from({length: 7}, (_, i) => i + 2) as num}
            <option value={num}>{num} Players</option>
          {/each}
        </select>
      </label>
    {/if}

    {#if gameMode === 'RobotPlay'}
      <label>
        Starting Chips:
        <input type="number" bind:value={startingChips} min="100" step="100" on:change={startNewGame} />
      </label>
    {/if}
  </div>

  <div class="controls">
    <button on:click={startNewGame}>New Game</button>
    {#if gameMode === 'Simulation'}
      <button on:click={dealFlop} disabled={!gameState || gamePhase !== 'preflop'}>Deal Flop</button>
      <button on:click={dealTurn} disabled={!gameState || gamePhase !== 'flop'}>Deal Turn</button>
      <button on:click={dealRiver} disabled={!gameState || gamePhase !== 'turn'}>Deal River</button>
    {/if}
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
            <span class="count-value">{gameMode === 'RobotPlay' ? '2' : numPlayers}</span>
          </div>
          <div class="phase-indicator">
            <span class="phase-label">Phase:</span>
            <span class="phase-value">{gamePhase.charAt(0).toUpperCase() + gamePhase.slice(1)}</span>
          </div>
          {#if gameMode === 'RobotPlay'}
            <div class="pot-indicator">
              <span class="pot-label">Pot:</span>
              <span class="pot-value">${gameState.pot}</span>
            </div>
            <div class="current-bet">
              <span class="bet-label">Current Bet:</span>
              <span class="bet-value">${gameState.current_bet}</span>
            </div>
          {/if}
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
          {#if !player.is_robot || gameMode === 'Simulation'}
            <div class="player-section" class:current-player={i === gameState.current_player}>
              <div class="player-header">
                <h3>{player.name}</h3>
                {#if player.is_robot && player.personality}
                  <div class="robot-personality">
                    <div class="style">{player.personality.style}</div>
                    <div class="description">{player.personality.description}</div>
                  </div>
                {/if}
                <div class="player-info">
                  <div class="chips">${player.chips}</div>
                  {#if !player.is_robot || showRobotCards}
                    <div class="win-probability">
                      <div class="probability-value">{(player.win_probability * 100).toFixed(1)}%</div>
                      <div class="probability-label">Win Probability</div>
                    </div>
                  {/if}
                </div>
              </div>
              <div class="player-cards">
                {#if !player.is_robot || showRobotCards}
                  {#each player.cards as card}
                    <Card {card} />
                  {/each}
                {:else}
                  {#each player.cards as _}
                    <div class="card-back">🂠</div>
                  {/each}
                {/if}
              </div>
              {#if gameMode === 'RobotPlay' && player.is_robot && !showRobotCards}
                <div class="robot-status">
                  {#if player.cards.is_empty()}
                    <span class="folded">Folded</span>
                  {:else}
                    <span class="active">Active</span>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
        {/each}
      </div>

      {#if gameMode === 'RobotPlay' && gameState && gameState.players && gameState.players.length > 0}
        <div class="betting-controls">
          <div class="bet-info">
            <div class="bet-limits">
              <span>Min Raise: ${minRaise}</span>
              <span>Max Bet: ${maxBet}</span>
              <span>Current Bet: ${gameState.current_bet}</span>
              <span>Your Chips: ${gameState.players[0].chips}</span>
            </div>
            <div class="quick-bets">
              <button on:click={() => setQuickBet(0.25)}>1/4 Pot</button>
              <button on:click={() => setQuickBet(0.5)}>1/2 Pot</button>
              <button on:click={() => setQuickBet(0.75)}>3/4 Pot</button>
              <button on:click={() => setQuickBet(1)}>Pot</button>
            </div>
          </div>
          
          <div class="bet-input">
            <label>
              Bet Amount:
              <input 
                type="range" 
                bind:value={betAmount} 
                min={minRaise} 
                max={maxBet} 
                step="10"
              />
              <input 
                type="number" 
                bind:value={betAmount} 
                min={minRaise} 
                max={maxBet} 
                step="10"
              />
            </label>
          </div>
          
          <div class="action-buttons">
            <button class="fold" on:click={() => handleAction('Fold')}>Fold</button>
            {#if gameState.current_bet === 0}
              <button class="check" on:click={() => handleAction('Check')}>Check</button>
            {:else}
              <button class="call" on:click={() => handleAction('Call')}>
                Call ${gameState.current_bet}
              </button>
            {/if}
            <button class="bet" on:click={() => handleAction('Bet', betAmount)}>
              Bet ${betAmount}
            </button>
            <button class="raise" on:click={() => handleAction('Raise', betAmount)}>
              Raise to ${betAmount}
            </button>
          </div>
        </div>
      {/if}
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
    display: flex;
    gap: 20px;
    justify-content: center;
    align-items: center;
    color: #2c3e50;
    font-weight: 600;
    font-size: 18px;
  }

  .setup label {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .setup select, .setup input {
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

  .setup input {
    width: 120px;
    cursor: text;
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
    margin-bottom: 30px;
  }

  .player-section {
    background: white;
    padding: 20px;
    border-radius: 12px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    transition: transform 0.2s;
    position: relative;
    margin-bottom: 20px;
  }

  .player-section:hover {
    transform: translateY(-5px);
  }

  .current-player {
    border: 3px solid #4CAF50;
    transform: translateY(-5px);
    box-shadow: 0 4px 12px rgba(76, 175, 80, 0.2);
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
    display: none;
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

  .pot-indicator, .current-bet {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 15px 30px;
    min-width: 180px;
  }

  .pot-label, .bet-label {
    font-weight: 600;
    color: rgba(255, 255, 255, 0.95);
    text-transform: uppercase;
    letter-spacing: 1px;
    font-size: 1em;
  }

  .pot-value, .bet-value {
    font-size: 2.2em;
    color: white;
    font-weight: bold;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .player-info {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
  }

  .chips {
    font-size: 1.2em;
    font-weight: bold;
    color: #4CAF50;
  }

  .betting-controls {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    margin: 20px auto;
    padding: 20px;
    background: white;
    border-radius: 12px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: center;
    width: 90%;
    max-width: 600px;
    z-index: 1000;
  }
  
  .bet-info {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 10px;
  }
  
  .bet-limits {
    display: flex;
    justify-content: space-between;
    font-size: 1.2em;
    color: #333;
    font-weight: 600;
    background: #f5f5f5;
    padding: 10px;
    border-radius: 8px;
  }
  
  .quick-bets {
    display: flex;
    gap: 10px;
    justify-content: center;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }
  
  .quick-bets button {
    padding: 8px 16px;
    font-size: 1em;
    background: #f0f0f0;
    color: #333;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .quick-bets button:hover {
    background: #e0e0e0;
    transform: translateY(-2px);
  }
  
  .bet-input {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 10px;
  }
  
  .bet-input label {
    display: flex;
    flex-direction: column;
    gap: 10px;
    align-items: center;
    font-size: 1.2em;
    font-weight: 600;
    color: #333;
  }
  
  .bet-input input[type="range"] {
    width: 100%;
    height: 20px;
    margin: 10px 0;
    -webkit-appearance: none;
    background: #f0f0f0;
    border-radius: 10px;
    outline: none;
  }
  
  .bet-input input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 20px;
    height: 20px;
    background: #4CAF50;
    border-radius: 50%;
    cursor: pointer;
  }
  
  .bet-input input[type="number"] {
    width: 120px;
    padding: 12px;
    border: 2px solid #4CAF50;
    border-radius: 4px;
    font-size: 16px;
    font-weight: 600;
    text-align: center;
  }
  
  .action-buttons {
    display: flex;
    gap: 10px;
    justify-content: center;
    flex-wrap: wrap;
    width: 100%;
  }
  
  .action-buttons button {
    min-width: 120px;
    padding: 12px 24px;
    font-size: 16px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .action-buttons button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  }
  
  .action-buttons button.fold {
    background-color: #d32f2f;
    color: white;
  }
  
  .action-buttons button.fold:hover {
    background-color: #b71c1c;
  }
  
  .action-buttons button.check {
    background-color: #2196f3;
    color: white;
  }
  
  .action-buttons button.check:hover {
    background-color: #1976d2;
  }
  
  .action-buttons button.call {
    background-color: #4CAF50;
    color: white;
  }
  
  .action-buttons button.call:hover {
    background-color: #388e3c;
  }
  
  .action-buttons button.bet,
  .action-buttons button.raise {
    background-color: #ff9800;
    color: white;
  }
  
  .action-buttons button.bet:hover,
  .action-buttons button.raise:hover {
    background-color: #f57c00;
  }

  .card-back {
    width: 100px;
    height: 140px;
    background: #2c3e50;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 48px;
    color: white;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    position: relative;
    overflow: hidden;
  }

  .card-back::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(45deg, transparent 45%, rgba(255, 255, 255, 0.1) 50%, transparent 55%);
    animation: shine 2s infinite;
  }

  @keyframes shine {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(100%);
    }
  }

  .robot-status {
    margin-top: 10px;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .robot-status .folded {
    color: #d32f2f;
  }

  .robot-status .active {
    color: #4CAF50;
  }

  .player-section.current-player::after {
    content: "Your Turn";
    position: absolute;
    top: -10px;
    left: 50%;
    transform: translateX(-50%);
    background: #4CAF50;
    color: white;
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 0.8em;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .robot-personality {
    font-size: 0.9em;
    color: #666;
    margin: 5px 0;
  }
  
  .robot-personality .style {
    font-weight: bold;
    color: #4CAF50;
  }
  
  .robot-personality .description {
    font-style: italic;
  }
</style>
