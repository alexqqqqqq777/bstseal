const { execSync } = require('child_process');
const path = require('path');

try {
  console.log('Building bstseal-ffi ...');
  execSync('cargo build -p bstseal-ffi --release', { stdio: 'inherit', cwd: path.resolve(__dirname, '..', '..') });
  console.log('bstseal-ffi build complete');
} catch (e) {
  console.error('Failed to build bstseal-ffi');
  process.exit(1);
}
