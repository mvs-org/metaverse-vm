pragma solidity ^0.6.0;

contract JSON_Test {
	event Log0(uint256 value);
	event Log0Anonym(uint256 value) anonymous;
	event Log1(bool indexed aBool, uint256 value);
	event Log1Anonym(bool indexed aBool, uint256 value) anonymous;
	event Log2(bool indexed aBool, address indexed aAddress, uint256 value);
	event Log2Anonym(bool indexed aBool, address indexed aAddress, uint256 value) anonymous;
	event Log3(
		bool indexed aBool,
		address indexed aAddress,
		bytes32 indexed aBytes32,
		uint256 value
	);
	event Log3Anonym(
		bool indexed aBool,
		address indexed aAddress,
		bytes32 indexed aBytes32,
		uint256 value
	) anonymous;
	event Log4(
		bool indexed aBool,
		address indexed aAddress,
		bytes32 indexed aBytes32,
		int8 aInt8,
		uint256 value
	);
	event Log4Anonym(
		bool indexed aBool,
		address indexed aAddress,
		bytes32 indexed aBytes32,
		int8 aInt8,
		uint256 value
	) anonymous;

	constructor() public {}

	function setBool(bool _bool) public {
		myBool = _bool;
	}

	function setInt8(int8 _int8) public {
		myInt8 = _int8;
	}

	function setUint8(uint8 _uint8) public {
		myUint8 = _uint8;
	}

	function setInt256(int256 _int256) public {
		myInt256 = _int256;
	}

	function setUint256(uint256 _uint256) public {
		myUint256 = _uint256;
	}

	function setAddress(address _address) public {
		myAddress = _address;
	}

	function setBytes32(bytes32 _bytes32) public {
		myBytes32 = _bytes32;
	}

	function getBool() public view returns (bool ret) {
		return myBool;
	}

	function getInt8() public view returns (int8 ret) {
		return myInt8;
	}

	function getUint8() public view returns (uint8 ret) {
		return myUint8;
	}

	function getInt256() public view returns (int256 ret) {
		return myInt256;
	}

	function getUint256() public view returns (uint256 ret) {
		return myUint256;
	}

	function getAddress() public view returns (address ret) {
		return myAddress;
	}

	function getBytes32() public view returns (bytes32 ret) {
		return myBytes32;
	}

	function fireEventLog0() public {
		emit Log0(42);
	}

	function fireEventLog0Anonym() public {
		emit Log0Anonym(42);
	}

	function fireEventLog1() public {
		emit Log1(true, 42);
	}

	function fireEventLog1Anonym() public {
		emit Log1Anonym(true, 42);
	}

	function fireEventLog2() public {
		emit Log2(true, msg.sender, 42);
	}

	function fireEventLog2Anonym() public {
		emit Log2Anonym(true, msg.sender, 42);
	}

	function fireEventLog3() public {
		emit Log3(
			true,
			msg.sender,
			0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
			42
		);
	}

	function fireEventLog3Anonym() public {
		emit Log3Anonym(
			true,
			msg.sender,
			0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
			42
		);
	}

	function fireEventLog4() public {
		emit Log4(
			true,
			msg.sender,
			0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
			-23,
			42
		);
	}

	function fireEventLog4Anonym() public {
		emit Log4Anonym(
			true,
			msg.sender,
			0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
			-23,
			42
		);
	}

	bool myBool;
	int8 myInt8;
	uint8 myUint8;
	int256 myInt256;
	uint256 myUint256;
	address myAddress;
	bytes32 myBytes32;
}
