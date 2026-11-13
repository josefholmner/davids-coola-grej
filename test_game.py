import unittest
from unittest.mock import patch
from game import GameState, make_addition


class GameRulesTests(unittest.TestCase):
    @patch('game.random.choice', return_value=13)
    def test_matching_addition_scores_and_changes_target(self, _choice):
        state = GameState(target=8)
        self.assertEqual(state.catch(6, 2), 'complete')
        self.assertEqual((state.target, state.score, state.round, state.lives), (13, 80, 2, 3))
        self.assertEqual(state.last_answer, '6 + 2 = 8')

    def test_wrong_addition_loses_life_above_or_below_target(self):
        for operands in [(7, 6), (1, 2)]:
            with self.subTest(operands=operands):
                state = GameState(target=8)
                self.assertEqual(state.catch(*operands), 'wrong')
                self.assertEqual((state.target, state.lives, state.score, state.round), (8, 2, 0, 1))

    def test_separate_additions_do_not_accumulate(self):
        state = GameState(target=8)
        self.assertEqual(state.catch(1, 3), 'wrong')
        self.assertEqual(state.catch(2, 2), 'wrong')
        self.assertEqual((state.lives, state.score), (1, 0))

    def test_generator_can_provide_matches_and_distractors_for_every_target(self):
        for target in range(2, 19):
            for random_value, matches in [(0.0, True), (0.9, False)]:
                with self.subTest(target=target, matches=matches):
                    with patch('game.random.random', return_value=random_value):
                        a, b = make_addition(target)
                    self.assertTrue(1 <= a <= 9 and 1 <= b <= 9)
                    self.assertEqual(a + b == target, matches)


if __name__ == '__main__':
    unittest.main()
