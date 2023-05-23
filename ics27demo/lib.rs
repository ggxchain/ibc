#![cfg_attr(not(feature = "std"), no_std)]
#![feature(default_alloc_error_handler)]

#[ink::contract]
mod ics27 {
    use ink::prelude::{string::String, vec::Vec};
    use scale::{Decode, Encode};

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct InstantiateMsg {
        pub reflect_code_id: u64,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AccountResponse {
        pub account: Option<String>,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ListAccountsResponse {
        pub accounts: Vec<AccountInfo>,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AccountInfo {
        pub account: String,
        pub channel_id: String,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Ics27demo {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    // pub enum CosmosMsg<T = Empty> {
    //     Bank(BankMsg),
    //     // by default we use RawMsg, but a contract can override that
    //     // to call into more app-specific code (whatever they define)
    //     Custom(T),
    //     #[cfg(feature = "staking")]
    //     Staking(StakingMsg),
    //     #[cfg(feature = "staking")]
    //     Distribution(DistributionMsg),
    //     /// A Stargate message encoded the same way as a protobuf [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
    //     /// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
    //     #[cfg(feature = "stargate")]
    //     Stargate {
    //         type_url: String,
    //         value: Binary,
    //     },
    //     #[cfg(feature = "stargate")]
    //     Ibc(IbcMsg),
    //     Wasm(WasmMsg),
    //     #[cfg(feature = "stargate")]
    //     Gov(GovMsg),
    // }

    // pub enum ReflectExecuteMsg {
    //     ReflectMsg { msgs: Vec<CosmosMsg> },
    // }

    // pub enum PacketMsg {
    //     Dispatch { msgs: Vec<CosmosMsg> },
    //     WhoAmI {},
    //     Balances {},
    // }

    // pub struct IbcChannel {
    //     pub endpoint: IbcEndpoint,
    //     pub counterparty_endpoint: IbcEndpoint,
    //     pub order: IbcOrder,
    //     /// Note: in ibcv3 this may be "", in the IbcOpenChannel handshake messages
    //     pub version: String,
    //     /// The connection upon which this channel was created. If this is a multi-hop
    //     /// channel, we only expose the first hop.
    //     pub connection_id: String,
    // }
    // pub enum IbcChannelOpenMsg {
    //     /// The ChanOpenInit step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    //     OpenInit { channel: IbcChannel },
    //     /// The ChanOpenTry step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    //     OpenTry {
    //         channel: IbcChannel,
    //         counterparty_version: String,
    //     },
    // }
    // pub enum IbcChannelConnectMsg {
    //     /// The ChanOpenAck step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    //     OpenAck {
    //         channel: IbcChannel,
    //         counterparty_version: String,
    //     },
    //     /// The ChanOpenConfirm step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    //     OpenConfirm { channel: IbcChannel },
    // }
    // pub enum IbcChannelCloseMsg {
    //     /// The ChanCloseInit step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    //     CloseInit { channel: IbcChannel },
    //     /// The ChanCloseConfirm step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    //     CloseConfirm { channel: IbcChannel }, // pub channel: IbcChannel,
    // }
    // pub struct IbcPacketReceiveMsg {
    //     pub packet: IbcPacket,
    //     #[cfg(feature = "ibc3")]
    //     pub relayer: Addr,
    // }
    // pub struct IbcPacketAckMsg {
    //     pub acknowledgement: IbcAcknowledgement,
    //     pub original_packet: IbcPacket,
    //     #[cfg(feature = "ibc3")]
    //     pub relayer: Addr,
    // }
    // pub struct IbcPacketTimeoutMsg {
    //     pub packet: IbcPacket,
    //     #[cfg(feature = "ibc3")]
    //     pub relayer: Addr,
    // }

    impl Ics27demo {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(_msg: InstantiateMsg) -> Self {
            Self { value: false }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(InstantiateMsg { reflect_code_id: 0 })
        }

        /// ibc base function
        // #[ink(message)]
        // pub fn reply(reply: Reply) -> Response {}

        // #[ink(message)]
        // pub fn ibc_channel_open(msg: IbcChannelOpenMsg) -> IbcChannelOpenResponse {}

        // #[ink(message)]
        // pub fn ibc_channel_connect(msg: IbcChannelConnectMsg) -> IbcBasicResponse {}

        // #[ink(message)]
        // pub fn ibc_channel_close(msg: IbcChannelCloseMsg) -> IbcBasicResponse {}

        // #[ink(message)]
        // pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> Response {}

        // #[ink(message)]
        // pub fn ibc_packet_receive(
        //     deps: DepsMut,
        //     _env: Env,
        //     msg: IbcPacketReceiveMsg,
        // ) -> Result<IbcReceiveResponse, Never> {
        // }

        // #[ink(message)]
        // pub fn ibc_packet_ack(
        //     _deps: DepsMut,
        //     _env: Env,
        //     _msg: IbcPacketAckMsg,
        // ) -> StdResult<IbcBasicResponse> {
        // }

        // #[ink(message)]
        // pub fn ibc_packet_timeout(
        //     _deps: DepsMut,
        //     _env: Env,
        //     _msg: IbcPacketTimeoutMsg,
        // ) -> StdResult<IbcBasicResponse> {
        // }

        /// query function list
        #[ink(message)]
        pub fn query_account(&self, channel_id: String) -> AccountResponse {
            AccountResponse { account: None }
        }

        #[ink(message)]
        pub fn query_list_accounts(&self) -> ListAccountsResponse {
            ListAccountsResponse {
                accounts: Vec::new(),
            }
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
