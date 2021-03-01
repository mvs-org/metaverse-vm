const expect = require("chai").expect;
const assert = require("chai").assert;
const utils = require("./utils");
const conf = require("./config.js");

let currentFilterId = null;

describe("Test filter API", function () {
	before(() => {
		utils.open();
	});

	after(() => {
		utils.close();
	});

	afterEach(async () => {
		if (currentFilterId) {
			const res = await utils.customRequest("eth_uninstallFilter", [currentFilterId]);
		}
		currentFilterId = null;
	});

	it.skip("should return a number as hexstring eth_newBlockFilter", async function () {
		const params = [];
		currentFilterId = await utils.customRequest("eth_newBlockFilter", params);
		assert.isNumber(currentFilterId);
	});

	it.skip("should return a number as hexstring eth_newPendingTransactionFilter", async function () {
		const params = [];
		currentFilterId = await utils.customRequest("eth_newPendingTransactionFilter", params);
		assert.isNumber(currentFilterId);
	});

	it.skip("should return a number as hexstring when all options are passed with single address eth_newFilter", async function () {
		const params = [
			{
				fromBlock: "0x1", // 1
				toBlock: "0x2", // 2
				address: "0xfd9801e0aa27e54970936aa910a7186fdf5549bc",
				topics: ["0x01e0aa27e54970936aa910a713", "0x6aa910a7186fdf"],
			},
		];
		currentFilterId = await utils.customRequest("eth_newFilter", params);
		assert.isNumber(currentFilterId);
	});

	it.skip('should return a number as hexstring when all options are passed with address array', async function () {
		const params = [{
			"fromBlock": "0x1", // 1
			"toBlock": "0x2", // 2
			"address": ["0xfd9801e0aa27e54970936aa910a7186fdf5549bc", "0xab9801e0aa27e54970936aa910a7186fdf5549bc"],
			"topics": ['0x01e0aa27e54970936aa910a713', '0x6aa910a7186fdf']
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		assert.isNumber(currentFilterId);
	});

	it.skip('should return a number as hexstring when all options with "latest" and "pending" for to and fromBlock', async function () {
		const params = [{
			"fromBlock": "latest", // 1
			"toBlock": "pending", // 2
			"address": "0xfd9801e0aa27e54970936aa910a7186fdf5549bc",
			"topics": ['0x01e0aa27e54970936aa910a713', '0x6aa910a7186fdf']
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		assert.isNumber(currentFilterId);
	});

	it.skip('should return a number as hexstring when a few options are passed', async function () {
		const params = [{
			"fromBlock": "0x1", // 1
			"toBlock": "0x2", // 2
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		assert.isNumber(currentFilterId);
	});

	it.skip('should return an error when no parameter is passed', async function () {
		const res = await utils.customRequest('eth_newFilter', []);
		expect(res.error.code, -32602);
	});

	it.skip('should return an error when no parameter is passed', async function () {
		const res = await utils.customRequest('eth_getFilterLogs', []);
		expect(res.error.code, -32602);
	});

	it.skip('should return an error when no parameter is passed', async function () {
		const res = await utils.customRequest('eth_uninstallFilter', []);
		expect(res.error.code, -32602);
	});

	it.skip('should return the correct log, when filtering without defining an address', async function () {
		var params = [{
			"fromBlock": '0x0',
			"toBlock": 'latest'
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		const logs = await utils.customRequest('eth_getFilterLogs');
		assert.isArray(logs);
	});

	it.skip('should return a list of logs, when asking without defining an address and using toBlock "latest"', async function () {
		var params = [{
			"fromBlock": '0x0',
			"toBlock": 'latest'
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		const logs = await utils.customRequest('eth_getFilterLogs');
		assert.isArray(logs);
	});

	it.skip('should return a list of logs, when asking without defining an address and using toBlock "pending"', async function () {
		var params = [{
			"fromBlock": '0x0',
			"toBlock": 'pending'
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		const logs = await utils.customRequest('eth_getFilterLogs');
		assert.isArray(logs);
	});

	it.skip('should return a list of logs, when filtering with defining an address and using toBlock "latest"', async function () {
		expect(conf.jsontestAddress).not.be.empty;
		var params = [{
			"address": conf.jsontestAddress,
			"fromBlock": '0x0',
			"toBlock": 'latest'
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		const logs = await utils.customRequest('eth_getFilterLogs');
		assert.isArray(logs);
	});

	it.skip('should return a list of logs, when filtering with defining an address and using toBlock "pending"', async function () {
		expect(conf.jsontestAddress).not.be.empty;
		var params = [{
			"address": conf.jsontestAddress,
			"fromBlock": '0x0',
			"toBlock": 'pending'
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		const logs = await utils.customRequest('eth_getFilterLogs');
		assert.isArray(logs);
	});

	it.skip('should return a list of logs, when filtering by topic "0x0000000000000000000000000000000000000000000000000000000000000001"', async function () {
		var params = [{
			"fromBlock": '0x0',
			"toBlock": 'latest',
			"topics": ['0x0000000000000000000000000000000000000000000000000000000000000001']
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		const logs = await utils.customRequest('eth_getFilterLogs');
		assert.isArray(logs);
	});

	it.skip('should return a list of anonymous logs, when filtering by topic "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"', async function () {
		var params = [{
			"fromBlock": '0x0',
			"toBlock": 'latest',
			"topics": [null, null, '0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff']
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		const logs = await utils.customRequest('eth_getFilterLogs');
		assert.isArray(logs);
	});

	it.skip('should return a list of logs, when filtering by topic "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"', async function () {
		var params = [{
			"fromBlock": '0x0',
			"toBlock": 'latest',
			"topics": [null, null, null, '0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff']
		}]
		currentFilterId = await utils.customRequest('eth_newFilter', params);
		const logs = await utils.customRequest('eth_getFilterLogs');
		assert.isArray(logs);
	});
});
