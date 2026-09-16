"""Regression checks for release input validation; no real installation or settings."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent

class ReleaseTests(unittest.TestCase):
    def formula(self, directory, url):
        archive = directory / 'archive.tar.gz'
        archive.write_bytes(b'test archive')
        return subprocess.run([sys.executable, str(ROOT / 'scripts/homebrew.py'),
            '--archive', str(archive), '--url', url, '--homepage', 'https://example.com/project',
            '--version', '0.1.0', '--output', str(directory / 'mac-cli.rb')],
            capture_output=True, text=True)

    def test_rejects_interpolation_and_malformed_urls(self):
        for url in ['https://example.com/#{system("id")}', 'https://example.com/#$global',
                    'https://example.com/#@instance', 'https:///missing-host',
                    'https://user:secret@example.com/archive', 'https://example.com/\nfile',
                    'https://example.com/\tfile', 'http://example.com/file']:
            with self.subTest(url=url), tempfile.TemporaryDirectory() as temp:
                directory = Path(temp)
                self.assertNotEqual(self.formula(directory, url).returncode, 0)
                self.assertFalse((directory / 'mac-cli.rb').exists())

    def test_formula_refuses_overwrite(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            self.assertEqual(self.formula(directory, 'https://example.com/archive').returncode, 0)
            before = (directory / 'mac-cli.rb').read_bytes()
            self.assertNotEqual(self.formula(directory, 'https://example.com/other').returncode, 0)
            self.assertEqual((directory / 'mac-cli.rb').read_bytes(), before)

    def test_public_release_requires_credentials_before_build(self):
        environment = dict(os.environ)
        environment.pop('MAC_CLI_SIGN_IDENTITY', None)
        environment.pop('MAC_CLI_NOTARY_PROFILE', None)
        result = subprocess.run(['sh', str(ROOT / 'scripts/package.sh'), '--release'],
                                env=environment, capture_output=True, text=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn('Developer ID Application', result.stderr)
        self.assertNotIn('Compiling', result.stderr)

if __name__ == '__main__':
    unittest.main()
