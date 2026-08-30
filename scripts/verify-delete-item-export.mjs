import { createRequire } from 'node:module'
import { resolve } from 'node:path'

const binaryPath = process.argv[2]
if (!binaryPath) {
  throw new Error('Usage: node scripts/verify-delete-item-export.mjs <native-binary>')
}

const require = createRequire(import.meta.url)
const native = require(resolve(binaryPath))
if (typeof native.workshop?.deleteItem !== 'function') {
  throw new Error(`Native binary ${binaryPath} does not export workshop.deleteItem`)
}

console.log(`Verified workshop.deleteItem in ${binaryPath}`)
