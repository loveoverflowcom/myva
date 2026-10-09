#!/usr/bin/env python3
"""Same-origin static development server with explicit WASM MIME and CSP."""
import argparse
import functools
import http.server
import pathlib

parser = argparse.ArgumentParser()
parser.add_argument('--host', default='127.0.0.1')
parser.add_argument('--port', type=int, default=8080)
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parent.parent / 'target' / 'web-shell'
for required in ['index.html', 'shell/myva_web_shell_bg.wasm', 'game/pkg/myva_web_game_bg.wasm']:
    if not (root / required).is_file():
        parser.error(f'Missing {required}; run ./scripts/build-web-shell.sh first')


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, '.wasm': 'application/wasm', '.js': 'text/javascript'}

    def end_headers(self):
        self.send_header('Content-Security-Policy', "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'self'")
        # Stable prototype filenames: revalidate, never serve stale WASM against newer JS.
        self.send_header('Cache-Control', 'no-cache')
        self.send_header('X-Content-Type-Options', 'nosniff')
        super().end_headers()


with http.server.ThreadingHTTPServer((args.host, args.port), functools.partial(Handler, directory=str(root))) as server:
    host, port = server.server_address[:2]
    print(f'\nMyVa · web shell\n  Play: http://{host}:{port}/\n  Build: http://{host}:{port}/bundle-report.json\n  Root: {root}\n  Stop: Ctrl+C\n', flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
