"""Documentation navigation must reject missing fragments without rejecting real ones."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import check_docs


class MarkdownAnchorTests(unittest.TestCase):
    def test_unicode_inline_code_and_duplicate_headings(self):
        text = '# IPv6: `manual`\n## Настройка DNS\n## Настройка DNS\n'
        self.assertEqual(check_docs.markdown_anchors(text),
                         {'ipv6-manual', 'настройка-dns', 'настройка-dns-1'})

    def test_fences_are_not_headings_and_html_anchors_work(self):
        text = '```md\n# Fake\n<a id="fake">\n```\n`<a id="inline-fake">`\n<a id="manual"></a>\nTitle\n=====\n'
        self.assertEqual(check_docs.markdown_anchors(text), {'manual', 'title'})

    def test_explicit_id_does_not_consume_heading_duplicate_suffix(self):
        text = '<a id="manual"></a>\n# Manual\n# Manual\n'
        self.assertEqual(check_docs.markdown_anchors(text), {'manual', 'manual-1'})

    def test_missing_and_valid_local_fragments(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            page = root / 'page.md'
            target = root / 'target.md'
            target.write_text('# Проверка DNS\n', encoding='utf8')
            page.write_text('# Here\n[valid](target.md#%D0%BF%D1%80%D0%BE%D0%B2%D0%B5%D1%80%D0%BA%D0%B0-dns)\n'
                            '[self](#here)\n[broken](target.md#removed)\n', encoding='utf8')
            with patch.object(check_docs, 'ROOT', root), patch.object(check_docs, 'failures', []):
                check_docs.check_links([page])
                self.assertEqual(len(check_docs.failures), 1)
                self.assertIn('target.md#removed: missing Markdown anchor', check_docs.failures[0])

    def test_release_body_can_link_to_its_own_heading(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'release').mkdir()
            page = root / 'release' / 'RELEASE_NOTES_0.8.2.md'
            page.write_text('# English\n[English](#english)\n', encoding='utf8')
            with patch.object(check_docs, 'ROOT', root), patch.object(check_docs, 'failures', []):
                check_docs.check_links([page])
                self.assertEqual(check_docs.failures, [])


if __name__ == '__main__':
    unittest.main()
