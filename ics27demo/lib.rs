#![cfg_attr(not(feature = "std"), no_std)]
#![feature(default_alloc_error_handler)]

#[ink::contract]
mod ics27 {
    use ibc::ibc::*;
    use ink::prelude::{string::String, string::ToString, vec::Vec};
    use scale::{Decode, Encode};

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Addr(String);

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct InstantiateMsg {
        pub reflect_code_id: u64,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AccountResponse {
        pub account: Option<String>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ListAccountsResponse {
        pub accounts: Vec<AccountInfo>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AccountInfo {
        pub account: String,
        pub channel_id: String,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum SpecialQuery {
        Ping {},
        Capitalized { text: String },
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CustomMsg {
        Debug(String),
        Raw(Vec<u8>),
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct OwnerResponse {
        pub owner: String,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
    pub struct CapitalizedResponse {
        pub text: String,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ChainResponse {
        pub data: Vec<u8>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Coin {
        pub denom: String,
        pub amount: u128,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct RawResponse {
        /// The returned value of the raw query. Empty data can be the
        /// result of a non-existent key or an empty value. We cannot
        /// differentiate those two cases in cross contract queries.
        pub data: Vec<u8>,
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ExecuteMsg {
        ReflectMsg { msgs: Vec<CosmosMsg<CustomMsg>> },
        ReflectSubMsg { msgs: Vec<SubMsg<CustomMsg>> },
        ChangeOwner { owner: String },
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    impl BaseIbc for Ics27demo {
        // ibc base function
        #[ink(message)]
        fn reply(&self, reply: Reply) -> Response {
            Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            }
        }

        #[ink(message)]
        fn migrate(&self, _msg: Empty) -> Response {
            Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            }
        }

        #[ink(message)]
        fn ibc_channel_open(&self, msg: IbcChannelOpenMsg) -> IbcChannelOpenResponse {
            ()
        }

        #[ink(message)]
        fn ibc_channel_connect(&self, msg: IbcChannelConnectMsg) -> IbcBasicResponse {
            IbcBasicResponse {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
            }
        }

        #[ink(message)]
        fn ibc_channel_close(&self, msg: IbcChannelCloseMsg) -> IbcBasicResponse {
            IbcBasicResponse {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
            }
        }

        #[ink(message)]
        fn ibc_packet_receive(
            &self,
            msg: IbcPacketReceiveMsg,
        ) -> Result<IbcReceiveResponse, ibc::ibc::Error> {
            Ok(IbcReceiveResponse {
                acknowledgement: Vec::new(),
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
            })
        }

        #[ink(message)]
        fn ibc_packet_ack(
            &self,
            _msg: IbcPacketAckMsg,
        ) -> Result<IbcBasicResponse, ibc::ibc::Error> {
            Ok(IbcBasicResponse {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
            })
        }

        #[ink(message)]
        fn ibc_packet_timeout(
            &self,
            _msg: IbcPacketTimeoutMsg,
        ) -> Result<IbcBasicResponse, ibc::ibc::Error> {
            Ok(IbcBasicResponse {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
            })
        }
    }

    impl Ics27demo {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(_msg: InstantiateMsg) -> Self {
            Self {
                key_config: Default::default(),
                key_pending_channel: Default::default(),
                prefix_accounts: Default::default(),
                result_prefix: Default::default(),
            }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(InstantiateMsg { reflect_code_id: 0 })
        }

        /// execute spec set function  for ExecuteMsg
        #[ink(message)]
        pub fn execute(
            &mut self,
            info: MessageInfo,
            msg: ExecuteMsg,
        ) -> Result<Response<CustomMsg>, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        /// query info for spec QueryMsg
        #[ink(message)]
        pub fn query(&self, msg: QueryMsg) -> Result<Vec<u8>, Error> {
            Ok(Vec::new())
        }

        /// create a reflect message
        #[ink(message)]
        pub fn try_reflect(
            &mut self,
            info: MessageInfo,
            msgs: Vec<CosmosMsg<CustomMsg>>,
        ) -> Result<Response<CustomMsg>, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        /// create a subcall reflect message
        #[ink(message)]
        pub fn try_reflect_subcall(
            &mut self,
            info: MessageInfo,
            msgs: Vec<SubMsg<CustomMsg>>,
        ) -> Result<Response<CustomMsg>, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        /// change contract owner
        #[ink(message)]
        pub fn try_change_owner(
            &mut self,
            info: MessageInfo,
            new_owner: String,
        ) -> Result<Response<CustomMsg>, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        /// Returns (reflect) account that is attached to this channel,
        /// or none.
        #[ink(message)]
        pub fn query_account(&self, channel_id: String) -> AccountResponse {
            AccountResponse { account: None }
        }

        /// Returns all (channel, reflect_account) pairs.
        /// No pagination - this is a test contract
        #[ink(message)]
        pub fn query_list_accounts(&self) -> ListAccountsResponse {
            ListAccountsResponse {
                accounts: Vec::new(),
            }
        }

        /// query contract owner
        #[ink(message)]
        pub fn query_owner(&self) -> OwnerResponse {
            OwnerResponse {
                owner: "".to_string(),
            }
        }

        /// If there was a previous ReflectSubMsg with this ID, returns cosmwasm_std::Reply
        #[ink(message)]
        pub fn query_subcall(&self, id: u64) -> Reply {
            Reply {
                id: 0,
                result: SubMsgResult::Err("".to_string()),
            }
        }

        /// This will call out to SpecialQuery::Capitalized
        #[ink(message)]
        pub fn query_capitalized(&self, text: String) -> CapitalizedResponse {
            CapitalizedResponse {
                text: "".to_string(),
            }
        }

        /// Queries the blockchain and returns the result untouched
        #[ink(message)]
        pub fn query_chain(&self, request: QueryRequest<SpecialQuery>) -> ChainResponse {
            ChainResponse {
                data: Vec::<u8>::new(),
            }
        }

        /// Queries another contract and returns the data
        #[ink(message)]
        pub fn query_raw(&self, contract: String, key: Vec<u8>) -> RawResponse {
            RawResponse {
                data: Vec::<u8>::new(),
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
