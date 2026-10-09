import { defineConfig } from '@playwright/test';

export default defineConfig({
    testDir: './tests/web',
    workers: 1,
    timeout: 300000,
    expect: { timeout: 90000 },
    reporter: [['list'], ['json', { outputFile: 'target/web-evidence/test-results.json' }]],
    use: {
        baseURL: 'http://127.0.0.1:8080',
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
        command: 'python3 scripts/serve-web-shell.py',
        url: 'http://127.0.0.1:8080/bundle-report.json',
        reuseExistingServer: !process.env.CI,
    },
});
