import unittest
from unittest.mock import patch
from game import GameState, Difficulty, TREE_FULL, make_addition, level_color


class GameRulesTests(unittest.TestCase):
    def test_correct_addition_grows_tree_and_changes_target(self):
        state = GameState(target=5)
        self.assertEqual(state.catch(2, 3), 'correct')
        self.assertEqual((state.growth, state.score, state.level), (3, 50, 1))
        self.assertNotEqual(state.target, 5)

    def test_full_tree_advances_level(self):
        state = GameState(target=5, growth=TREE_FULL - 1)
        self.assertEqual(state.catch(2, 3), 'level_up')
        self.assertEqual((state.level, state.growth), (2, 2))
        self.assertTrue(state.difficulty.min_target <= state.target <= state.difficulty.max_operand * 2)

    def test_wrong_sum_shrinks_without_accumulating(self):
        state = GameState(target=6, growth=4)
        for _ in range(2):
            self.assertEqual(state.catch(1, 2), 'wrong')
        self.assertEqual((state.growth, state.target, state.score), (2, 6, 0))

    def test_empty_tree_goes_back_and_restores_half_growth(self):
        state = GameState(level=5, growth=1, target=14)
        self.assertEqual(state.catch(1, 1), 'level_down')
        self.assertEqual((state.level, state.growth), (4, TREE_FULL // 2))
        self.assertTrue(state.difficulty.min_target <= state.target <= state.difficulty.max_operand * 2)

    def test_first_level_always_keeps_seedling(self):
        state = GameState(growth=1, target=5)
        for _ in range(20):
            state.catch(1, 1)
        self.assertEqual((state.level, state.growth), (1, 1))

    def test_difficulty_increases_and_early_levels_are_easy(self):
        self.assertEqual([Difficulty(n).max_operand for n in range(1, 4)], [3, 4, 5])
        for level in range(1, 15):
            a, b = Difficulty(level), Difficulty(level + 1)
            self.assertGreater(b.max_operand, a.max_operand)
            self.assertGreater(b.speed, a.speed)
            self.assertLessEqual(b.interval, a.interval)
            self.assertNotEqual(level_color(level), level_color(level + 1))

    def test_additions_work_for_all_targets_and_large_levels(self):
        for level in [1, 2, 3, 8, 25, 1000]:
            difficulty = Difficulty(level)
            targets = range(difficulty.min_target, difficulty.max_operand * 2 + 1) if level < 25 else [difficulty.min_target, difficulty.max_operand * 2]
            for target in targets:
                for value, matches in [(0.0, True), (0.9, False)]:
                    with self.subTest(level=level, target=target, matches=matches):
                        with patch('game.random.random', return_value=value):
                            a, b = make_addition(target, difficulty.max_operand)
                        self.assertTrue(1 <= a <= difficulty.max_operand and 1 <= b <= difficulty.max_operand)
                        self.assertEqual(a + b == target, matches)


if __name__ == '__main__':
    unittest.main()
