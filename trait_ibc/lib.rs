#![cfg_attr(not(feature = "std"), no_std)]
#![feature(default_alloc_error_handler)]

#[ink::contract]
pub mod ibc {
    use ink::prelude::{string::String, vec::Vec};
    use scale::{Decode, Encode};

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Addr(String);

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcEndpoint {
        pub port_id: String,
        pub channel_id: String,
    }

    #[derive(Decode, Encode)]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcTimeout {
        block: Option<IbcTimeoutBlock>,
        timestamp: Option<Timestamp>,
    }

    #[derive(Decode, Encode)]
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

    #[derive(Decode, Encode)]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CosmosMsg<T> {
        //(BankMsg),
        // by default we use RawMsg, but a contract can override that
        // to call into more app-specific code (whatever they define)
        Custom(T),
        // #[cfg(feature = "staking")]
        // Staking(StakingMsg),
        // #[cfg(feature = "staking")]
        // Distribution(DistributionMsg),
        // /// A Stargate message encoded the same way as a protobuf [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
        // /// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
        // #[cfg(feature = "stargate")]
        // Stargate {
        //     type_url: String,
        //     value: Binary,
        // },
        // #[cfg(feature = "stargate")]
        // Ibc(IbcMsg),
        // Wasm(WasmMsg),
        // #[cfg(feature = "stargate")]
        // Gov(GovMsg),
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Empty;

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

    #[derive(Decode, Encode)]
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

    #[derive(Decode, Encode)]
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

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        // let thiserror implement From<StdError> for you
        StdError,
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
        #[ink(message)]
        fn reply(&self, reply: Reply) -> Response;

        #[ink(message)]
        fn migrate(&self, _msg: Empty) -> Response;

        #[ink(message)]
        fn ibc_channel_open(&self, msg: IbcChannelOpenMsg) -> IbcChannelOpenResponse;

        #[ink(message)]
        fn ibc_channel_connect(&self, msg: IbcChannelConnectMsg) -> IbcBasicResponse;

        #[ink(message)]
        fn ibc_channel_close(&self, msg: IbcChannelCloseMsg) -> IbcBasicResponse;

        #[ink(message)]
        fn ibc_packet_receive(&self, msg: IbcPacketReceiveMsg)
            -> Result<IbcReceiveResponse, Error>;

        #[ink(message)]
        fn ibc_packet_ack(&self, _msg: IbcPacketAckMsg) -> Result<IbcBasicResponse, Error>;

        #[ink(message)]
        fn ibc_packet_timeout(&self, _msg: IbcPacketTimeoutMsg) -> Result<IbcBasicResponse, Error>;
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

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
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
