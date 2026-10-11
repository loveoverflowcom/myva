import { defineConfig } from '@playwright/test';

const port = Number(process.env.MYVA_WEB_TEST_PORT || 8080);
if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('Invalid MYVA_WEB_TEST_PORT');
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
    testDir: './tests/web',
    workers: 1,
    timeout: 300000,
    expect: { timeout: 90000 },
    reporter: [['list'], ['json', { outputFile: 'target/web-evidence/test-results.json' }]],
    use: {
        baseURL,
        viewport: { width: 1280, height: 900 },
        screenshot: 'only-on-failure',
        trace: 'retain-on-failure',
        launchOptions: {
            executablePath: process.env.CHROME_PATH || undefined,
            args: ['--enable-unsafe-swiftshader', '--use-angle=swiftshader'],
        },
    },
    outputDir: 'target/web-evidence/artifacts',
    webServer: {
        command: `python3 scripts/serve-web-shell.py --port ${port}`,
        url: `${baseURL}/bundle-report.json`,
        reuseExistingServer: !process.env.CI,
    },
});
