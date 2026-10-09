"""Record reproducible file sizes; gzip is potential transfer size, not observed wire bytes."""
import gzip
import hashlib
import json
import pathlib
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parent.parent
output = pathlib.Path(sys.argv[1])
files = []
for path in sorted(output.rglob('*')):
    if path.is_file() and path.name != 'bundle-report.json':
        data = path.read_bytes()
        files.append({'path': str(path.relative_to(output)), 'bytes': len(data),
                      'gzip_bytes': len(gzip.compress(data, mtime=0)),
                      'sha256': hashlib.sha256(data).hexdigest()})
report = {
    'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
    'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=root)),
    'profile': sys.argv[2],
    'rustc': subprocess.check_output(['rustc', '--version'], cwd=root, text=True).strip(),
    'files': files,
    'total_bytes': sum(item['bytes'] for item in files),
    'total_gzip_bytes': sum(item['gzip_bytes'] for item in files),
}
(output / 'bundle-report.json').write_text(json.dumps(report, indent=2) + '\n')
print(f"Web build: {output}\nTotal: {report['total_bytes']:,} bytes; gzip estimate: {report['total_gzip_bytes']:,} bytes")
