/**
 * Simple test server that serves both the test app and the SDK
 * - /        -> e2e/test-app/
 * - /sdk/    -> ../web-client/dist/
 */
const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 8081;

const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.mjs': 'application/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.wasm': 'application/wasm',
  '.map': 'application/json',
};

const TEST_APP_DIR = path.join(__dirname, 'test-app');
const SDK_DIR = path.join(__dirname, '..', '..', 'web-client', 'dist');

const server = http.createServer((req, res) => {
  let filePath;
  let requestPath = req.url.split('?')[0]; // Remove query strings

  // Set CORS and security headers
  res.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
  res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
  res.setHeader('Access-Control-Allow-Origin', '*');

  if (requestPath.startsWith('/sdk/')) {
    // Serve from web-client dist
    const sdkPath = requestPath.replace('/sdk/', '');
    filePath = path.join(SDK_DIR, sdkPath);
  } else {
    // Serve from test-app
    if (requestPath === '/') {
      requestPath = '/index.html';
    }
    filePath = path.join(TEST_APP_DIR, requestPath);
  }

  // Security: prevent directory traversal
  if (!filePath.startsWith(TEST_APP_DIR) && !filePath.startsWith(SDK_DIR)) {
    res.writeHead(403);
    res.end('Forbidden');
    return;
  }

  const ext = path.extname(filePath).toLowerCase();
  const contentType = MIME_TYPES[ext] || 'application/octet-stream';

  fs.readFile(filePath, (err, content) => {
    if (err) {
      if (err.code === 'ENOENT') {
        console.log(`404: ${requestPath} -> ${filePath}`);
        res.writeHead(404);
        res.end(`Not found: ${requestPath}`);
      } else {
        console.error(`Error reading ${filePath}:`, err);
        res.writeHead(500);
        res.end('Internal server error');
      }
    } else {
      res.writeHead(200, { 'Content-Type': contentType });
      res.end(content);
    }
  });
});

server.listen(PORT, () => {
  console.log(`Test server running at http://localhost:${PORT}`);
  console.log(`  Test app: ${TEST_APP_DIR}`);
  console.log(`  SDK: ${SDK_DIR}`);
});
