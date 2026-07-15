import assert from 'node:assert/strict';
import test from 'node:test';
import { formatSize, getParentPath } from '../app/utils/helpers.js';
import { nextVersion } from '../scripts/release.mjs';

test('formatSize handles invalid and large values', () => {
  assert.equal(formatSize(-1), '0 B');
  assert.equal(formatSize(Number.NaN), '0 B');
  assert.match(formatSize(1024 ** 5), /PB$/);
});

test('getParentPath supports Windows, UNC, and Unix paths', () => {
  assert.equal(getParentPath('C:\\images\\photo.jpg'), 'C:\\images');
  assert.equal(
    getParentPath('\\\\server\\share\\photo.jpg'),
    '\\\\server\\share',
  );
  assert.equal(getParentPath('/home/user/photo.jpg'), '/home/user');
});

test('nextVersion bumps patch, minor, and major releases', () => {
  assert.equal(nextVersion('1.2.3', 'patch'), '1.2.4');
  assert.equal(nextVersion('1.2.3', 'minor'), '1.3.0');
  assert.equal(nextVersion('1.2.3', 'major'), '2.0.0');
  assert.throws(() => nextVersion('1.2.3', 'other'));
});
