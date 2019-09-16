//		MIT © 2016-2018 Protocol Labs Inc., 2016-2019 Haja Networks Oy
//		https://github.com/orbitdb/ipfs-log

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
