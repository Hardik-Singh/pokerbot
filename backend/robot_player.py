import random
from typing import List, Optional, Tuple
from dataclasses import dataclass
from enum import Enum

class ActionType(Enum):
    FOLD = "fold"
    CHECK = "check"
    CALL = "call"
    BET = "bet"
    RAISE = "raise"

@dataclass
class Action:
    action_type: ActionType
    amount: Optional[int] = None

class RobotPlayer:
    def __init__(self, name: str, starting_chips: int):
        self.name = name
        self.chips = starting_chips
        self.cards: List[Tuple[str, str]] = []
        self.win_probability: float = 0.0

    def decide_action(self, current_bet: int, pot: int, min_raise: int) -> Action:
        # Simple random strategy for now
        # Will be replaced with ML model later
        if current_bet == 0:
            # Can check or bet
            if random.random() < 0.7:  # 70% chance to check
                return Action(ActionType.CHECK)
            else:
                # Random bet between 1/4 and 1/2 of pot
                bet_amount = random.randint(pot // 4, pot // 2)
                return Action(ActionType.BET, bet_amount)
        else:
            # Must fold, call, or raise
            if random.random() < 0.3:  # 30% chance to fold
                return Action(ActionType.FOLD)
            elif random.random() < 0.6:  # 60% chance to call
                return Action(ActionType.CALL)
            else:
                # Random raise between min_raise and 2x min_raise
                raise_amount = random.randint(min_raise, min_raise * 2)
                return Action(ActionType.RAISE, raise_amount)

    def update_state(self, cards: List[Tuple[str, str]], win_probability: float):
        self.cards = cards
        self.win_probability = win_probability

def main():
    # This will be used when we implement the robot player as a separate service
    pass

if __name__ == "__main__":
    main() 