const assert = require("chai").assert;
const Web3 = require("web3");
const BigNumber = require("bignumber.js");
const conf = require("./config.js");
var web3;

function customRequest(method, params) {
	return new Promise((resolve, reject) => {
		web3.currentProvider.send(
			{
				method: method,
				params: params,
				jsonrpc: "2.0",
				id: conf.rpcMessageId++,
			},
			function (error, result) {
				if (error) {
					reject(
						`Failed to send custom request (${method} (${params.join(",")})): ${error.message || error.toString()
						}`
					);
				}
				resolve(result);
			}
		);
	});
}

function open() {
	web3 = new Web3(conf.host);
}

function close() {
	web3.currentProvider.disconnect();
}

module.exports = { customRequest, open, close };
