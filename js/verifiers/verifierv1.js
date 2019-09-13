'use strict'
const crypto = require('libp2p-crypto')
const Buffer = require('safe-buffer/').Buffer

module.exports = {
  verify: async (signature, publicKey, data) => {
    if (!signature) {
      throw new Error('No signature given')
    }
    if (!publicKey) {
      throw new Error('Given publicKey was undefined')
    }
    if (!data) {
      throw new Error('Given input data was undefined')
    }

    if (!Buffer.isBuffer(data)) {
      data = Buffer.from(data)
    }

    const isValid = (key, msg, sig) => new Promise((resolve, reject) => {
      key.verify(msg, sig, (err, valid) => {
        if (!err) {
          resolve(valid)
        }
        reject(valid)
      })
    })

    let res = false
    try {
      const pubKey = crypto.keys.supportedKeys.secp256k1.unmarshalSecp256k1PublicKey(Buffer.from(publicKey, 'hex'))
      res = await isValid(pubKey, data, Buffer.from(signature, 'hex'))
    } catch (e) {
      // Catch error: sig length wrong
    }
    return Promise.resolve(res)
  }
}
