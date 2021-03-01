const _ = require("underscore");

const config = {
	rpcMessageId: 1,
	host: "ws://localhost:9944",
	address: "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b",
	privKey: "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342", // Genesis private key
	gas: "4294967295",
	bytecode:
		"0x608060405234801561001057600080fd5b506108db806100206000396000f3fe608060405234801561001057600080fd5b50600436106101585760003560e01c80639a19a953116100c3578063d2282dc51161007c578063d2282dc514610381578063e30081a0146103af578063e8beef5b146103f3578063f38b0600146103fd578063f5b53e1714610407578063fd4087671461042557610158565b80639a19a953146102d65780639dc2c8f514610307578063a53b1c1e14610311578063a67808571461033f578063b61c050314610349578063c2b12a731461035357610158565b806338cc48311161011557806338cc48311461022c5780634e7ad3671461027657806357cb2fc41461028057806365538c73146102a457806368895979146102ae57806376bc21d9146102cc57610158565b8063102accc11461015d57806312a7b914146101675780631774e646146101895780631e26fd33146101ba5780631f903037146101ea578063343a875d14610208575b600080fd5b61016561042f565b005b61016f610484565b604051808215151515815260200191505060405180910390f35b6101b86004803603602081101561019f57600080fd5b81019080803560ff16906020019092919050505061049a565b005b6101e8600480360360208110156101d057600080fd5b810190808035151590602001909291905050506104b8565b005b6101f26104d4565b6040518082815260200191505060405180910390f35b6102106104de565b604051808260ff1660ff16815260200191505060405180910390f35b6102346104f4565b604051808273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390f35b61027e61051e565b005b61028861053b565b604051808260000b60000b815260200191505060405180910390f35b6102ac610551565b005b6102b661058b565b6040518082815260200191505060405180910390f35b6102d4610595565b005b610305600480360360208110156102ec57600080fd5b81019080803560000b90602001909291905050506105c9565b005b61030f6105ea565b005b61033d6004803603602081101561032757600080fd5b810190808035906020019092919050505061066d565b005b610347610677565b005b610351610690565b005b61037f6004803603602081101561036957600080fd5b81019080803590602001909291905050506106ce565b005b6103ad6004803603602081101561039757600080fd5b81019080803590602001909291905050506106d8565b005b6103f1600480360360208110156103c557600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506106e2565b005b6103fb610726565b005b61040561077e565b005b61040f6107f7565b6040518082815260200191505060405180910390f35b61042d610801565b005b3373ffffffffffffffffffffffffffffffffffffffff16600115157f0e216b62efbb97e751a2ce09f607048751720397ecfb9eef1e48a6644948985b602a6040518082815260200191505060405180910390a3565b60008060009054906101000a900460ff16905090565b80600060026101000a81548160ff021916908360ff16021790555050565b806000806101000a81548160ff02191690831515021790555050565b6000600454905090565b60008060029054906101000a900460ff16905090565b6000600360009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b60011515602a6040518082815260200191505060405180910390a1565b60008060019054906101000a900460000b905090565b7f65c9ac8011e286e89d02a269890f41d67ca2cc597b2c76c7c69321ff492be580602a6040518082815260200191505060405180910390a1565b6000600254905090565b3373ffffffffffffffffffffffffffffffffffffffff1660011515602a6040518082815260200191505060405180910390a2565b80600060016101000a81548160ff021916908360000b60ff16021790555050565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60001b3373ffffffffffffffffffffffffffffffffffffffff16600115157fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe9602a604051808360000b81526020018281526020019250505060405180910390a3565b8060018190555050565b602a6040518082815260200191505060405180910390a0565b600115157f81933b308056e7e85668661dcd102b1f22795b4431f9cf4625794f381c271c6b602a6040518082815260200191505060405180910390a2565b8060048190555050565b8060028190555050565b80600360006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60001b3373ffffffffffffffffffffffffffffffffffffffff1660011515602a6040518082815260200191505060405180910390a3565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60001b3373ffffffffffffffffffffffffffffffffffffffff16600115157f317b31292193c2a4f561cc40a95ea0d97a2733f14af6d6d59522473e1f3ae65f602a6040518082815260200191505060405180910390a4565b6000600154905090565b7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60001b3373ffffffffffffffffffffffffffffffffffffffff16600115157fd5f0a30e4be0c6be577a71eceb7464245a796a7e6a55c0d971837b250de05f4e7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe9602a604051808360000b81526020018281526020019250505060405180910390a456fea2646970667358221220577f07990960d4d95c9523bf1d85b8b6d97ccbb8ff276b695dce59e5fcaf619b64736f6c63430006000033",
	abi: [
		{
			inputs: [],
			stateMutability: "nonpayable",
			type: "constructor",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log0",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log0Anonym",
			type: "event",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log1",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log1Anonym",
			type: "event",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log2",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log2Anonym",
			type: "event",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: true,
					internalType: "bytes32",
					name: "aBytes32",
					type: "bytes32",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log3",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: true,
					internalType: "bytes32",
					name: "aBytes32",
					type: "bytes32",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log3Anonym",
			type: "event",
		},
		{
			anonymous: false,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: true,
					internalType: "bytes32",
					name: "aBytes32",
					type: "bytes32",
				},
				{
					indexed: false,
					internalType: "int8",
					name: "aInt8",
					type: "int8",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log4",
			type: "event",
		},
		{
			anonymous: true,
			inputs: [
				{
					indexed: true,
					internalType: "bool",
					name: "aBool",
					type: "bool",
				},
				{
					indexed: true,
					internalType: "address",
					name: "aAddress",
					type: "address",
				},
				{
					indexed: true,
					internalType: "bytes32",
					name: "aBytes32",
					type: "bytes32",
				},
				{
					indexed: false,
					internalType: "int8",
					name: "aInt8",
					type: "int8",
				},
				{
					indexed: false,
					internalType: "uint256",
					name: "value",
					type: "uint256",
				},
			],
			name: "Log4Anonym",
			type: "event",
		},
		{
			inputs: [],
			name: "fireEventLog0",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog0Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog1",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog1Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog2",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog2Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog3",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog3Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog4",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "fireEventLog4Anonym",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [],
			name: "getAddress",
			outputs: [
				{
					internalType: "address",
					name: "ret",
					type: "address",
				},
			],
			stateMutability: "view",
			type: "function",
		},
		{
			inputs: [],
			name: "getBool",
			outputs: [
				{
					internalType: "bool",
					name: "ret",
					type: "bool",
				},
			],
			stateMutability: "view",
			type: "function",
		},
		{
			inputs: [],
			name: "getBytes32",
			outputs: [
				{
					internalType: "bytes32",
					name: "ret",
					type: "bytes32",
				},
			],
			stateMutability: "view",
			type: "function",
		},
		{
			inputs: [],
			name: "getInt256",
			outputs: [
				{
					internalType: "int256",
					name: "ret",
					type: "int256",
				},
			],
			stateMutability: "view",
			type: "function",
		},
		{
			inputs: [],
			name: "getInt8",
			outputs: [
				{
					internalType: "int8",
					name: "ret",
					type: "int8",
				},
			],
			stateMutability: "view",
			type: "function",
		},
		{
			inputs: [],
			name: "getUint256",
			outputs: [
				{
					internalType: "uint256",
					name: "ret",
					type: "uint256",
				},
			],
			stateMutability: "view",
			type: "function",
		},
		{
			inputs: [],
			name: "getUint8",
			outputs: [
				{
					internalType: "uint8",
					name: "ret",
					type: "uint8",
				},
			],
			stateMutability: "view",
			type: "function",
		},
		{
			inputs: [
				{
					internalType: "address",
					name: "_address",
					type: "address",
				},
			],
			name: "setAddress",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [
				{
					internalType: "bool",
					name: "_bool",
					type: "bool",
				},
			],
			name: "setBool",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [
				{
					internalType: "bytes32",
					name: "_bytes32",
					type: "bytes32",
				},
			],
			name: "setBytes32",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [
				{
					internalType: "int256",
					name: "_int256",
					type: "int256",
				},
			],
			name: "setInt256",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [
				{
					internalType: "int8",
					name: "_int8",
					type: "int8",
				},
			],
			name: "setInt8",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [
				{
					internalType: "uint256",
					name: "_uint256",
					type: "uint256",
				},
			],
			name: "setUint256",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
		{
			inputs: [
				{
					internalType: "uint8",
					name: "_uint8",
					type: "uint8",
				},
			],
			name: "setUint8",
			outputs: [],
			stateMutability: "nonpayable",
			type: "function",
		},
	],
};

module.exports = config;
