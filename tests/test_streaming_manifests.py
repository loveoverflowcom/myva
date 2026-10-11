"""Projection contract tests, not runtime or GPU measurements."""
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('streaming_export', ROOT / 'scripts/export-streaming-manifests.py')
exporter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(exporter)


class ManifestProjectionTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((ROOT / 'assets/fixture/manifest.json').read_text())

    def test_checked_in_load_units_exactly_partition_catalog(self):
        seen = set()
        for bundle in self.manifest['bundles']:
            projection = exporter.project(self.manifest, bundle)
            stored = json.loads((ROOT / 'assets' / ('runtime-' + bundle['id'] + '.myva')).read_text())
            self.assertEqual(stored, projection)
            for asset in projection['assets']:
                self.assertNotIn(asset['id'], seen)
                seen.add(asset['id'])
                original = next(a for a in self.manifest['assets'] if a['id'] == asset['id'])
                self.assertEqual(asset['sha256'], original['sha256'])
                self.assertEqual(asset['bytes'], original['bytes'])
                self.assertTrue((ROOT / 'assets' / asset['path']).is_file())
            self.assertEqual(projection['bundles'], [bundle])
        self.assertEqual(seen, {a['id'] for a in self.manifest['assets']})

    def test_projection_does_not_mutate_source(self):
        original = deepcopy(self.manifest)
        exporter.project(self.manifest, self.manifest['bundles'][0])
        self.assertEqual(original, self.manifest)

    def test_cross_bundle_dependencies_require_explicit_expansion(self):
        bundle = deepcopy(self.manifest['bundles'][0])
        bundle['dependencies'] = ['map.dap_co']
        with self.assertRaises(ValueError):
            exporter.project(self.manifest, bundle)

    def test_unknown_asset_rejected(self):
        bundle = deepcopy(self.manifest['bundles'][0])
        bundle['assets'].append('missing.atlas')
        with self.assertRaises(ValueError):
            exporter.project(self.manifest, bundle)


if __name__ == '__main__':
    unittest.main()
