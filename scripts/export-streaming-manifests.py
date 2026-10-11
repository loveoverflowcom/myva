#!/usr/bin/env python3
"""Export deterministic per-bundle load units without copying D05-A art bytes."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def project(manifest, bundle):
    if bundle['dependencies']:
        raise ValueError('Fixture bundles must be self-contained; expand dependencies before export')
    result = deepcopy(manifest)
    ids = set(bundle['assets'])
    result['assets'] = [deepcopy(asset) for asset in manifest['assets'] if asset['id'] in ids]
    if {asset['id'] for asset in result['assets']} != ids:
        raise ValueError('Unknown bundle asset')
    for asset in result['assets']:
        asset['path'] = 'fixture/' + asset['path']
    result['bundles'] = [deepcopy(bundle)]
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    manifest = json.loads((ROOT / 'assets/fixture/manifest.json').read_text())
    failures = []
    for bundle in manifest['bundles']:
        path = ROOT / 'assets' / ('runtime-' + bundle['id'] + '.myva')
        encoded = (json.dumps(project(manifest, bundle), ensure_ascii=False, indent=2) + '\n').encode()
        if args.check:
            if not path.exists() or path.read_bytes() != encoded:
                failures.append(str(path.relative_to(ROOT)))
        else:
            path.write_bytes(encoded)
    if failures:
        parser.exit(1, 'FAIL: stale/missing runtime projections: ' + ', '.join(failures) + '\n')
    print('PASS: four runtime manifests match the D05-A catalog' if args.check else 'Exported runtime manifests')


if __name__ == '__main__':
    main()
