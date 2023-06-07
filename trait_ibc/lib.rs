#![cfg_attr(not(feature = "std"), no_std)]
#![feature(default_alloc_error_handler)]

#[ink::contract]
pub mod ibc {
    use ink::prelude::{string::String, vec, vec::Vec};
    use scale::{Decode, Encode};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};

    #[cfg(feature = "std")]
    use ink::storage::traits::StorageLayout;

    #[derive(Decode, Encode, Default, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Addr(String);

    #[derive(Decode, Encode, Clone)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct IbcEndpoint {
        pub port_id: String,
        pub channel_id: String,
    }

    #[derive(Decode, Encode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum IbcOrder {
        Unordered,
        Ordered,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcTimeout {
        block: Option<IbcTimeoutBlock>,
        timestamp: Option<Timestamp>,
    }

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcTimeoutBlock {
        /// the version that the client is currently on
        /// (eg. after reseting the chain this could increment 1 as height drops to 0)
        pub revision: u64,
        /// block height after which the packet times out.
        /// the height within the given revision
        pub height: u64,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcAcknowledgement {
        pub data: Vec<u8>,
        // we may add more info here in the future (meta-data from the acknowledgement)
        // there have been proposals to extend this type in core ibc for future versions
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum SubMsgResult {
        Ok(SubMsgResponse),
        /// An error type that every custom error created by contract developers can be converted to.
        /// This could potientially have more structure, but String is the easiest.
        Err(String),
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct SubMsgResponse {
        pub events: Vec<Event>,
        pub data: Option<Vec<u8>>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Attribute {
        pub key: String,
        pub value: String,
    }

    #[derive(Decode, Encode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum IbcChannelOpenMsg {
        /// The ChanOpenInit step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        OpenInit { channel: IbcChannel },
        /// The ChanOpenTry step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        OpenTry {
            channel: IbcChannel,
            counterparty_version: String,
        },
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum IbcChannelConnectMsg {
        /// The ChanOpenAck step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        OpenAck {
            channel: IbcChannel,
            counterparty_version: String,
        },
        /// The ChanOpenConfirm step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        OpenConfirm { channel: IbcChannel },
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum IbcChannelCloseMsg {
        /// The ChanCloseInit step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        CloseInit { channel: IbcChannel },
        /// The ChanCloseConfirm step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
        CloseConfirm { channel: IbcChannel }, // pub channel: IbcChannel,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcPacketReceiveMsg {
        pub packet: IbcPacket,
        #[cfg(feature = "ibc3")]
        pub relayer: Addr,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcPacketAckMsg {
        pub acknowledgement: IbcAcknowledgement,
        pub original_packet: IbcPacket,
        #[cfg(feature = "ibc3")]
        pub relayer: Addr,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcPacketTimeoutMsg {
        pub packet: IbcPacket,
        #[cfg(feature = "ibc3")]
        pub relayer: Addr,
    }

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    impl<T> From<BankMsg> for CosmosMsg<T> {
        fn from(msg: BankMsg) -> Self {
            CosmosMsg::Bank(msg)
        }
    }

    impl<T> From<WasmMsg> for CosmosMsg<T> {
        fn from(msg: WasmMsg) -> Self {
            CosmosMsg::Wasm(msg)
        }
    }

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Coin {
        pub denom: String,
        pub amount: u128,
    }

    #[derive(Decode, Encode, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Empty {}

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    impl<T> Default for Response<T> {
        fn default() -> Self {
            Response {
                messages: vec![],
                attributes: vec![],
                events: vec![],
                data: None,
            }
        }
    }

    impl<T> Response<T> {
        pub fn new() -> Self {
            Self::default()
        }

        /// Add an attribute included in the main `wasm` event.
        ///
        /// For working with optional values or optional attributes, see [`add_attributes`][Self::add_attributes].
        pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.attributes.push(Attribute {
                key: key.into(),
                value: value.into(),
            });
            self
        }

        /// This creates a "fire and forget" message, by using `SubMsg::new()` to wrap it,
        /// and adds it to the list of messages to process.
        pub fn add_message(mut self, msg: impl Into<CosmosMsg<T>>) -> Self {
            self.messages.push(SubMsg::new(msg));
            self
        }

        /// This takes an explicit SubMsg (creates via eg. `reply_on_error`)
        /// and adds it to the list of messages to process.
        pub fn add_submessage(mut self, msg: SubMsg<T>) -> Self {
            self.messages.push(msg);
            self
        }

        /// Adds an extra event to the response, separate from the main `wasm` event
        /// that is always created.
        ///
        /// The `wasm-` prefix will be appended by the runtime to the provided type
        /// of event.
        pub fn add_event(mut self, event: Event) -> Self {
            self.events.push(event);
            self
        }

        /// Bulk add attributes included in the main `wasm` event.
        ///
        /// Anything that can be turned into an iterator and yields something
        /// that can be converted into an `Attribute` is accepted.
        ///
        /// ## Examples
        ///
        /// Adding a list of attributes using the pair notation for key and value:
        ///
        /// ```
        /// use cosmwasm_std::Response;
        ///
        /// let attrs = vec![
        ///     ("action", "reaction"),
        ///     ("answer", "42"),
        ///     ("another", "attribute"),
        /// ];
        /// let res: Response = Response::new().add_attributes(attrs.clone());
        /// assert_eq!(res.attributes, attrs);
        /// ```
        ///
        /// Adding an optional value as an optional attribute by turning it into a list of 0 or 1 elements:
        ///
        /// ```
        /// use cosmwasm_std::{Attribute, Response};
        ///
        /// // Some value
        /// let value: Option<String> = Some("sarah".to_string());
        /// let attribute: Option<Attribute> = value.map(|v| Attribute::new("winner", v));
        /// let res: Response = Response::new().add_attributes(attribute);
        /// assert_eq!(res.attributes, [Attribute {
        ///     key: "winner".to_string(),
        ///     value: "sarah".to_string(),
        /// }]);
        ///
        /// // No value
        /// let value: Option<String> = None;
        /// let attribute: Option<Attribute> = value.map(|v| Attribute::new("winner", v));
        /// let res: Response = Response::new().add_attributes(attribute);
        /// assert_eq!(res.attributes.len(), 0);
        /// ```
        pub fn add_attributes<A: Into<Attribute>>(
            mut self,
            attrs: impl IntoIterator<Item = A>,
        ) -> Self {
            self.attributes.extend(attrs.into_iter().map(A::into));
            self
        }

        /// Bulk add "fire and forget" messages to the list of messages to process.
        ///
        /// ## Examples
        ///
        /// ```
        /// use cosmwasm_std::{CosmosMsg, Response};
        ///
        /// fn make_response_with_msgs(msgs: Vec<CosmosMsg>) -> Response {
        ///     Response::new().add_messages(msgs)
        /// }
        /// ```
        pub fn add_messages<M: Into<CosmosMsg<T>>>(
            self,
            msgs: impl IntoIterator<Item = M>,
        ) -> Self {
            self.add_submessages(msgs.into_iter().map(SubMsg::new))
        }

        /// Bulk add explicit SubMsg structs to the list of messages to process.
        ///
        /// ## Examples
        ///
        /// ```
        /// use cosmwasm_std::{SubMsg, Response};
        ///
        /// fn make_response_with_submsgs(msgs: Vec<SubMsg>) -> Response {
        ///     Response::new().add_submessages(msgs)
        /// }
        /// ```
        pub fn add_submessages(mut self, msgs: impl IntoIterator<Item = SubMsg<T>>) -> Self {
            self.messages.extend(msgs.into_iter());
            self
        }

        /// Bulk add custom events to the response. These are separate from the main
        /// `wasm` event.
        ///
        /// The `wasm-` prefix will be appended by the runtime to the provided types
        /// of events.
        pub fn add_events(mut self, events: impl IntoIterator<Item = Event>) -> Self {
            self.events.extend(events.into_iter());
            self
        }

        /// Set the binary data included in the response.
        pub fn set_data(mut self, data: impl Into<Vec<u8>>) -> Self {
            self.data = Some(data.into());
            self
        }
    }

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct SubMsg<T = Empty> {
        /// An arbitrary ID chosen by the contract.
        /// This is typically used to match `Reply`s in the `reply` entry point to the submessage.
        pub id: u64,
        pub msg: CosmosMsg<T>,
        /// Gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
        pub gas_limit: Option<u64>,
        pub reply_on: ReplyOn,
    }

    /// This is used for cases when we use ReplyOn::Never and the id doesn't matter
    pub const UNUSED_MSG_ID: u64 = 0;

    impl<T> SubMsg<T> {
        /// new creates a "fire and forget" message with the pre-0.14 semantics
        pub fn new(msg: impl Into<CosmosMsg<T>>) -> Self {
            SubMsg {
                id: UNUSED_MSG_ID,
                msg: msg.into(),
                reply_on: ReplyOn::Never,
                gas_limit: None,
            }
        }

        /// create a `SubMsg` that will provide a `reply` with the given id if the message returns `Ok`
        pub fn reply_on_success(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
            Self::reply_on(msg.into(), id, ReplyOn::Success)
        }

        /// create a `SubMsg` that will provide a `reply` with the given id if the message returns `Err`
        pub fn reply_on_error(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
            Self::reply_on(msg.into(), id, ReplyOn::Error)
        }

        /// create a `SubMsg` that will always provide a `reply` with the given id
        pub fn reply_always(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
            Self::reply_on(msg.into(), id, ReplyOn::Always)
        }

        /// Add a gas limit to the message.
        /// This gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
        ///
        /// ## Examples
        ///
        /// ```
        /// # use cosmwasm_std::{coins, BankMsg, ReplyOn, SubMsg};
        /// # let msg = BankMsg::Send { to_address: String::from("you"), amount: coins(1015, "earth") };
        /// let sub_msg: SubMsg = SubMsg::reply_always(msg, 1234).with_gas_limit(60_000);
        /// assert_eq!(sub_msg.id, 1234);
        /// assert_eq!(sub_msg.gas_limit, Some(60_000));
        /// assert_eq!(sub_msg.reply_on, ReplyOn::Always);
        /// ```
        pub fn with_gas_limit(mut self, limit: u64) -> Self {
            self.gas_limit = Some(limit);
            self
        }

        fn reply_on(msg: CosmosMsg<T>, id: u64, reply_on: ReplyOn) -> Self {
            SubMsg {
                id,
                msg,
                reply_on,
                gas_limit: None,
            }
        }
    }

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    #[cfg(feature = "ibc3")]
    #[derive(Decode, Encode)]
    pub type IbcChannelOpenResponse = Option<Ibc3ChannelOpenResponse>;

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Ibc3ChannelOpenResponse {
        /// We can set the channel version to a different one than we were called with
        pub version: String,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    // Custom imlementation in order to implement it for all `T`, even if `T` is not `Default`.
    impl<T> Default for IbcBasicResponse<T> {
        fn default() -> Self {
            IbcBasicResponse {
                messages: vec![],
                attributes: vec![],
                events: vec![],
            }
        }
    }

    impl<T> IbcBasicResponse<T> {
        pub fn new() -> Self {
            Self::default()
        }

        /// Add an attribute included in the main `wasm` event.
        pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.attributes.push(Attribute {
                key: key.into(),
                value: value.into(),
            });
            self
        }

        /// This creates a "fire and forget" message, by using `SubMsg::new()` to wrap it,
        /// and adds it to the list of messages to process.
        pub fn add_message(mut self, msg: impl Into<CosmosMsg<T>>) -> Self {
            self.messages.push(SubMsg::new(msg));
            self
        }

        /// This takes an explicit SubMsg (creates via eg. `reply_on_error`)
        /// and adds it to the list of messages to process.
        pub fn add_submessage(mut self, msg: SubMsg<T>) -> Self {
            self.messages.push(msg);
            self
        }

        /// Adds an extra event to the response, separate from the main `wasm` event
        /// that is always created.
        ///
        /// The `wasm-` prefix will be appended by the runtime to the provided type
        /// of event.
        pub fn add_event(mut self, event: Event) -> Self {
            self.events.push(event);
            self
        }

        /// Bulk add attributes included in the main `wasm` event.
        ///
        /// Anything that can be turned into an iterator and yields something
        /// that can be converted into an `Attribute` is accepted.
        ///
        /// ## Examples
        ///
        /// ```
        /// use cosmwasm_std::{attr, IbcBasicResponse};
        ///
        /// let attrs = vec![
        ///     ("action", "reaction"),
        ///     ("answer", "42"),
        ///     ("another", "attribute"),
        /// ];
        /// let res: IbcBasicResponse = IbcBasicResponse::new().add_attributes(attrs.clone());
        /// assert_eq!(res.attributes, attrs);
        /// ```
        pub fn add_attributes<A: Into<Attribute>>(
            mut self,
            attrs: impl IntoIterator<Item = A>,
        ) -> Self {
            self.attributes.extend(attrs.into_iter().map(A::into));
            self
        }

        /// Bulk add "fire and forget" messages to the list of messages to process.
        ///
        /// ## Examples
        ///
        /// ```
        /// use cosmwasm_std::{CosmosMsg, IbcBasicResponse};
        ///
        /// fn make_response_with_msgs(msgs: Vec<CosmosMsg>) -> IbcBasicResponse {
        ///     IbcBasicResponse::new().add_messages(msgs)
        /// }
        /// ```
        pub fn add_messages<M: Into<CosmosMsg<T>>>(
            self,
            msgs: impl IntoIterator<Item = M>,
        ) -> Self {
            self.add_submessages(msgs.into_iter().map(SubMsg::new))
        }

        /// Bulk add explicit SubMsg structs to the list of messages to process.
        ///
        /// ## Examples
        ///
        /// ```
        /// use cosmwasm_std::{SubMsg, IbcBasicResponse};
        ///
        /// fn make_response_with_submsgs(msgs: Vec<SubMsg>) -> IbcBasicResponse {
        ///     IbcBasicResponse::new().add_submessages(msgs)
        /// }
        /// ```
        pub fn add_submessages(mut self, msgs: impl IntoIterator<Item = SubMsg<T>>) -> Self {
            self.messages.extend(msgs.into_iter());
            self
        }

        /// Bulk add custom events to the response. These are separate from the main
        /// `wasm` event.
        ///
        /// The `wasm-` prefix will be appended by the runtime to the provided types
        /// of events.
        pub fn add_events(mut self, events: impl IntoIterator<Item = Event>) -> Self {
            self.events.extend(events.into_iter());
            self
        }
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    impl<T> Default for IbcReceiveResponse<T> {
        fn default() -> Self {
            IbcReceiveResponse {
                acknowledgement: vec![],
                messages: vec![],
                attributes: vec![],
                events: vec![],
            }
        }
    }

    impl<T> IbcReceiveResponse<T> {
        pub fn new() -> Self {
            Self::default()
        }

        /// Set the acknowledgement for this response.
        pub fn set_ack(mut self, ack: impl Into<Vec<u8>>) -> Self {
            self.acknowledgement = ack.into();
            self
        }

        /// Add an attribute included in the main `wasm` event.
        pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.attributes.push(Attribute {
                key: key.into(),
                value: value.into(),
            });
            self
        }

        /// This creates a "fire and forget" message, by using `SubMsg::new()` to wrap it,
        /// and adds it to the list of messages to process.
        pub fn add_message(mut self, msg: impl Into<CosmosMsg<T>>) -> Self {
            self.messages.push(SubMsg::new(msg));
            self
        }

        /// This takes an explicit SubMsg (creates via eg. `reply_on_error`)
        /// and adds it to the list of messages to process.
        pub fn add_submessage(mut self, msg: SubMsg<T>) -> Self {
            self.messages.push(msg);
            self
        }

        /// Adds an extra event to the response, separate from the main `wasm` event
        /// that is always created.
        ///
        /// The `wasm-` prefix will be appended by the runtime to the provided type
        /// of event.
        pub fn add_event(mut self, event: Event) -> Self {
            self.events.push(event);
            self
        }

        /// Bulk add attributes included in the main `wasm` event.
        ///
        /// Anything that can be turned into an iterator and yields something
        /// that can be converted into an `Attribute` is accepted.
        ///
        /// ## Examples
        ///
        /// ```
        /// use cosmwasm_std::{attr, IbcReceiveResponse};
        ///
        /// let attrs = vec![
        ///     ("action", "reaction"),
        ///     ("answer", "42"),
        ///     ("another", "attribute"),
        /// ];
        /// let res: IbcReceiveResponse = IbcReceiveResponse::new().add_attributes(attrs.clone());
        /// assert_eq!(res.attributes, attrs);
        /// ```
        pub fn add_attributes<A: Into<Attribute>>(
            mut self,
            attrs: impl IntoIterator<Item = A>,
        ) -> Self {
            self.attributes.extend(attrs.into_iter().map(A::into));
            self
        }

        /// Bulk add "fire and forget" messages to the list of messages to process.
        ///
        /// ## Examples
        ///
        /// ```
        /// use cosmwasm_std::{CosmosMsg, IbcReceiveResponse};
        ///
        /// fn make_response_with_msgs(msgs: Vec<CosmosMsg>) -> IbcReceiveResponse {
        ///     IbcReceiveResponse::new().add_messages(msgs)
        /// }
        /// ```
        pub fn add_messages<M: Into<CosmosMsg<T>>>(
            self,
            msgs: impl IntoIterator<Item = M>,
        ) -> Self {
            self.add_submessages(msgs.into_iter().map(SubMsg::new))
        }

        /// Bulk add explicit SubMsg structs to the list of messages to process.
        ///
        /// ## Examples
        ///
        /// ```
        /// use cosmwasm_std::{SubMsg, IbcReceiveResponse};
        ///
        /// fn make_response_with_submsgs(msgs: Vec<SubMsg>) -> IbcReceiveResponse {
        ///     IbcReceiveResponse::new().add_submessages(msgs)
        /// }
        /// ```
        pub fn add_submessages(mut self, msgs: impl IntoIterator<Item = SubMsg<T>>) -> Self {
            self.messages.extend(msgs.into_iter());
            self
        }

        /// Bulk add custom events to the response. These are separate from the main
        /// `wasm` event.
        ///
        /// The `wasm-` prefix will be appended by the runtime to the provided types
        /// of events.
        pub fn add_events(mut self, events: impl IntoIterator<Item = Event>) -> Self {
            self.events.extend(events.into_iter());
            self
        }
    }

    #[derive(Encode, Decode, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Ics20Packet {
        /// amount of tokens to transfer is encoded as a string, but limited to u64 max
        pub amount: u128,
        /// the token denomination to be transferred
        pub denom: String,
        /// the recipient address on the destination chain
        pub receiver: String,
        /// the sender address
        pub sender: String,
        /// optional memo for the IBC transfer
        //#[serde(skip_serializing_if = "Option::is_none")]
        pub memo: Option<String>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        // let thiserror implement From<StdError> for you
        StdError,
        InvalidIbcVersion {
            version: String,
        },
        OnlyOrderedChannel,
        ParseError,
        SerializeError,
        PacketAckError,
        TimeoutError,
        UndoReduceChannelBalanceError,

        /// #[error("Got a submessage reply with unknown id: {id}")]
        UnknownReplyId {
            id: u64,
        },
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct TraitIbc {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    #[ink::trait_definition]
    pub trait BaseIbc {
        /// support submessage callbacks
        #[ink(message)]
        fn reply(&mut self, reply: Reply) -> Result<Response, Error>;

        /// in-place contract migrations
        #[ink(message)]
        fn migrate(&self, _msg: Empty) -> Result<Response, Error>;

        /// The first step of a handshake on either chain is ibc_channel_open
        #[ink(message)]
        fn ibc_channel_open(&self, msg: IbcChannelOpenMsg)
            -> Result<IbcChannelOpenResponse, Error>;

        /// Once both sides have returned Ok() to ibc_channel_open, we move onto the second step of the handshake, which is equivalent to ChanOpenAck and ChanOpenConfirm from the spec
        #[ink(message)]
        fn ibc_channel_connect(
            &mut self,
            msg: IbcChannelConnectMsg,
        ) -> Result<IbcBasicResponse, Error>;

        /// Once a channel is closed, whether due to an IBC error, at our request, or at the request of the other side, the following callback is made on the contract, which allows it to take appropriate cleanup action
        #[ink(message)]
        fn ibc_channel_close(&self, msg: IbcChannelCloseMsg) -> Result<IbcBasicResponse, Error>;

        /// After a contract on chain A sends a packet, it is generally processed by the contract on chain B on the other side of the channel. This is done by executing the following entry point on chain B:
        #[ink(message)]
        fn ibc_packet_receive(
            &mut self,
            msg: IbcPacketReceiveMsg,
        ) -> Result<IbcReceiveResponse, Error>;

        /// If chain B successfully received the packet (even if the contract returned an error message), chain A will eventually get an acknowledgement:
        #[ink(message)]
        fn ibc_packet_ack(&mut self, _msg: IbcPacketAckMsg) -> Result<IbcBasicResponse, Error>;

        /// If the packet was not received on chain B before the timeout, we can be certain that it will never be processed there. In such a case, a relayer can return a timeout proof to cancel the pending packet. In such a case the calling contract will never get ibc_packet_ack, but rather ibc_packet_timeout. One of the two calls will eventually get called for each packet that is sent as long as there is a functioning relayer. (In the absence of a functioning relayer, it will never get a response).
        #[ink(message)]
        fn ibc_packet_timeout(
            &mut self,
            _msg: IbcPacketTimeoutMsg,
        ) -> Result<IbcBasicResponse, Error>;
    }

    impl TraitIbc {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    pub fn from_slice<T: DeserializeOwned>(value: &[u8]) -> Result<T, Error> {
        serde_json_wasm::from_slice(value).map_err(|_e| Error::ParseError)
    }

    pub fn from_binary<T: DeserializeOwned>(value: &Vec<u8>) -> Result<T, Error> {
        from_slice(value.as_slice())
    }

    pub fn to_vec<T>(data: &T) -> Result<Vec<u8>, Error>
    where
        T: Serialize + ?Sized,
    {
        serde_json_wasm::to_vec(data).map_err(|_e| Error::SerializeError)
    }

    pub fn to_binary<T>(data: &T) -> Result<Vec<u8>, Error>
    where
        T: Serialize + ?Sized,
    {
        to_vec(data)
    }

    #[inline]
    pub fn attr(key: impl Into<String>, value: impl Into<String>) -> Attribute {
        Attribute {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
    }
}
