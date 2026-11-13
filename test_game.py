import unittest
from unittest.mock import patch
from game import GameState


class GameRulesTests(unittest.TestCase):
    def test_partial_sum(self):
        state = GameState(target=15)
        self.assertEqual(state.catch(6), "caught")
        self.assertEqual((state.total, state.lives, state.score, state.caught), (6, 3, 0, [6]))

    @patch("game.random.randint", return_value=20)
    def test_exact_sum_starts_new_round(self, _random):
        state = GameState(target=15)
        state.catch(6)
        self.assertEqual(state.catch(9), "complete")
        self.assertEqual((state.total, state.target, state.score, state.round, state.lives), (0, 20, 150, 2, 3))
        self.assertEqual(state.caught, [])

    def test_overflow_loses_life_and_keeps_target(self):
        state = GameState(target=10)
        state.catch(8)
        self.assertEqual(state.catch(3), "over")
        self.assertEqual((state.total, state.target, state.lives, state.score), (0, 10, 2, 0))
        self.assertEqual(state.caught, [])


if __name__ == "__main__":
    unittest.main()
