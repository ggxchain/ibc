
# 1 ibc infomation
In order to enable IBC communication, a contract must expose the following 6 entry points.
```js
        fn ibc_channel_open(&self, msg: IbcChannelOpenMsg) ->  Result<IbcChannelOpenResponse, Error>;
        fn ibc_channel_connect(&mut self, msg: IbcChannelConnectMsg) ->  Result<IbcBasicResponse, Error>;
        fn ibc_channel_close(&self, msg: IbcChannelCloseMsg) ->  Result<IbcBasicResponse, Error>;
        fn ibc_packet_receive(&self, msg: IbcPacketReceiveMsg) -> Result<IbcReceiveResponse, Error>;
        fn ibc_packet_ack(&self, _msg: IbcPacketAckMsg) -> Result<IbcBasicResponse, Error>;
        fn ibc_packet_timeout(&self, _msg: IbcPacketTimeoutMsg) -> Result<IbcBasicResponse, Error>;
```

## 1.1 Channel lifecycle management

![](https://github.com/cosmos/ibc/raw/main/spec/core/ics-004-channel-and-packet-semantics/channel-state-machine.png)


## 1.2 interface 
### the ink! interface
```js
        /// support submessage callbacks //for developer submessage
        fn reply(&self, reply: Reply) ->  Result<Response, Error>;

        /// in-place contract migrations //for contract upgrade
        fn migrate(&self, _msg: Empty) ->  Result<Response, Error>;

        /// The first step of a handshake on either chain is ibc_channel_open
        fn ibc_channel_open(&self, msg: IbcChannelOpenMsg) ->  Result<IbcChannelOpenResponse, Error>;

        /// Once both sides have returned Ok() to ibc_channel_open, we move onto the second step of the handshake, which is equivalent to ChanOpenAck and ChanOpenConfirm from the spec
        fn ibc_channel_connect(&mut self, msg: IbcChannelConnectMsg) ->  Result<IbcBasicResponse, Error>;

        /// Once a channel is closed, whether due to an IBC error, at our request, or at the request of the other side, the following callback is made on the contract, which allows it to take appropriate cleanup action
        fn ibc_channel_close(&self, msg: IbcChannelCloseMsg) ->  Result<IbcBasicResponse, Error>;

        /// After a contract on chain A sends a packet, it is generally processed by the contract on chain B on the other side of the channel. This is done by executing the following entry point on chain B:
        fn ibc_packet_receive(&self, msg: IbcPacketReceiveMsg)
            -> Result<IbcReceiveResponse, Error>;

        /// If chain B successfully received the packet (even if the contract returned an error message), chain A will eventually get an acknowledgement:
        fn ibc_packet_ack(&self, _msg: IbcPacketAckMsg) -> Result<IbcBasicResponse, Error>;

        /// If the packet was not received on chain B before the timeout, we can be certain that it will never be processed there. In such a case, a relayer can return a timeout proof to cancel the pending packet. In such a case the calling contract will never get ibc_packet_ack, but rather ibc_packet_timeout. One of the two calls will eventually get called for each packet that is sent as long as there is a functioning relayer. (In the absence of a functioning relayer, it will never get a response).
        fn ibc_packet_timeout(&self, _msg: IbcPacketTimeoutMsg) -> Result<IbcBasicResponse, Error>;
```

## 1.3 open channel
```js
        /// The first step of a handshake on either chain is ibc_channel_open
        fn ibc_channel_open(&self, msg: IbcChannelOpenMsg) -> Result<IbcChannelOpenResponse, Error>;
```

## 1.4 connect a channel
```js
    /// Once both sides have returned Ok() to ibc_channel_open, we move onto the second step of the handshake, which is equivalent to ChanOpenAck and ChanOpenConfirm from the spec
    fn ibc_channel_connect(&mut self, msg: IbcChannelConnectMsg) -> Result<IbcBasicResponse, Error>;
```

## 1.5 close a channel
```js
      /// Once a channel is closed, whether due to an IBC error, at our request, or at the request of the other side, the following callback is made on the contract, which allows it to take appropriate cleanup action
      fn ibc_channel_close(&self, msg: IbcChannelCloseMsg) -> Result<IbcBasicResponse, Error>;
```

## 1.6 sending a packet
```js
 /// execute spec set function  for ExecuteMsg
 pub fn execute(&self, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, Error> 
```

## 1.7 receiving a packet
```js
    /// After a contract on chain A sends a packet, it is generally processed by the contract on chain B on the other side of the channel. This is done by executing the following entry point on chain B:
    fn ibc_packet_receive(&self, msg: IbcPacketReceiveMsg)
        -> Result<IbcReceiveResponse, Error>;
```

## 1.8 error handing
```js
// 1.8.1  If the message doesn't modify any state directly, capture errors, converting them into error acknowledgements
fn ibc_packet_receive(&self, msg: IbcPacketReceiveMsg)-> Result<IbcReceiveResponse, Error> {
 (|| {
        // which local channel did this packet come on
        let packet = msg.packet;
        let caller = packet.dest.channel_id;
        let msg: PacketMsg = from_slice(&packet.data)?;
        match msg {
            PacketMsg::Dispatch { msgs } => self.receive_dispatch(deps, caller, msgs),
            PacketMsg::WhoAmI {} => self.receive_who_am_i(deps, caller),
            PacketMsg::Balances {} => self.receive_balances(deps, caller),
        }
    })()
    .or_else(|e| {
        // we try to capture all app-level errors and convert them into
        // acknowledgement packets that contain an error code.
        let acknowledgement = encode_ibc_error(format!("invalid packet: {}", e));
        Ok(IbcReceiveResponse {
            acknowledgement,
            submessages: vec![],
            messages: vec![],
            attributes: vec![],
        })
    })
}

// 1.8.2  If we modify state with an external call, we need to wrap it in a submessage and capture the error. 
fn receive_dispatch(
    &self, 
    caller: String,
    msgs: Vec<CosmosMsg>,
) -> Result<IbcReceiveResponse> {
    // what is the reflect contract here
    let reflect_addr = accounts(deps.storage).load(caller.as_bytes())?;//todo, do ink! can reflect address?

    // let them know we're fine
    let acknowledgement = to_binary(&AcknowledgementMsg::<DispatchResponse>::Ok(()))?;
    // create the message to re-dispatch to the reflect contract
    let reflect_msg = ReflectExecuteMsg::ReflectMsg { msgs };
    let wasm_msg = wasm_execute(reflect_addr, &reflect_msg, vec![])?;

    // we wrap it in a submessage to properly report errors
    let sub_msg = SubMsg {
        id: RECEIVE_DISPATCH_ID,
        msg: wasm_msg.into(),
        gas_limit: None,
        reply_on: ReplyOn::Error,
    };

    Ok(IbcReceiveResponse {
        acknowledgement,
        submessages: vec![sub_msg],
        messages: vec![],
        attributes: vec![attr("action", "receive_dispatch")],
    })
}

pub fn reply(&self, reply: Reply) -> Response {
   match (reply.id, reply.result) {
      (RECEIVE_DISPATCH_ID, ContractResult::Err(err)) => Ok(Response {
         data: Some(encode_ibc_error(err)),
         ..Response::default()
      }),
      (INIT_CALLBACK_ID, ContractResult::Ok(response)) => handle_init_callback(deps, response),
      _ => Err(Error("invalid reply id or result")),
   }
}

```

## 1.9 json and protobuf based acknowledgement envelopes
```js
/// Although the ICS spec leave the actual acknowledgement as opaque bytes, it does provide a recommendation for the format you can use, allowing contracts to easily differentiate between success and error (and allow IBC explorers to label such packets without knowing every protocol).

It is defined as part of the ICS4 - Channel Spec.
message Acknowledgement {
  // response contains either a result or an error and must be non-empty
  oneof response {
    bytes  result = 21;
    string error  = 22;
  }
}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// This is compatible with the JSON serialization
pub enum Ics20Ack {
    Result(Vec<u8>),
    Error(String),
}

// create a serialized success message
fn ack_success() -> Vec<u8> {
    let res = Ics20Ack::Result(b"1".into());
    to_binary(&res).unwrap()
}

// create a serialized error message
fn ack_fail(err: String) -> Vec<u8> {
    let res = Ics20Ack::Error(err);
    to_binary(&res).unwrap()
}
```
## 1.10 acknowledgement handing
```js
        /// If chain B successfully received the packet (even if the contract returned an error message), chain A will eventually get an acknowledgement:
        fn ibc_packet_ack(&self, _msg: IbcPacketAckMsg) -> Result<IbcBasicResponse, Error>;
```

## 1.11 handing timeouts
```js
        /// If the packet was not received on chain B before the timeout, we can be certain that it will never be processed there. In such a case, a relayer can return a timeout proof to cancel the pending packet. In such a case the calling contract will never get ibc_packet_ack, but rather ibc_packet_timeout. One of the two calls will eventually get called for each packet that is sent as long as there is a functioning relayer. (In the absence of a functioning relayer, it will never get a response).
        fn ibc_packet_timeout(&self, _msg: IbcPacketTimeoutMsg) -> Result<IbcBasicResponse, Error>;
```

## the ibc struct
```js
    pub enum IbcChannelOpenMsg {
        /// The ChanOpenInit step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        OpenInit { channel: IbcChannel },
        /// The ChanOpenTry step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        OpenTry {
            channel: IbcChannel,
            counterparty_version: String,
        },
    }

    pub enum IbcChannelConnectMsg {
        /// The ChanOpenAck step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        OpenAck {
            channel: IbcChannel,
            counterparty_version: String,
        },
        /// The ChanOpenConfirm step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        OpenConfirm { channel: IbcChannel },
    }

    pub enum IbcChannelCloseMsg {
        /// The ChanCloseInit step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        CloseInit { channel: IbcChannel },
        /// The ChanCloseConfirm step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        CloseConfirm { channel: IbcChannel }, // pub channel: IbcChannel,
    }

    pub struct IbcPacketReceiveMsg {
        pub packet: IbcPacket,
        #[cfg(feature = "ibc3")]
        pub relayer: Addr,
    }

    pub struct IbcPacketAckMsg {
        pub acknowledgement: IbcAcknowledgement,
        pub original_packet: IbcPacket,
        #[cfg(feature = "ibc3")]
        pub relayer: Addr,
    }

    pub struct IbcPacketTimeoutMsg {
        pub packet: IbcPacket,
        #[cfg(feature = "ibc3")]
        pub relayer: Addr,
    }

    pub struct Addr(String);

    pub struct IbcEndpoint {
        pub port_id: String,
        pub channel_id: String,
    }

    pub enum IbcOrder {
        Unordered,
        Ordered,
    }

    pub struct IbcPacket {
        /// The raw data sent from the other side in the packet
        pub data: Vec<u8>,
        /// identifies the channel and port on the sending chain.
        pub src: IbcEndpoint,
        /// identifies the channel and port on the receiving chain.
        pub dest: IbcEndpoint,
        /// The sequence number of the packet on the given channel
        pub sequence: u64,
        pub timeout: IbcTimeout,
    }

    pub struct IbcTimeout {
        block: Option<IbcTimeoutBlock>,
        timestamp: Option<Timestamp>,
    }

    pub struct IbcTimeoutBlock {
        /// the version that the client is currently on
        /// (eg. after reseting the chain this could increment 1 as height drops to 0)
        pub revision: u64,
        /// block height after which the packet times out.
        /// the height within the given revision
        pub height: u64,
    }

    pub struct IbcAcknowledgement {
        pub data: Vec<u8>,
        // we may add more info here in the future (meta-data from the acknowledgement)
        // there have been proposals to extend this type in core ibc for future versions
    }

    pub enum SubMsgResult {
        Ok(SubMsgResponse),
        /// An error type that every custom error created by contract developers can be converted to.
        /// This could potientially have more structure, but String is the easiest.
        Err(String),
    }

    pub struct SubMsgResponse {
        pub events: Vec<Event>,
        pub data: Option<Vec<u8>>,
    }

    pub struct Event {
        /// The event type. This is renamed to "ty" because "type" is reserved in Rust. This sucks, we know.
        pub ty: String,
        /// The attributes to be included in the event.
        ///
        /// You can learn more about these from [*Cosmos SDK* docs].
        ///
        /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/core/events.html
        pub attributes: Vec<Attribute>,
    }

    pub struct Attribute {
        pub key: String,
        pub value: String,
    }

    pub struct IbcChannel {
        pub endpoint: IbcEndpoint,
        pub counterparty_endpoint: IbcEndpoint,
        pub order: IbcOrder,
        /// Note: in ibcv3 this may be "", in the IbcOpenChannel handshake messages
        pub version: String,
        /// The connection upon which this channel was created. If this is a multi-hop
        /// channel, we only expose the first hop.
        pub connection_id: String,
    }

    pub enum CosmosMsg<T> {
        Bank(BankMsg),
        // by default we use RawMsg, but a contract can override that
        // to call into more app-specific code (whatever they define)
        Custom(T),
        //#[cfg(feature = "staking")]
        Staking(StakingMsg),
        //#[cfg(feature = "staking")]
        Distribution(DistributionMsg),
        /// A Stargate message encoded the same way as a protobuf [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
        /// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
        //#[cfg(feature = "stargate")]
        Stargate {
            type_url: String,
            value: Vec<u8>,
        },
        //#[cfg(feature = "stargate")]
        Ibc(IbcMsg),
        Wasm(WasmMsg),
        //#[cfg(feature = "stargate")]
        //Gov(GovMsg),
    }

    pub enum BankMsg {
        /// Sends native tokens from the contract to the given address.
        ///
        /// This is translated to a [MsgSend](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto#L19-L28).
        /// `from_address` is automatically filled with the current contract's address.
        Send {
            to_address: String,
            amount: Vec<Coin>,
        },
        /// This will burn the given coins from the contract's account.
        /// There is no Cosmos SDK message that performs this, but it can be done by calling the bank keeper.
        /// Important if a contract controls significant token supply that must be retired.
        Burn { amount: Vec<Coin> },
    }

    pub enum StakingMsg {
        /// This is translated to a [MsgDelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L81-L90).
        /// `delegator_address` is automatically filled with the current contract's address.
        Delegate { validator: String, amount: Coin },
        /// This is translated to a [MsgUndelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L112-L121).
        /// `delegator_address` is automatically filled with the current contract's address.
        Undelegate { validator: String, amount: Coin },
        /// This is translated to a [MsgBeginRedelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L95-L105).
        /// `delegator_address` is automatically filled with the current contract's address.
        Redelegate {
            src_validator: String,
            dst_validator: String,
            amount: Coin,
        },
    }

    pub enum DistributionMsg {
        /// This is translated to a [MsgSetWithdrawAddress](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L29-L37).
        /// `delegator_address` is automatically filled with the current contract's address.
        SetWithdrawAddress {
            /// The `withdraw_address`
            address: String,
        },
        /// This is translated to a [[MsgWithdrawDelegatorReward](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L42-L50).
        /// `delegator_address` is automatically filled with the current contract's address.
        WithdrawDelegatorReward {
            /// The `validator_address`
            validator: String,
        },
    }

    pub enum WasmMsg {
        /// Dispatches a call to another contract at a known address (with known ABI).
        ///
        /// This is translated to a [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78).
        /// `sender` is automatically filled with the current contract's address.
        Execute {
            contract_addr: String,
            /// msg is the json-encoded ExecuteMsg struct (as raw Binary)
            //#[derivative(Debug(format_with = "binary_to_string"))]
            msg: Vec<u8>,
            funds: Vec<Coin>,
        },
        /// Instantiates a new contracts from previously uploaded Wasm code.
        ///
        /// The contract address is non-predictable. But it is guaranteed that
        /// when emitting the same Instantiate message multiple times,
        /// multiple instances on different addresses will be generated. See also
        /// Instantiate2.
        ///
        /// This is translated to a [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L53-L71).
        /// `sender` is automatically filled with the current contract's address.
        Instantiate {
            admin: Option<String>,
            code_id: u64,
            /// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
            //#[derivative(Debug(format_with = "binary_to_string"))]
            msg: Vec<u8>,
            funds: Vec<Coin>,
            /// A human-readbale label for the contract
            label: String,
        },
        /// Instantiates a new contracts from previously uploaded Wasm code
        /// using a predictable address derivation algorithm implemented in
        /// [`cosmwasm_std::instantiate2_address`].
        ///
        /// This is translated to a [MsgInstantiateContract2](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L73-L96).
        /// `sender` is automatically filled with the current contract's address.
        /// `fix_msg` is automatically set to false.
        #[cfg(feature = "cosmwasm_1_2")]
        Instantiate2 {
            admin: Option<String>,
            code_id: u64,
            /// A human-readbale label for the contract
            label: String,
            /// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
            //#[derivative(Debug(format_with = "binary_to_string"))]
            msg: Vec<u8>,
            funds: Vec<Coin>,
            salt: Vec<u8>,
        },
        /// Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to
        /// customize behavior.
        ///
        /// Only the contract admin (as defined in wasmd), if any, is able to make this call.
        ///
        /// This is translated to a [MsgMigrateContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L86-L96).
        /// `sender` is automatically filled with the current contract's address.
        Migrate {
            contract_addr: String,
            /// the code_id of the new logic to place in the given contract
            new_code_id: u64,
            /// msg is the json-encoded MigrateMsg struct that will be passed to the new code
            //#[derivative(Debug(format_with = "binary_to_string"))]
            msg: Vec<u8>,
        },
        /// Sets a new admin (for migrate) on the given contract.
        /// Fails if this contract is not currently admin of the target contract.
        UpdateAdmin {
            contract_addr: String,
            admin: String,
        },
        /// Clears the admin on the given contract, so no more migration possible.
        /// Fails if this contract is not currently admin of the target contract.
        ClearAdmin { contract_addr: String },
    }

    pub enum IbcMsg {
        /// Sends bank tokens owned by the contract to the given address on another chain.
        /// The channel must already be established between the ibctransfer module on this chain
        /// and a matching module on the remote chain.
        /// We cannot select the port_id, this is whatever the local chain has bound the ibctransfer
        /// module to.
        Transfer {
            /// exisiting channel to send the tokens over
            channel_id: String,
            /// address on the remote chain to receive these tokens
            to_address: String,
            /// packet data only supports one coin
            /// https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
            amount: Coin,
            /// when packet times out, measured on remote chain
            timeout: IbcTimeout,
        },
        /// Sends an IBC packet with given data over the existing channel.
        /// Data should be encoded in a format defined by the channel version,
        /// and the module on the other side should know how to parse this.
        SendPacket {
            channel_id: String,
            data: Vec<u8>,
            /// when packet times out, measured on remote chain
            timeout: IbcTimeout,
        },
        /// This will close an existing channel that is owned by this contract.
        /// Port is auto-assigned to the contract's IBC port
        CloseChannel { channel_id: String },
    }

    pub struct Coin {
        pub denom: String,
        pub amount: u128,
    }

    pub struct Empty;

    pub struct Response<T = Empty> {
        /// Optional list of messages to pass. These will be executed in order.
        /// If the ReplyOn variant matches the result (Always, Success on Ok, Error on Err),
        /// the runtime will invoke this contract's `reply` entry point
        /// after execution. Otherwise, they act like "fire and forget".
        /// Use `SubMsg::new` to create messages with the older "fire and forget" semantics.
        pub messages: Vec<SubMsg<T>>,
        /// The attributes that will be emitted as part of a "wasm" event.
        ///
        /// More info about events (and their attributes) can be found in [*Cosmos SDK* docs].
        ///
        /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/core/events.html
        pub attributes: Vec<Attribute>,
        /// Extra, custom events separate from the main `wasm` one. These will have
        /// `wasm-` prepended to the type.
        ///
        /// More info about events can be found in [*Cosmos SDK* docs].
        ///
        /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/core/events.html
        pub events: Vec<Event>,
        /// The binary payload to include in the response.
        pub data: Option<Vec<u8>>,
    }

    pub struct SubMsg<T = Empty> {
        /// An arbitrary ID chosen by the contract.
        /// This is typically used to match `Reply`s in the `reply` entry point to the submessage.
        pub id: u64,
        pub msg: CosmosMsg<T>,
        /// Gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
        pub gas_limit: Option<u64>,
        pub reply_on: ReplyOn,
    }

    pub enum ReplyOn {
        /// Always perform a callback after SubMsg is processed
        Always,
        /// Only callback if SubMsg returned an error, no callback on success case
        Error,
        /// Only callback if SubMsg was successful, no callback on error case
        Success,
        /// Never make a callback - this is like the original CosmosMsg semantics
        Never,
    }

    pub enum QueryRequest<C> {
        //Bank(BankQuery),
        Custom(C),
        // #[cfg(feature = "staking")]
        // Staking(StakingQuery),
        // /// A Stargate query is encoded the same way as abci_query, with path and protobuf encoded request data.
        // /// The format is defined in [ADR-21](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-021-protobuf-query-encoding.md).
        // /// The response is protobuf encoded data directly without a JSON response wrapper.
        // /// The caller is responsible for compiling the proper protobuf definitions for both requests and responses.
        // #[cfg(feature = "stargate")]
        // Stargate {
        //     /// this is the fully qualified service path used for routing,
        //     /// eg. custom/cosmos_sdk.x.bank.v1.Query/QueryBalance
        //     path: String,
        //     /// this is the expected protobuf message type (not any), binary encoded
        //     data: Binary,
        // },
        // #[cfg(feature = "stargate")]
        // Ibc(IbcQuery),
        // Wasm(WasmQuery),
    }

    pub struct Reply {
        /// The ID that the contract set when emitting the `SubMsg`.
        /// Use this to identify which submessage triggered the `reply`.
        pub id: u64,
        pub result: SubMsgResult,
    }

    /// Note that this serializes as "null".
    #[cfg(not(feature = "ibc3"))]
    pub type IbcChannelOpenResponse = ();

    /// This serializes either as "null" or a JSON object.   
    #[cfg(feature = "ibc3")]
    pub type IbcChannelOpenResponse = Option<Ibc3ChannelOpenResponse>;

    
    
    pub struct Ibc3ChannelOpenResponse {
        /// We can set the channel version to a different one than we were called with
        pub version: String,
    }

    pub struct IbcBasicResponse<T = Empty> {
        /// Optional list of messages to pass. These will be executed in order.
        /// If the ReplyOn member is set, they will invoke this contract's `reply` entry point
        /// after execution. Otherwise, they act like "fire and forget".
        /// Use `SubMsg::new` to create messages with the older "fire and forget" semantics.
        pub messages: Vec<SubMsg<T>>,
        /// The attributes that will be emitted as part of a `wasm` event.
        ///
        /// More info about events (and their attributes) can be found in [*Cosmos SDK* docs].
        ///
        /// [*Cosmos SDK* docs]: https://docs.cosmos.network/v0.42/core/events.html
        pub attributes: Vec<Attribute>,
        /// Extra, custom events separate from the main `wasm` one. These will have
        /// `wasm-` prepended to the type.
        ///
        /// More info about events can be found in [*Cosmos SDK* docs].
        ///
        /// [*Cosmos SDK* docs]: https://docs.cosmos.network/v0.42/core/events.html
        pub events: Vec<Event>,
    }

    pub struct IbcReceiveResponse<T = Empty> {
        /// The bytes we return to the contract that sent the packet.
        /// This may represent a success or error of exection
        pub acknowledgement: Vec<u8>,
        /// Optional list of messages to pass. These will be executed in order.
        /// If the ReplyOn member is set, they will invoke this contract's `reply` entry point
        /// after execution. Otherwise, they act like "fire and forget".
        /// Use `call` or `msg.into()` to create messages with the older "fire and forget" semantics.
        pub messages: Vec<SubMsg<T>>,
        /// The attributes that will be emitted as part of a "wasm" event.
        ///
        /// More info about events (and their attributes) can be found in [*Cosmos SDK* docs].
        ///
        /// [*Cosmos SDK* docs]: https://docs.cosmos.network/v0.42/core/events.html
        pub attributes: Vec<Attribute>,
        /// Extra, custom events separate from the main `wasm` one. These will have
        /// `wasm-` prepended to the type.
        ///
        /// More info about events can be found in [*Cosmos SDK* docs].
        ///
        /// [*Cosmos SDK* docs]: https://docs.cosmos.network/v0.42/core/events.html
        pub events: Vec<Event>,
    }

```

# 2 ics20 interface
## 2.1 ink! interface
```js
        /// execute spec set function  for ExecuteMsg
        pub fn execute(&self, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, Error>;

        /// query info for spec QueryMsg
        pub fn query(&self, msg: QueryMsg) -> Result<Vec<u8>, Error> ;

        /// receive token, This accepts a properly-encoded ReceiveMsg from a cw20 contract
        pub fn execute_receive(
            &self,
            info: MessageInfo,
            wrapper: Cw20ReceiveMsg,
        ) -> Result<Response, Error> ;

        /// transfer token, This allows us to transfer *exactly one* native token
        pub fn execute_transfer(
            &self,
            msg: TransferMsg,
            amount: Amount,
            sender: Addr,
        ) -> Result<Response, Error> ;

        /// This must be called by gov_contract, will allow a new cw20 token to be sent
        //// The gov contract can allow new contracts, or increase the gas limit on existing contracts.
        /// It cannot block or reduce the limit to avoid forcible sticking tokens in the channel.
        pub fn execute_allow(&self, info: MessageInfo, allow: AllowMsg) -> Result<Response, Error>;

        /// update admin address, Change the admin (must be called by current admin)
        pub fn execute_update_admin(&self, addr: Addr) -> Result<Response, Error> ;

        // query function list

        /// Return the port ID bound by this contract.
        pub fn query_port(&self) -> PortResponse ;

        /// Show all channels we have connected to.
        pub fn query_list(&self) -> ListChannelsResponse ;

        ///  Returns the details of the name channel, error if not created.
        pub fn query_channel(&self, id: String) -> ChannelResponse ;

        /// Show the Config.
        pub fn query_config(&self) -> ConfigResponse ;

        /// Query if a given cw20 contract is allowed.
        pub fn query_allowed(&self) -> AllowedResponse ;

        /// List all allowed cw20 contracts.
        pub fn list_allowed(
            &self,
            start_after: Option<String>,
            limit: Option<u32>,
        ) -> ListAllowedResponse ;

        /// Show current admin
        pub fn query_admin(&self) -> Option<Addr>;
```

## 2.2 p2p22 interface  (open brach ink! Fungible Token Standard for Substrate's contracts pallet)
```js
/// Trait implemented by all PSP-20 respecting smart traits.
#[openbrush::trait_definition]
pub trait PSP22 {
    /// Returns the total token supply.
    fn total_supply(&self) -> Balance;

    /// Returns the account Balance for the specified `owner`.
    ///
    /// Returns `0` if the account is non-existent.
    fn balance_of(&self, owner: AccountId) -> Balance;

    /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
    ///
    /// Returns `0` if no allowance has been set `0`.
    fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

    /// Transfers `value` amount of tokens from the caller's account to account `to`
    /// with additional `data` in unspecified format.
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `InsufficientBalance` error if there are not enough tokens on
    /// the caller's account Balance.
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    fn transfer(&mut self, to: AccountId, value: Balance, data: Vec<u8>) -> Result<(), PSP22Error>;

    /// Transfers `value` tokens on the behalf of `from` to the account `to`
    /// with additional `data` in unspecified format.
    ///
    /// This can be used to allow a contract to transfer tokens on ones behalf and/or
    /// to charge fees in sub-currencies, for example.
    ///
    /// On success a `Transfer` and `Approval` events are emitted.
    ///
    /// # Errors
    ///
    /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
    /// for the caller to withdraw from `from`.
    ///
    /// Returns `InsufficientBalance` error if there are not enough tokens on
    /// the the account Balance of `from`.
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        value: Balance,
        data: Vec<u8>,
    ) -> Result<(), PSP22Error>;

    /// Allows `spender` to withdraw from the caller's account multiple times, up to
    /// the `value` amount.
    ///
    /// If this function is called again it overwrites the current allowance with `value`.
    ///
    /// An `Approval` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error>;

    /// Atomically increases the allowance granted to `spender` by the caller.
    ///
    /// An `Approval` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    fn increase_allowance(&mut self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error>;

    /// Atomically decreases the allowance granted to `spender` by the caller.
    ///
    /// An `Approval` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
    /// by owner for `spender`.
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    fn decrease_allowance(&mut self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error>;
}

#[openbrush::wrapper]
pub type PSP22ReceiverRef = dyn PSP22Receiver;

/// PSP22Receiver is a trait for any contract that wants to support safe transfers from a PSP22
/// token smart contract to avoid unexpected tokens in the balance of contract.
/// This method is called before a transfer to ensure the recipient of the tokens acknowledges the receipt.
#[openbrush::trait_definition]
pub trait PSP22Receiver {
    /// Ensures that the smart contract allows reception of PSP22 token(s).
    /// Returns `Ok(())` if the contract allows the reception of the token(s) and Error `TransferRejected(String))` otherwise.
    ///
    /// This method will get called on every transfer to check whether the recipient in `transfer` or
    /// `transfer_from` is a contract, and if it is, does it accept tokens.
    /// This is done to prevent contracts from locking tokens forever.
    ///
    /// Returns `PSP22ReceiverError` if the contract does not accept the tokens.
    fn before_received(
        &mut self,
        operator: AccountId,
        from: AccountId,
        value: Balance,
        data: Vec<u8>,
    ) -> Result<(), PSP22ReceiverError>;
}
```

## 2.3 storage
```js
    pub struct Contract {
        /// p2p22 storage data
        // pub struct p2p22::Data {
        //   pub supply: Balance,
        //   pub balances: Mapping<AccountId, Balance>,
        //   pub allowances: Mapping<(AccountId, AccountId), Balance, AllowancesKey>,
        //   pub _reserved: Option<()>,
        // }
        #[storage_field]
        psp22: psp22::Data,
        /// wrapper storage data
        // pub struct wrapper::Data {
        //   pub underlying: AccountId,
        //   ub _reserved: Option<()>,
        // }
        #[storage_field]
        wrapper: wrapper::Data,

        /// contract admin
        admin: Addr,
        /// isc20_config
        config: Config,
        /// Used to pass info from the ibc_packet_receive to the reply handler
        replay_args: ReplyArgs,
        /// static info on one channel that doesn't change
        channel_info: Mapping<String, ChannelInfo>,
        /// indexed by (channel_id, denom) maintaining the balance of the channel in that currency
        channel_state: Mapping<String, String>,
        /// Every cw20 contract we allow to be sent is stored here, possibly with a gas_limit
        allow_list: Mapping<Addr, AllowInfo>,
    }
```

## 2.4 struct
```js
    pub struct Addr(String);

    pub struct MessageInfo {
        pub sender: Addr,
        pub funds: Vec<Coin>,
    }

    pub struct Coin {
        pub denom: String,
        pub amount: u128,
    }

    pub struct Cw20Coin {
        pub address: String,
        pub amount: u128,
    }

    pub struct Cw20ReceiveMsg {
        pub sender: String,
        pub amount: u128,
        pub msg: Vec<u8>,
    }

    pub struct AllowMsg {
        pub contract: String,
        pub gas_limit: Option<u64>,
    }

    pub struct InitMsg {
        /// Default timeout for ics20 packets, specified in seconds
        pub default_timeout: u64,
        /// who can allow more contracts
        pub gov_contract: String,
        /// initial allowlist - all cw20 tokens we will send must be previously allowed by governance
        pub allowlist: Vec<AllowMsg>,
        /// If set, contracts off the allowlist will run with this gas limit.
        /// If unset, will refuse to accept any contract off the allow list.
        pub default_gas_limit: Option<u64>,
    }

    pub struct TransferMsg {
        /// The local channel to send the packets on
        pub channel: String,
        /// The remote address to send to.
        /// Don't use HumanAddress as this will likely have a different Bech32 prefix than we use
        /// and cannot be validated locally
        pub remote_address: String,
        /// How long the packet lives in seconds. If not specified, use default_timeout
        pub timeout: Option<u64>,
        /// An optional memo to add to the IBC transfer
        pub memo: Option<String>,
    }


    pub enum Amount {
        Native(Coin),
        // FIXME? USe Cw20CoinVerified, and validate cw20 addresses
        Cw20(Cw20Coin),
    }

    impl Amount {
        // TODO: write test for this
        pub fn from_parts(denom: String, amount: u128) -> Self {
            if denom.starts_with("cw20:") {
                let address = denom.get(5..).unwrap().into();
                Amount::Cw20(Cw20Coin { address, amount })
            } else {
                Amount::Native(Coin { denom, amount })
            }
        }

        pub fn cw20(amount: u128, addr: &str) -> Self {
            Amount::Cw20(Cw20Coin {
                address: addr.into(),
                amount: amount,
            })
        }

        pub fn native(amount: u128, denom: &str) -> Self {
            Amount::Native(Coin {
                denom: denom.to_string(),
                amount: amount,
            })
        }
    }

    impl Amount {
        pub fn denom(&self) -> String {
            match self {
                Amount::Native(c) => c.denom.clone(),
                Amount::Cw20(c) => "cw20:".to_owned() + c.address.as_str(),
            }
        }

        pub fn amount(&self) -> u128 {
            match self {
                Amount::Native(c) => c.amount,
                Amount::Cw20(c) => c.amount,
            }
        }

        /// convert the amount into u64
        pub fn u64_amount(&self) -> Result<u64, Error> {
            Ok(self.amount().try_into().unwrap())
        }

        pub fn is_empty(&self) -> bool {
            match self {
                Amount::Native(c) => c.amount == 0,
                Amount::Cw20(c) => c.amount == 0,
            }
        }
    }

    pub struct IbcEndpoint {
        pub port_id: String,
        pub channel_id: String,
    }

    pub struct ChannelInfo {
        /// id of this channel
        pub id: String,
        /// the remote channel/port we connect to
        pub counterparty_endpoint: IbcEndpoint,
        /// the connection this exists on (you can use to query client/consensus info)
        pub connection_id: String,
    }

    pub struct AllowedInfo {
        pub contract: String,
        pub gas_limit: Option<u64>,
    }

    pub struct PortResponse {
        pub port_id: String,
    }

    pub struct ListChannelsResponse {
        pub channels: Vec<ChannelInfo>,
    }

    pub struct ChannelResponse {
        /// Information on the channel's connection
        pub info: ChannelInfo,
        /// How many tokens we currently have pending over this channel
        pub balances: Vec<Amount>,
        /// The total number of tokens that have been sent over this channel
        /// (even if many have been returned, so balance is low)
        pub total_sent: Vec<Amount>,
    }

    pub struct ConfigResponse {
        pub default_timeout: u64,
        pub default_gas_limit: Option<u64>,
        pub gov_contract: String,
    }

    pub struct AllowedResponse {
        pub is_allowed: bool,
        pub gas_limit: Option<u64>,
    }

    pub struct ListAllowedResponse {
        pub allow: Vec<AllowedInfo>,
    }

    pub enum ExecuteMsg {
        /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
        Receive(Cw20ReceiveMsg),
        /// This allows us to transfer *exactly one* native token
        Transfer(TransferMsg),
        /// This must be called by gov_contract, will allow a new cw20 token to be sent
        Allow(AllowMsg),
        /// Change the admin (must be called by current admin)
        UpdateAdmin { admin: String },
    }

    pub enum QueryMsg {
        /// Return the port ID bound by this contract.
        //#[returns(PortResponse)]
        Port {},
        /// Show all channels we have connected to.
        // #[returns(ListChannelsResponse)]
        ListChannels {},
        /// Returns the details of the name channel, error if not created.
        //#[returns(ChannelResponse)]
        Channel {
            id: String,
        },
        /// Show the Config.
        //#[returns(ConfigResponse)]
        Config {},
        //#[returns(cw_controllers::AdminResponse)]
        Admin {},
        /// Query if a given cw20 contract is allowed.
        //#[returns(AllowedResponse)]
        Allowed {
            contract: String,
        },
        /// List all allowed cw20 contracts.
        //#[returns(ListAllowedResponse)]
        ListAllowed {
            start_after: Option<String>,
            limit: Option<u32>,
        },
    }

    pub struct Config {
        pub default_timeout: u64,
        pub default_gas_limit: Option<u64>,
    }

    pub struct ReplyArgs {
        pub channel: String,
        pub denom: String,
        pub amount: u128,
    }

    pub struct AllowInfo {
        pub gas_limit: Option<u64>,
    }
```
## 2.5 error
```js
pub enum Error {
        /// StdError
        StdError,
        /// PaymentError
        PaymentError,
        /// AdminError
        AdminError,
        /// #[error("Channel doesn't exist: {id}")]
        NoSuchChannel { id: String },
        /// #[error("Didn't send any funds")]
        NoFunds {},
        /// #[error("Amount larger than 2**64, not supported by ics20 packets")]
        AmountOverflow {},
        /// #[error("Only supports channel with ibc version ics20-1, got {version}")]
        InvalidIbcVersion { version: String },
        /// #[error("Only supports unordered channel")]
        OnlyOrderedChannel {},
        /// #[error("Insufficient funds to redeem voucher on channel")]
        InsufficientFunds {},
        /// #[error("Only accepts tokens that originate on this chain, not native tokens of remote chain")]
        NoForeignTokens {},
        /// #[error("Parsed port from denom ({port}) doesn't match packet")]
        FromOtherPort { port: String },
        /// #[error("Parsed channel from denom ({channel}) doesn't match packet")]
        FromOtherChannel { channel: String },
        /// #[error("Cannot migrate from different contract type: {previous_contract}")]
        CannotMigrate { previous_contract: String },
        /// #[error("Cannot migrate from unsupported version: {previous_version}")]
        CannotMigrateVersion { previous_version: String },
        /// #[error("Got a submessage reply with unknown id: {id}")]
        UnknownReplyId { id: u64 },
        /// #[error("You cannot lower the gas limit for a contract on the allow list")]
        CannotLowerGas,
        /// #[error("Only the governance contract can do this")]
        Unauthorized,
        /// #[error("You can only send cw20 tokens that have been explicitly allowed by governance")]
        NotOnAllowList,
    }

```

## 2.6 use isc20 transfer video
[CW20 tokens Transfer through IBC using CW20-ICS20 Smart Contract](https://www.youtube.com/watch?v=Yix0BThxTIU)

# 3 ics27 interface
## 3.1 interface
```js
     pub fn execute(
            &self,
            info: MessageInfo,
            msg: ExecuteMsg,
        ) -> Result<Response<CustomMsg>, Error> 

        /// query info for spec QueryMsg
             pub fn query(&self, msg: QueryMsg) -> Result<Vec<u8>, Error>

        /// create a reflect message
             pub fn try_reflect(
            &self,
            info: MessageInfo,
            msgs: Vec<CosmosMsg<CustomMsg>>,
        ) -> Result<Response<CustomMsg>, Error>

        /// create a subcall reflect message
             pub fn try_reflect_subcall(
            &self,
            info: MessageInfo,
            msgs: Vec<SubMsg<CustomMsg>>,
        ) -> Result<Response<CustomMsg>, Error>

        /// change contract owner
             pub fn try_change_owner(
            &self,
            info: MessageInfo,
            new_owner: String,
        ) -> Result<Response<CustomMsg>, Error>

        /// Returns (reflect) account that is attached to this channel,
        /// or none.
        pub fn query_account(&self, channel_id: String) -> AccountResponse

        /// Returns all (channel, reflect_account) pairs.
        /// No pagination - this is a test contract
        pub fn query_list_accounts(&self) -> ListAccountsResponse

        /// query contract owner
        pub fn query_owner(&self) -> OwnerResponse

        /// If there was a previous ReflectSubMsg with this ID, returns cosmwasm_std::Reply
        pub fn query_subcall(&self, id: u64) -> Reply 

        /// This will call out to SpecialQuery::Capitalized
        pub fn query_capitalized(&self, text: String) -> CapitalizedResponse 

        /// Queries the blockchain and returns the result untouched
        pub fn query_chain(&self, request: QueryRequest<SpecialQuery>) -> ChainResponse 

        /// Queries another contract and returns the data
        pub fn query_raw(&self, contract: String, key: Vec<u8>) -> RawResponse 
```
## 3.2 storage
```js
    pub struct Ics27demo {
        /// config static string "config"
        key_config: Vec<u8>,
        ///pending_channel static string "pending"
        key_pending_channel: Vec<u8>,
        ///prefix_accounts static string "accounts"
        prefix_accounts: Vec<u8>,
        ///result static string "result"
        result_prefix: Vec<u8>,
    }
```

## 3.3 struct

```js
    pub struct Addr(String);

    pub struct InstantiateMsg {
        pub reflect_code_id: u64,
    }


    pub struct AccountResponse {
        pub account: Option<String>,
    }

    pub struct ListAccountsResponse {
        pub accounts: Vec<AccountInfo>,
    }

    pub struct AccountInfo {
        pub account: String,
        pub channel_id: String,
    }

    pub enum SpecialQuery {
        Ping {},
        Capitalized { text: String },
    }

    pub enum CustomMsg {
        Debug(String),
        Raw(Vec<u8>),
    }

    pub struct OwnerResponse {
        pub owner: String,
    }

    pub struct MessageInfo {
        /// The `sender` field from `MsgInstantiateContract` and `MsgExecuteContract`.
        /// You can think of this as the address that initiated the action (i.e. the message). What that
        /// means exactly heavily depends on the application.
        ///
        /// The x/wasm module ensures that the sender address signed the transaction or
        /// is otherwise authorized to send the message.
        ///
        /// Additional signers of the transaction that are either needed for other messages or contain unnecessary
        /// signatures are not propagated into the contract.
        pub sender: Addr,
        /// The funds that are sent to the contract as part of `MsgInstantiateContract`
        /// or `MsgExecuteContract`. The transfer is processed in bank before the contract
        /// is executed such that the new balance is visible during contract execution.
        pub funds: Vec<Coin>,
    }

    pub struct SubMsgResponse {
        pub events: Vec<Event>,
        pub data: Option<Vec<u8>>,
    }

    pub struct Event {
        /// The event type. This is renamed to "ty" because "type" is reserved in Rust. This sucks, we know.
        pub ty: String,
        /// The attributes to be included in the event.
        ///
        /// You can learn more about these from [*Cosmos SDK* docs].
        ///
        /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/core/events.html
        pub attributes: Vec<Attribute>,
    }

    pub struct Attribute {
        pub key: String,
        pub value: String,
    }

    pub struct CapitalizedResponse {
        pub text: String,
    }

    pub struct ChainResponse {
        pub data: Vec<u8>,
    }

    pub struct Coin {
        pub denom: String,
        pub amount: u128,
    }

    pub struct RawResponse {
        /// The returned value of the raw query. Empty data can be the
        /// result of a non-existent key or an empty value. We cannot
        /// differentiate those two cases in cross contract queries.
        pub data: Vec<u8>,
    }

    pub enum CosmosMsg<T> {
        BankMsg(BankMsg),
        // by default we use RawMsg, but a contract can override that
        // to call into more app-specific code (whatever they define)
        Custom(T),
        // #[cfg(feature = "staking")]
        Staking(StakingMsg),
        // #[cfg(feature = "staking")]
        Distribution(DistributionMsg),
        // /// A Stargate message encoded the same way as a protobuf [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
        // /// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
        // #[cfg(feature = "stargate")]
        Stargate {
            type_url: String,
            value: Binary,
        },
        // #[cfg(feature = "stargate")]
        Ibc(IbcMsg),
        Wasm(WasmMsg),
        // #[cfg(feature = "stargate")]
        Gov(GovMsg),
    }

    pub struct SubMsg<T = Empty> {
        /// An arbitrary ID chosen by the contract.
        /// This is typically used to match `Reply`s in the `reply` entry point to the submessage.
        pub id: u64,
        pub msg: CosmosMsg<T>,
        /// Gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
        pub gas_limit: Option<u64>,
        pub reply_on: ReplyOn,
    }

    pub enum ReplyOn {
        /// Always perform a callback after SubMsg is processed
        Always,
        /// Only callback if SubMsg returned an error, no callback on success case
        Error,
        /// Only callback if SubMsg was successful, no callback on error case
        Success,
        /// Never make a callback - this is like the original CosmosMsg semantics
        Never,
    }

    pub enum QueryRequest<C> {
        //Bank(BankQuery),
        Custom(C),
        // #[cfg(feature = "staking")]
        Staking(StakingQuery),
        /// A Stargate query is encoded the same way as abci_query, with path and protobuf encoded request data.
        /// The format is defined in [ADR-21](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-021-protobuf-query-encoding.md).
        /// The response is protobuf encoded data directly without a JSON response wrapper.
        /// The caller is responsible for compiling the proper protobuf definitions for both requests and responses.
        #[cfg(feature = "stargate")]
        Stargate {
            /// this is the fully qualified service path used for routing,
            /// eg. custom/cosmos_sdk.x.bank.v1.Query/QueryBalance
            path: String,
            /// this is the expected protobuf message type (not any), binary encoded
            data: Binary,
        },
        #[cfg(feature = "stargate")]
        Ibc(IbcQuery),
        Wasm(WasmQuery),
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Ics27demo {
        /// config static string "config"
        key_config: Vec<u8>,
        ///pending_channel static string "pending"
        key_pending_channel: Vec<u8>,
        ///prefix_accounts static string "accounts"
        prefix_accounts: Vec<u8>,
        ///result static string "result"
        result_prefix: Vec<u8>,
    }

    // pub enum ReflectExecuteMsg {
    //     ReflectMsg { msgs: Vec<CosmosMsg> },
    // }

    // pub enum PacketMsg {
    //     Dispatch { msgs: Vec<CosmosMsg> },
    //     WhoAmI {},
    //     Balances {},
    // }


    pub enum ExecuteMsg {
        ReflectMsg { msgs: Vec<CosmosMsg<CustomMsg>> },
        ReflectSubMsg { msgs: Vec<SubMsg<CustomMsg>> },
        ChangeOwner { owner: String },
    }

    pub enum QueryMsg {
        //#[returns(AccountResponse)]
        Account {
            channel_id: String,
        },
        /// Returns all (channel, reflect_account) pairs.
        /// No pagination - this is a test contract
        //#[returns(ListAccountsResponse)]
        ListAccounts {},

        //#[returns(OwnerResponse)]
        Owner {},
        /// This will call out to SpecialQuery::Capitalized
        //#[returns(CapitalizedResponse)]
        Capitalized {
            text: String,
        },
        /// Queries the blockchain and returns the result untouched
        //#[returns(ChainResponse)]
        Chain {
            request: QueryRequest<SpecialQuery>,
        },
        /// Queries another contract and returns the data
        //#[returns(RawResponse)]
        Raw {
            contract: String,
            key: Vec<u8>,
        },
        /// If there was a previous ReflectSubMsg with this ID, returns cosmwasm_std::Reply
        //#[returns(cosmwasm_std::Reply)]
        SubMsgResult {
            id: u64,
        },
    }
```

## 3.4 error
```js
pub enum Error {
  // let thiserror implement From<StdError> for you
  StdError,
  // this is whatever we want
  ///#[error("Permission denied: the sender is not the current owner")]
  NotCurrentOwner {
      expected: String,
      actual: String,
  },
  ///#[error("Messages empty. Must reflect at least one message")]
  MessagesEmpty,
}
```

# 4 ins721 interface
## 4.1 interface
```js
 /// execute spec set function  for ExecuteMsg
        pub fn execute(&self, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, Error> 

        /// query info for spec QueryMsg
        pub fn query(&self, msg: QueryMsg) -> Result<Vec<u8>, Error>

        //set function list

        // receive nft
        /// Receives a NFT to be IBC transfered away. The `msg` field must
        /// be a binary encoded `IbcOutgoingMsg`.
        pub fn execute_receive_nft(
            &self,
            info: MessageInfo,
            token_id: String,
            sender: String,
            msg: Vec<u8>,
        ) -> Result<Response, Error> 

        // receive proxy nft
        //In the context of CosmWasm and the ReceiveProxyNft function, the term "eyeball" does not have a specific or predefined meaning. It is possible that "eyeball" is used as a variable or parameter name within the code implementation of ReceiveProxyNft or within the surrounding codebase, but without further information, it is difficult to provide a more specific explanation.
        pub fn execute_receive_proxy_nft(
            &self,
            info: MessageInfo,
            eyeball: String,
            msg: Cw721ReceiveMsg,
        ) -> Result<Response, Error> 

        /// Mesages used internally by the contract. These may only be
        /// called by the contract itself.
        fn execute_callback(&self, info: MessageInfo, msg: CallbackMsg) -> Result<Response, Error>

        // pause the contract
        /// Pauses the bridge. Only the pauser may call this. In pausing
        /// the contract, the pauser burns the right to do so again.
        pub fn execute_pause(&self, info: MessageInfo) -> Result<Response, Error> 

        /// query class id by contract
        pub fn query_class_id_for_nft_contract(&self, contract: String) -> Option<ClassId> 

        /// query contract by class id
        pub fn query_nft_contract_for_class_id(&self, class_id: String) -> Option<Addr> 

        /// query class metadata
        pub fn query_class_metadata(&self, class_id: String) -> Option<Class> 

        /// query token metadata
        pub fn query_token_metadata(&self, class_id: String, token_id: String) -> Option<Token> 

        /// query nft owner
        pub fn query_owner(&self, class_id: String, token_id: String) -> OwnerOfResponse 

        /// query pauser admin
        pub fn query_pauser(&self) -> Option<Addr>

        /// query if contract is paused
        pub fn query_paused(&self) -> bool 

        /// query proxy address
        /// The proxy that this contract is receiving NFTs from, if any.
        pub fn query_proxy(&self) -> Option<Addr> 
```

## 4.2 storage
```js
    pub struct Ics721demo {
        /// The code ID we will use for instantiating new cw721s.
        cw721_code_id: u64,
        /// The proxy that this contract is receiving NFTs from, if any.
        proxy: Option<Addr>,
        /// Manages contract pauses.
        pause_orchestrator: PauseOrchestrator,
        /// Maps classID (from NonFungibleTokenPacketData) to the cw721
        /// contract we have instantiated for that classID.
        class_id_to_nft_contract: Mapping<ClassId, Addr>,
        /// Maps cw721 contracts to the classID they were instantiated for.
        nft_contract_to_class_id: Mapping<Addr, ClassId>,
        /// Maps between classIDs and classs. We need to keep this state
        /// ourselves as cw721 contracts do not have class-level metadata.
        class_id_to_class: Mapping<ClassId, Class>,
        /// Maps (class ID, token ID) -> local channel ID. Used to determine
        /// the local channel that NFTs have been sent out on.
        outgoing_class_token_to_channel: Mapping<(ClassId, TokenId), String>,
        /// Same as above, but for NFTs arriving at this contract.
        incoming_class_token_to_channel: Mapping<(ClassId, TokenId), String>,
        /// Maps (class ID, token ID) -> token metadata. Used to store
        /// on-chain metadata for tokens that have arrived from other
        /// chains. When a token arrives, it's metadata (regardless of if it
        /// is `None`) is stored in this map. When the token is returned to
        /// it's source chain, the metadata is removed from the map.
        token_metadata: Mapping<(ClassId, TokenId), Option<Vec<u8>>>,
    }
```

## 4.3 struct
```js

    pub struct Addr(String);
    pub struct TokenId(String);

    /// A token according to the ICS-721 spec.
    pub struct Token {
        /// A unique identifier for the token.
        pub id: TokenId,
        /// Optional URI pointing to off-chain metadata about the token.
        pub uri: Option<String>,
        /// Optional base64 encoded metadata about the token.
        pub data: Option<Vec<u8>>,
    }

    /// A class ID according to the ICS-721 spec. The newtype pattern is
    /// used here to provide some distinction between token and class IDs
    /// in the type system.
    
    
    pub struct ClassId(String);

    pub struct Class {
        /// A unique (from the source chain's perspective) identifier for
        /// the class.
        pub id: ClassId,
        /// Optional URI pointing to off-chain metadata about the class.
        pub uri: Option<String>,
        /// Optional base64 encoded metadata about the class.
        pub data: Option<Vec<u8>>,
    }

    impl TokenId {
        pub(crate) fn new<T>(token_id: T) -> Self
        where
            T: Into<String>,
        {
            Self(token_id.into())
        }
    }

    impl ClassId {
        pub(crate) fn new<T>(class_id: T) -> Self
        where
            T: Into<String>,
        {
            Self(class_id.into())
        }
    }

    pub struct Cw721ReceiveMsg {
        pub sender: String,
        pub token_id: String,
        pub msg: Vec<u8>,
    }

    pub struct MessageInfo {
        pub sender: Addr,
        pub funds: Vec<Coin>,
    }

    pub struct Coin {
        pub denom: String,
        pub amount: u128,
    }

    pub struct OwnerOfResponse {
        /// Owner of the token
        pub owner: String,
        /// If set this address is approved to transfer/send the token as well
        pub approvals: Vec<Approval>,
    }

    pub struct Approval {
        /// Account that can transfer/send the token
        pub spender: String,
        /// When the Approval expires (maybe Expiration::never)
        pub expires: Expiration,
    }

    pub enum Expiration {
        /// AtHeight will expire when `env.block.height` >= height
        AtHeight(u64),
        /// AtTime will expire when `env.block.time` >= time
        AtTime(Timestamp),
        /// Never will never expire. Used to express the empty variant
        Never,
    }

    /// The default (empty value) is to never expire
    impl Default for Expiration {
        fn default() -> Self {
            Expiration::Never {}
        }
    }

    pub enum Admin {
        Address { addr: String },
        Instantiator {},
    }

    pub struct ContractInstantiateInfo {
        pub code_id: u64,
        pub msg: Vec<u8>,
        pub admin: Option<Admin>,
        pub label: String,
    }

    pub struct InstantiateMsg {
        /// Code ID of cw721-ics contract. A new cw721-ics will be
        /// instantiated for each new IBCd NFT classID.
        ///
        /// NOTE: this _must_ correspond to the cw721-base contract. Using
        /// a regular cw721 may cause the ICS 721 interface implemented by
        /// this contract to stop working, and IBCd away NFTs to be
        /// unreturnable as cw721 does not have a mint method in the spec.
        pub cw721_base_code_id: u64,
        /// An optional proxy contract. If a proxy is set the contract
        /// will only accept NFTs from that proxy. The proxy is expected
        /// to implement the cw721 proxy interface defined in the
        /// cw721-proxy crate.
        pub proxy: Option<ContractInstantiateInfo>,
        /// Address that may pause the contract. PAUSER may pause the
        /// contract a single time; in pausing the contract they burn the
        /// right to do so again. A new pauser may be later nominated by
        /// the CosmWasm level admin via a migration.
        pub pauser: Option<String>,
    }

    pub struct VoucherCreation {
        /// The class that these vouchers are being created for.
        pub class: Class,
        /// The tokens to create debt-vouchers for.
        pub tokens: Vec<Token>,
    }

    
    
    pub struct VoucherRedemption {
        /// The class that these vouchers are being redeemed from.
        pub class: Class,
        /// The tokens belonging to `class` that ought to be redeemed.
        pub token_ids: Vec<TokenId>,
    }

    pub enum CallbackMsg {
        CreateVouchers {
            /// The address that ought to receive the NFT. This is a local
            /// address, not a bech32 public key.
            receiver: String,
            /// Information about the vouchers being created.
            create: VoucherCreation,
        },
        RedeemVouchers {
            /// The address that should receive the tokens.
            receiver: String,
            /// Information about the vouchers been redeemed.
            redeem: VoucherRedemption,
        },
        /// Mints a NFT of collection class_id for receiver with the
        /// provided id and metadata. Only callable by this contract.
        Mint {
            /// The class_id to mint for. This must have previously been
            /// created with `SaveClass`.
            class_id: ClassId,
            /// The address that ought to receive the NFTs. This is a
            /// local address, not a bech32 public key.
            receiver: String,
            /// The tokens to mint on the collection.
            tokens: Vec<Token>,
        },
        /// In submessage terms, say a message that results in an error
        /// "returns false" and one that succedes "returns true". Returns
        /// the logical conjunction (&&) of all the messages in operands.
        ///
        /// Under the hood this just executes them in order. We use this
        /// to respond with a single ACK when a message calls for the
        /// execution of both `CreateVouchers` and `RedeemVouchers`.
        Conjunction { operands: Vec<WasmMsg> },
    }

    pub enum WasmMsg {
        /// Dispatches a call to another contract at a known address (with known ABI).
        ///
        /// This is translated to a [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78).
        /// `sender` is automatically filled with the current contract's address.
        Execute {
            contract_addr: String,
            /// msg is the json-encoded ExecuteMsg struct (as raw Binary)
            msg: Vec<u8>,
            funds: Vec<Coin>,
        },
        /// Instantiates a new contracts from previously uploaded Wasm code.
        ///
        /// The contract address is non-predictable. But it is guaranteed that
        /// when emitting the same Instantiate message multiple times,
        /// multiple instances on different addresses will be generated. See also
        /// Instantiate2.
        ///
        /// This is translated to a [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L53-L71).
        /// `sender` is automatically filled with the current contract's address.
        Instantiate {
            admin: Option<String>,
            code_id: u64,
            msg: Vec<u8>,
            funds: Vec<Coin>,
            /// A human-readbale label for the contract
            label: String,
        },
        /// Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to
        /// customize behavior.
        ///
        /// Only the contract admin (as defined in wasmd), if any, is able to make this call.
        ///
        /// This is translated to a [MsgMigrateContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L86-L96).
        /// `sender` is automatically filled with the current contract's address.
        Migrate {
            contract_addr: String,
            /// the code_id of the new logic to place in the given contract
            new_code_id: u64,
            /// msg is the json-encoded MigrateMsg struct that will be passed to the new code
            msg: Vec<u8>,
        },
        /// Sets a new admin (for migrate) on the given contract.
        /// Fails if this contract is not currently admin of the target contract.
        UpdateAdmin {
            contract_addr: String,
            admin: String,
        },
        /// Clears the admin on the given contract, so no more migration possible.
        /// Fails if this contract is not currently admin of the target contract.
        ClearAdmin { contract_addr: String },
    }

    
    
    pub enum ExecuteMsg {
        /// Receives a NFT to be IBC transfered away. The `msg` field must
        /// be a binary encoded `IbcOutgoingMsg`.
        ReceiveNft(Cw721ReceiveMsg),
        /// Pauses the bridge. Only the pauser may call this. In pausing
        /// the contract, the pauser burns the right to do so again.
        Pause {},
        /// Mesages used internally by the contract. These may only be
        /// called by the contract itself.
        Callback(CallbackMsg),
    }

    pub enum QueryMsg {
        /// Gets the classID this contract has stored for a given NFT
        /// contract. If there is no class ID for the provided contract,
        /// returns None.
        //#[returns(Option<crate::token_types::ClassId>)]
        ClassId {
            contract: String,
        },
        /// Gets the NFT contract associated wtih the provided class
        /// ID. If no such contract exists, returns None. Returns
        /// Option<Addr>.
        //#[returns(Option<::cosmwasm_std::Addr>)]
        NftContract {
            class_id: String,
        },
        /// Gets the class level metadata URI for the provided
        /// class_id. If there is no metadata, returns None. Returns
        /// `Option<Class>`.
        //#[returns(Option<crate::token_types::Class>)]
        ClassMetadata {
            class_id: String,
        },
        //#[returns(Option<crate::token_types::Token>)]
        TokenMetadata {
            class_id: String,
            token_id: String,
        },
        /// Gets the owner of the NFT identified by CLASS_ID and
        /// TOKEN_ID. Errors if no such NFT exists. Returns
        /// `cw721::OwnerOfResonse`.
        //#[returns(::cw721::OwnerOfResponse)]
        Owner {
            class_id: String,
            token_id: String,
        },
        /// Gets the address that may pause this contract if one is set.
        //#[returns(Option<::cosmwasm_std::Addr>)]
        Pauser {},
        /// Gets the current pause status.
        //#[returns(bool)]
        Paused {},
        /// Gets this contract's cw721-proxy if one is set.
        //#[returns(Option<::cosmwasm_std::Addr>)]
        Proxy {},
    }

    pub struct PauseOrchestrator {
        pauser: Option<Addr>,
        paused: bool,
    }
```

## 4.4 error
```js
    pub enum Error {
        /// StdError
        StdError,
        /// PauseError
        PauseError,
        /// #[error("unauthorized")]
        Unauthorized {},
        /// #[error("only unordered channels are supported")]
        OrderedChannel {},
        /// #[error("invalid IBC channel version - got ({actual}), expected ({expected})")]
        InvalidVersion { actual: String, expected: String },
        /// #[error("ICS 721 channels may not be closed")]
        CantCloseChannel {},
        /// #[error("unrecognised class ID")]
        UnrecognisedClassId {},
        /// #[error("class ID already exists")]
        ClassIdAlreadyExists {},
        /// #[error("empty class ID")]
        EmptyClassId {},
        /// #[error("must transfer at least one token")]
        NoTokens {},
        /// #[error("optional fields may not be empty if provided")]
        EmptyOptional {},
        /// #[error("unrecognised reply ID")]
        UnrecognisedReplyId {},
        /// ParseReplyError
        ParseReplyError,
        /// #[error("must provide same number of token IDs and URIs")]
        ImbalancedTokenInfo {},
        /// #[error("unexpected uri for classID {class_id} - got ({actual:?}), expected ({expected:?})")]
        ClassUriClash {
            class_id: String,
            expected: Option<String>,
            actual: Option<String>,
        },
        /// #[error("tokenIds, tokenUris, and tokenData must have the same length")]
        TokenInfoLenMissmatch {},
    }
```


# 5 ref
[IBC interfaces for CosmWasm contracts](https://github.com/CosmWasm/cosmwasm/blob/main/IBC.md)

[contracts SEMANTICS](https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md)

[Introduction to cosmwasm](https://github.com/CosmWasm/cosmwasm/blob/main/README.md)

[spec/core/ics-004-channel-and-packet-semantics](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics)

[PSP22 protocol](https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md)

[PSP22 trait](https://github.com/Brushfam/openbrush-contracts/blob/main/contracts/src/traits/psp22/psp22.rs)
