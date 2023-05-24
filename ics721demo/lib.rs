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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct VoucherCreation {
        /// The class that these vouchers are being created for.
        pub class: Class,
        /// The tokens to create debt-vouchers for.
        pub tokens: Vec<Token>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct VoucherRedemption {
        /// The class that these vouchers are being redeemed from.
        pub class: Class,
        /// The tokens belonging to `class` that ought to be redeemed.
        pub token_ids: Vec<TokenId>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

        /// execute spec set function  for ExecuteMsg
        #[ink(message)]
        pub fn execute(&self, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, Error> {
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

        //set function list

        // receive nft
        /// Receives a NFT to be IBC transfered away. The `msg` field must
        /// be a binary encoded `IbcOutgoingMsg`.
        #[ink(message)]
        pub fn execute_receive_nft(
            &self,
            info: MessageInfo,
            token_id: String,
            sender: String,
            msg: Vec<u8>,
        ) {
        }

        // receive proxy nft
        //In the context of CosmWasm and the ReceiveProxyNft function, the term "eyeball" does not have a specific or predefined meaning. It is possible that "eyeball" is used as a variable or parameter name within the code implementation of ReceiveProxyNft or within the surrounding codebase, but without further information, it is difficult to provide a more specific explanation.

        #[ink(message)]
        pub fn execute_receive_proxy_nft(
            &self,
            info: MessageInfo,
            eyeball: String,
            msg: Cw721ReceiveMsg,
        ) -> Result<Response, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        /// Mesages used internally by the contract. These may only be
        /// called by the contract itself.
        fn execute_callback(&self, info: MessageInfo, msg: CallbackMsg) -> Result<Response, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        // pause the contract
        /// Pauses the bridge. Only the pauser may call this. In pausing
        /// the contract, the pauser burns the right to do so again.
        #[ink(message)]
        pub fn execute_pause(&self, info: MessageInfo) -> Result<Response, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        //query function list

        /// query class id by contract
        #[ink(message)]
        pub fn query_class_id_for_nft_contract(&self, contract: String) -> Option<ClassId> {
            Some(ClassId("".to_string()))
        }

        /// query contract by class id
        #[ink(message)]
        pub fn query_nft_contract_for_class_id(&self, class_id: String) -> Option<Addr> {
            Some(Addr("".to_string()))
        }

        /// query class metadata
        #[ink(message)]
        pub fn query_class_metadata(&self, class_id: String) -> Option<Class> {
            None
        }

        /// query token metadata
        #[ink(message)]
        pub fn query_token_metadata(&self, class_id: String, token_id: String) -> Option<Token> {
            None
        }

        /// query nft owner
        #[ink(message)]
        pub fn query_owner(&self, class_id: String, token_id: String) -> OwnerOfResponse {
            Default::default()
        }

        /// query pauser admin
        #[ink(message)]
        pub fn query_pauser(&self) -> Option<Addr> {
            Some(Addr("".to_string()))
        }

        /// query if contract is paused
        #[ink(message)]
        pub fn query_paused(&self) -> bool {
            Default::default()
        }

        /// query proxy address
        /// The proxy that this contract is receiving NFTs from, if any.
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
