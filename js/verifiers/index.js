'use strict'

const verifiers = {
  'v0': require('./verifierv0'),
  'v1': require('./verifierv1')
}

module.exports = {
  verifier: (v) => {
    return verifiers[v]
  }
}
