
# 1 ibc infomation
## 1.1 Channel lifecycle management

![](https://github.com/cosmos/ibc/raw/main/spec/core/ics-004-channel-and-packet-semantics/channel-state-machine.png)


# 2 ics20 interface
## 2.1 ink! interface
```js
        /// transfer token, This allows us to transfer *exactly one* native token
        pub fn execute_transfer(
            &self,
            msg: TransferMsg,
            amount: Amount,
            sender: Addr,
        ) -> Result<Response, Error> ;


        /// Return the port ID bound by this contract.
        pub fn query_port(&self) -> PortResponse ;
```

## 2.2 p2p22 interface  (open brach ink! Fungible Token Standard for Substrate's contracts pallet)
```js
/// Contract module which provides a basic implementation of multiple token types.
/// A single deployed contract may include any combination of fungible tokens,
/// non-fungible tokens or other configurations (e.g. semi-fungible tokens).
#[openbrush::trait_definition]
pub trait PSP37 {
    /// Returns the amount of tokens of token type `id` owned by `account`.
    ///
    /// If `id` is `None` returns the total number of `owner`'s tokens.
    #[ink(message)]
    fn balance_of(&self, owner: AccountId, id: Option<Id>) -> Balance;

    /// Returns the total amount of token type `id` in the supply.
    ///
    /// If `id` is `None` returns the total number of tokens.
    #[ink(message)]
    fn total_supply(&self, id: Option<Id>) -> Balance;

    /// Returns amount of `id` token of `owner` that `operator` can withdraw
    /// If `id` is `None` returns allowance `Balance::MAX` of all tokens of `owner`
    #[ink(message)]
    fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<Id>) -> Balance;

    /// Allows `operator` to withdraw the `id` token from the caller's account
    /// multiple times, up to the `value` amount.
    /// If this function is called again it overwrites the current allowance with `value`
    /// If `id` is `None` approves or disapproves the operator for all tokens of the caller.
    ///
    /// An `Approval` event is emitted.
    #[ink(message)]
    fn approve(&mut self, operator: AccountId, id: Option<Id>, value: Balance) -> Result<(), PSP37Error>;

    /// Transfers `value` of `id` token from `caller` to `to`
    ///
    /// On success a `TransferSingle` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `TransferToZeroAddress` error if recipient is zero account.
    ///
    /// Returns `NotAllowed` error if transfer is not approved.
    ///
    /// Returns `InsufficientBalance` error if `caller` doesn't contain enough balance.
    ///
    /// Returns `SafeTransferCheckFailed` error if `to` doesn't accept transfer.
    #[ink(message)]
    fn transfer(&mut self, to: AccountId, id: Id, value: Balance, data: Vec<u8>) -> Result<(), PSP37Error>;

    /// Transfers `amount` tokens of token type `id` from `from` to `to`. Also some `data` can be passed.
    ///
    /// On success a `TransferSingle` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `TransferToZeroAddress` error if recipient is zero account.
    ///
    /// Returns `NotAllowed` error if transfer is not approved.
    ///
    /// Returns `InsufficientBalance` error if `from` doesn't contain enough balance.
    ///
    /// Returns `SafeTransferCheckFailed` error if `to` doesn't accept transfer.
    #[ink(message)]
    fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        id: Id,
        amount: Balance,
        data: Vec<u8>,
    ) -> Result<(), PSP37Error>;
}
```

## 2.2 struct
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

    pub enum TimeoutHeight {
    #[codec(index = 0)]
    Never,
    #[codec(index = 1)]
    At(runtime_types::ibc::core::ics02_client::height::Height),
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

# 3 ics27 interface
## 3.1 interface
```js
        /// create a reflect message
        pub fn try_reflect(
            &self,
            info: MessageInfo,
            msgs: Vec<CosmosMsg<CustomMsg>>,
        ) -> Result<Response<CustomMsg>, Error>
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

[PSP37 protocol](https://github.com/w3f/PSPs/blob/master/PSPs/psp-37.md)


[PSP37 trait](https://github.com/Brushfam/openbrush-contracts/blob/main/contracts/src/traits/psp37/psp37.rs)