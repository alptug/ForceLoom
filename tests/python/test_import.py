import importlib.metadata
import unittest
import forceloom
from forceloom import _forceloom

class ImportTests(unittest.TestCase):
    def test_version(self):
        self.assertEqual(forceloom.__version__, importlib.metadata.version("forceloom"))
        self.assertEqual(_forceloom.__version__, forceloom.__version__)

if __name__ == "__main__":
    unittest.main()
