import test from 'tape'

import { Keypair } from '../index.js'

test('generate keys from a seed', async (t) => {
  const secretkey = Buffer.from('5aa93b299a343aa2691739771f2b5b85e740ca14c685793d67870f88fa89dc51', 'hex')

  const keypair = Keypair.from_secret_key(secretkey)

  const publicKey = keypair.public_key()

  t.is(publicKey.to_string(), 'gcumbhd7sqit6nn457jxmrwqx9pyymqwamnarekgo3xppqo6a19o')
})
