#![cfg_attr(not(feature = "std"), no_std)]
#![feature(default_alloc_error_handler)]

#[ink::contract]
mod ics721demo {
    use ibc::ibc::*;
    use ink::prelude::{
        string::{String, ToString},
        vec::Vec,
    };
    use scale::{Decode, Encode};

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Addr(String);

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct TokenId(String);

    /// A token according to the ICS-721 spec.
    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ClassId(String);

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Cw721ReceiveMsg {
        pub sender: String,
        pub token_id: String,
        pub msg: Vec<u8>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct MessageInfo {
        pub sender: Addr,
        pub funds: Vec<Coin>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Coin {
        pub denom: String,
        pub amount: u128,
    }

    #[derive(Decode, Encode, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct OwnerOfResponse {
        /// Owner of the token
        pub owner: String,
        /// If set this address is approved to transfer/send the token as well
        pub approvals: Vec<Approval>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Approval {
        /// Account that can transfer/send the token
        pub spender: String,
        /// When the Approval expires (maybe Expiration::never)
        pub expires: Expiration,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Admin {
        Address { addr: String },
        Instantiator {},
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ContractInstantiateInfo {
        pub code_id: u64,
        pub msg: Vec<u8>,
        pub admin: Option<Admin>,
        pub label: String,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    // #[cw_serde]
    // pub enum ExecuteMsg {
    //     /// Receives a NFT to be IBC transfered away. The `msg` field must
    //     /// be a binary encoded `IbcOutgoingMsg`.
    //     ReceiveNft(cw721::Cw721ReceiveMsg),

    //     /// Pauses the bridge. Only the pauser may call this. In pausing
    //     /// the contract, the pauser burns the right to do so again.
    //     Pause {},

    //     /// Mesages used internally by the contract. These may only be
    //     /// called by the contract itself.
    //     Callback(CallbackMsg),
    // }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Ics721demo {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl BaseIbc for Ics721demo {
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

    impl Ics721demo {
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

        //set function list

        // receive nft
        #[ink(message)]
        pub fn execute_receive_nft(
            &self,
            info: MessageInfo,
            token_id: String,
            sender: String,
            msg: Vec<u8>,
        ) {
        }

        // receive nft
        #[ink(message)]
        pub fn execute_receive_proxy_nft(
            &self,
            info: MessageInfo,
            eyeball: String,
            msg: Cw721ReceiveMsg,
        ) {
        }

        // pause the contract
        #[ink(message)]
        pub fn execute_pause(&self, info: MessageInfo) {}

        // ibc function list
        // #[ink(message)]
        // pub fn ibc_channel_open(
        //     msg: IbcChannelOpenMsg,
        // ) -> Result<IbcChannelOpenResponse, ContractError> {
        // }

        // #[ink(message)]
        // pub fn ibc_channel_connect(
        //     msg: IbcChannelConnectMsg,
        // ) -> Result<IbcBasicResponse, ContractError> {
        // }

        // #[ink(message)]
        // pub fn ibc_channel_close(
        //     msg: IbcChannelCloseMsg,
        // ) -> Result<IbcBasicResponse, ContractError> {
        // }

        // #[ink(message)]
        // pub fn ibc_packet_receive(msg: IbcPacketReceiveMsg) -> Result<IbcReceiveResponse, Never> {}

        // #[ink(message)]
        // pub fn ibc_packet_ack(ack: IbcPacketAckMsg) -> Result<IbcBasicResponse, ContractError> {}

        // #[ink(message)]
        // pub fn ibc_packet_timeout(
        //     msg: IbcPacketTimeoutMsg,
        // ) -> Result<IbcBasicResponse, ContractError> {
        // }

        // #[ink(message)]
        // pub fn reply(reply: Reply) -> Result<Response, ContractError> {}

        // #[ink(message)]
        // pub fn migrate(
        //     msg: MigrateMsg,
        // ) -> Result<Response, ContractError> {
        // }

        //query function list

        #[ink(message)]
        pub fn query_class_id_for_nft_contract(&self, contract: String) -> Option<ClassId> {
            Some(ClassId("".to_string()))
        }
        #[ink(message)]
        pub fn query_nft_contract_for_class_id(&self, class_id: String) -> Option<Addr> {
            Some(Addr("".to_string()))
        }
        #[ink(message)]
        pub fn query_class_metadata(&self, class_id: String) -> Option<Class> {
            None
        }

        #[ink(message)]
        pub fn query_token_metadata(&self, class_id: String, token_id: String) -> Option<Token> {
            None
        }

        #[ink(message)]
        pub fn query_owner(&self, class_id: String, token_id: String) -> OwnerOfResponse {
            Default::default()
        }

        #[ink(message)]
        pub fn query_pauser(&self) -> Option<Addr> {
            Some(Addr("".to_string()))
        }

        #[ink(message)]
        pub fn query_paused(&self) -> bool {
            Default::default()
        }

        #[ink(message)]
        pub fn query_proxy(&self) -> Option<Addr> {
            Some(Addr("".to_string()))
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
