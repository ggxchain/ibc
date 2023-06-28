#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]
#![feature(default_alloc_error_handler)]

use ink::env::{chain_extension::FromStatusCode, DefaultEnvironment, Environment};
use ink::prelude::vec::Vec;

/// General result type.
pub type Result<T> = core::result::Result<T, IBCICS20Error>;

type DefaultAccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type DefaultBalance = <ink::env::DefaultEnvironment as Environment>::Balance;

#[ink::chain_extension]
pub trait IBCICS20Extension {
    type ErrorCode = IBCICS20Error;

    #[ink(extension = 0x20001)]
    fn raw_tranfer(input: [u8; 4096]) -> Result<()>;

    #[ink(extension = 0x20002)]
    fn raw_tranfer_ex(input: Vec<u8>) -> Result<()>;

    // PSP22 Metadata interfaces

    #[ink(extension = 0x30001)]
    fn token_name(asset_id: u32) -> Result<Vec<u8>>;

    #[ink(extension = 0x30002)]
    fn token_symbol(asset_id: u32) -> Result<Vec<u8>>;

    #[ink(extension = 0x30003)]
    fn token_decimals(asset_id: u32) -> Result<u8>;

    // PSP22 interface queries

    #[ink(extension = 0x30004)]
    fn total_supply(asset_id: u32) -> Result<DefaultBalance>;

    #[ink(extension = 0x30005)]
    fn balance_of(asset_id: u32, owner: DefaultAccountId) -> Result<DefaultBalance>;

    #[ink(extension = 0x30006)]
    fn allowance(
        asset_id: u32,
        owner: DefaultAccountId,
        spender: DefaultAccountId,
    ) -> Result<DefaultBalance>;

    // PSP22 transfer
    #[ink(extension = 0x30007)]
    fn transfer(asset_id: u32, to: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    // PSP22 transfer_from
    #[ink(extension = 0x30008)]
    fn transfer_from(
        asset_id: u32,
        from: DefaultAccountId,
        to: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 approve
    #[ink(extension = 0x30009)]
    fn approve(asset_id: u32, spender: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    // PSP22 increase_allowance
    #[ink(extension = 0x3000a)]
    fn increase_allowance(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 decrease_allowance
    #[ink(extension = 0x3000b)]
    fn decrease_allowance(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum IBCICS20Error {
    FailIBCCall,
    FailScaleCode,
}

impl FromStatusCode for IBCICS20Error {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::FailIBCCall),
            _ => panic!("encountered unknown status code"),
        }
    }
}

// todo(smith) need parse scale error, this is for test
impl From<scale::Error> for IBCICS20Error {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum IBCDefaultEnvironment {}

impl Environment for IBCDefaultEnvironment {
    const MAX_EVENT_TOPICS: usize = <DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <DefaultEnvironment as Environment>::AccountId;
    type Balance = <DefaultEnvironment as Environment>::Balance;
    type Hash = <DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = IBCICS20Extension;
}

#[openbrush::contract(env = crate::IBCDefaultEnvironment)]
pub mod my_psp22_wrapper {
    use core::str::FromStr;
    use core::time::Duration;
    use trait_ibc::ibc::*;

    use ibc::core::ics24_host::identifier::ChannelId;
    use ibc::core::ics24_host::identifier::PortId;
    use ibc::signer::Signer;
    use ink::prelude::borrow::ToOwned;
    use ink::prelude::{
        string::{String, ToString},
        vec,
        vec::Vec,
    };
    use ink::storage::Mapping;
    use openbrush::{contracts::psp22::extensions::wrapper::*, traits::Storage};
    use scale::{Decode, Encode};

    use serde::{Deserialize, Serialize};

    use core::fmt::Formatter;

    pub const ICS20_VERSION: &str = "ics20-1";
    pub const ICS20_ORDERING: IbcOrder = IbcOrder::Unordered;

    const RECEIVE_ID: u64 = 1337;
    const ACK_FAILURE_ID: u64 = 0xfa17;

    #[cfg(feature = "std")]
    use ink::storage::traits::StorageLayout;

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct MessageInfo {
        pub sender: Addr,
        pub funds: Vec<Coin>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Cw20Coin {
        pub address: String,
        pub amount: u128,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Cw20ReceiveMsg {
        pub sender: String,
        pub amount: u128,
        pub msg: Vec<u8>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AllowMsg {
        pub contract: String,
        pub gas_limit: Option<u64>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Clone, Debug, PartialEq, Eq)]
    pub enum TimeoutHeight {
        Never,
        At(ibc::core::ics02_client::height::Height),
    }

    #[derive(Decode, Encode, Clone, Debug, PartialEq, Eq)]
    pub struct MsgTransfer<C = Coin> {
        /// the port on which the packet will be sent
        pub source_port: PortId,
        /// the channel by which the packet will be sent
        pub source_channel: ChannelId,
        /// the tokens to be transferred
        pub token: C,
        /// the sender address
        pub sender: Signer,
        /// the recipient address on the destination chain
        pub receiver: Signer,
        /// Timeout height relative to the current block height.
        /// The timeout is disabled when set to None.
        pub timeout_height: TimeoutHeight,
        /// Timeout timestamp relative to the current block timestamp.
        /// The timeout is disabled when set to 0.
        pub timeout_timestamp: ibc::timestamp::Timestamp,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Clone)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct ChannelInfo {
        /// id of this channel
        pub id: String,
        /// the remote channel/port we connect to
        pub counterparty_endpoint: IbcEndpoint,
        /// the connection this exists on (you can use to query client/consensus info)
        pub connection_id: String,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AllowedInfo {
        pub contract: String,
        pub gas_limit: Option<u64>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PortResponse {
        pub port_id: String,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ListChannelsResponse {
        pub channels: Vec<ChannelInfo>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ChannelResponse {
        /// Information on the channel's connection
        pub info: ChannelInfo,
        /// How many tokens we currently have pending over this channel
        pub balances: Vec<Amount>,
        /// The total number of tokens that have been sent over this channel
        /// (even if many have been returned, so balance is low)
        pub total_sent: Vec<Amount>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ConfigResponse {
        pub default_timeout: u64,
        pub default_gas_limit: Option<u64>,
        pub gov_contract: String,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AllowedResponse {
        pub is_allowed: bool,
        pub gas_limit: Option<u64>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ListAllowedResponse {
        pub allow: Vec<AllowedInfo>,
    }

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[derive(Decode, Encode, Default)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct Config {
        pub default_timeout: u64,
        pub default_gas_limit: Option<u64>,
    }

    #[derive(Decode, Encode, Default)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct ReplyArgs {
        pub channel: String,
        pub denom: String,
        pub amount: u128,
    }

    #[derive(Decode, Encode, Default)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct AllowInfo {
        pub gas_limit: Option<u64>,
    }

    #[derive(Decode, Encode, Default)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub struct ChannelState {
        pub outstanding: u128,
        pub total_sent: u128,
    }

    #[derive(Decode, Encode, Deserialize, Serialize)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub enum Ics20Ack {
        Result(Vec<u8>),
        Error(String),
    }

    // create a serialized success message
    fn ack_success() -> Vec<u8> {
        let res = Ics20Ack::Result(vec![0x1]);
        trait_ibc::ibc::to_binary(&res).unwrap()
    }

    // create a serialized error message
    fn ack_fail(err: String) -> Vec<u8> {
        let res = Ics20Ack::Error(err);
        trait_ibc::ibc::to_binary(&res).unwrap()
    }

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub enum Expiration {
        /// AtHeight will expire when `env.block.height` >= height
        AtHeight(u64),
        /// AtTime will expire when `env.block.time` >= time
        AtTime(Timestamp),
        /// Never will never expire. Used to express the empty variant
        Never {},
    }

    #[derive(Decode, Encode, Serialize, Deserialize)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    pub enum Cw20ExecuteMsg {
        /// Transfer is a base message to move tokens to another account without triggering actions
        Transfer { recipient: String, amount: u128 },
        /// Burn is a base message to destroy tokens forever
        Burn { amount: u128 },
        /// Send is a base message to transfer tokens to a contract and trigger an action
        /// on the receiving contract.
        Send {
            contract: String,
            amount: u128,
            msg: Vec<u8>,
        },
        /// Only with "approval" extension. Allows spender to access an additional amount tokens
        /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
        /// expiration with this one.
        IncreaseAllowance {
            spender: String,
            amount: u128,
            expires: Option<Expiration>,
        },
        /// Only with "approval" extension. Lowers the spender's access of tokens
        /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
        /// allowance expiration with this one.
        DecreaseAllowance {
            spender: String,
            amount: u128,
            expires: Option<Expiration>,
        },
        /// Only with "approval" extension. Transfers amount tokens from owner -> recipient
        /// if `env.sender` has sufficient pre-approval.
        TransferFrom {
            owner: String,
            recipient: String,
            amount: u128,
        },
        /// Only with "approval" extension. Sends amount tokens from owner -> contract
        /// if `env.sender` has sufficient pre-approval.
        SendFrom {
            owner: String,
            contract: String,
            amount: u128,
            msg: Vec<u8>,
        },
        /// Only with "approval" extension. Destroys tokens forever
        BurnFrom { owner: String, amount: u128 },
        /// Only with the "mintable" extension. If authorized, creates amount new tokens
        /// and adds to the recipient balance.
        Mint { recipient: String, amount: u128 },
        /// Only with the "mintable" extension. The current minter may set
        /// a new minter. Setting the minter to None will remove the
        /// token's minter forever.
        UpdateMinter { new_minter: Option<String> },
        /// Only with the "marketing" extension. If authorized, updates marketing metadata.
        /// Setting None/null for any of these will leave it unchanged.
        /// Setting Some("") will clear this field on the contract storage
        UpdateMarketing {
            /// A URL pointing to the project behind this token.
            project: Option<String>,
            /// A longer description of the token and it's utility. Designed for tooltips or such
            description: Option<String>,
            /// The address (if any) who can update this data structure
            marketing: Option<String>,
        },
        // /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
        //UploadLogo(Logo),
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// IBCError
        IBCError(trait_ibc::ibc::Error),

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

        /// # channelinfo not found for channel_id
        ChannelInfoNotFound,

        /// # channel Denom not found for channel_id
        ChannelTokenDenomNotFound,
    }

    #[cfg(not(feature = "std"))]
    impl core::fmt::Display for Error {
        fn fmt(&self, _f: &mut Formatter<'_>) -> core::fmt::Result {
            Ok(())
        }
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        wrapper: wrapper::Data,

        /// contract admin
        admin: Addr,
        /// isc20_config
        config: Config,
        /// Used to pass info from the ibc_packet_receive to the reply handler
        reply_args: ReplyArgs,
        /// static info on one channel that doesn't change
        channel_info: Mapping<String, ChannelInfo>,
        /// channel_token_denom list
        channel_token_denom: Mapping<String, Vec<String>>,
        /// indexed by (channel_id, denom) maintaining the balance of the channel in that currency
        channel_state: Mapping<(String, String), ChannelState>,
        /// Every cw20 contract we allow to be sent is stored here, possibly with a gas_limit
        allow_list: Mapping<Addr, AllowInfo>,
    }

    impl PSP22 for Contract {}

    impl PSP22Wrapper for Contract {}

    impl BaseIbc for Contract {
        // ibc base function
        #[ink(message)]
        fn reply(&mut self, reply: Reply) -> Result<Response, trait_ibc::ibc::Error> {
            //todo(smith)
            match reply.id {
                RECEIVE_ID => match reply.result {
                    SubMsgResult::Ok(_) => Ok(Response::new()),
                    SubMsgResult::Err(err) => {
                        // Important design note:  with ibcv2 and wasmd 0.22 we can implement this all much easier.
                        // No reply needed... the receive function and submessage should return error on failure and all
                        // state gets reverted with a proper app-level message auto-generated

                        // Since we need compatibility with Juno (Jan 2022), we need to ensure that optimisitic
                        // state updates in ibc_packet_receive get reverted in the (unlikely) chance of an
                        // error while sending the token

                        // However, this requires passing some state between the ibc_packet_receive function and
                        // the reply handler. We do this with a singleton, with is "okay" for IBC as there is no
                        // reentrancy on these functions (cannot be called by another contract). This pattern
                        // should not be used for ExecuteMsg handlers
                        match self.undo_reduce_channel_balance(
                            self.reply_args.channel.clone().as_str(),
                            self.reply_args.denom.clone().as_str(),
                            self.reply_args.amount,
                        ) {
                            Err(_) => {
                                return Err(trait_ibc::ibc::Error::UndoReduceChannelBalanceError);
                            }
                            _ => {}
                        };

                        return Ok(Response::new().set_data(ack_fail(err)));
                    }
                },
                ACK_FAILURE_ID => match reply.result {
                    SubMsgResult::Ok(_) => Ok(Response::new()),
                    SubMsgResult::Err(err) => Ok(Response::new().set_data(ack_fail(err))),
                },
                _ => Err(trait_ibc::ibc::Error::UnknownReplyId { id: reply.id }),
            }
        }

        #[ink(message)]
        fn migrate(&self, _msg: Empty) -> Result<Response, trait_ibc::ibc::Error> {
            //todo(smith) with a proxy address
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        #[ink(message)]
        fn ibc_channel_open(
            &self,
            msg: IbcChannelOpenMsg,
        ) -> Result<IbcChannelOpenResponse, trait_ibc::ibc::Error> {
            let (channel, opt_counterparty_version) = match msg {
                IbcChannelOpenMsg::OpenInit { channel } => (channel, None),
                IbcChannelOpenMsg::OpenTry {
                    channel,
                    counterparty_version,
                } => (channel, Some(counterparty_version)),
            };

            let _ = self.enforce_order_and_version(
                channel,
                opt_counterparty_version.as_ref().map(AsRef::as_ref),
            )?;

            Ok(())
        }

        #[ink(message)]
        fn ibc_channel_connect(
            &mut self,
            msg: IbcChannelConnectMsg,
        ) -> Result<IbcBasicResponse, trait_ibc::ibc::Error> {
            let (channel, opt_counterparty_version) = match msg {
                IbcChannelConnectMsg::OpenAck {
                    channel,
                    counterparty_version,
                } => (channel, Some(counterparty_version)),
                IbcChannelConnectMsg::OpenConfirm { channel } => (channel, None),
            };

            let _ = self.enforce_order_and_version(
                channel.clone(),
                opt_counterparty_version.as_deref().map(AsRef::as_ref),
            )?;

            let info = ChannelInfo {
                id: channel.endpoint.channel_id,
                counterparty_endpoint: channel.counterparty_endpoint,
                connection_id: channel.connection_id,
            };
            self.channel_info.insert(&*info.id, &info);

            Ok(IbcBasicResponse {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
            })
        }

        #[ink(message)]
        fn ibc_channel_close(
            &self,
            msg: IbcChannelCloseMsg,
        ) -> Result<IbcBasicResponse, trait_ibc::ibc::Error> {
            // TODO: what to do here?
            // we will have locked funds that need to be returned somehow
            Ok(IbcBasicResponse {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
            })
        }

        #[ink(message)]
        fn ibc_packet_receive(
            &mut self,
            msg: IbcPacketReceiveMsg,
        ) -> Result<IbcReceiveResponse, trait_ibc::ibc::Error> {
            let packet = msg.packet;

            self.do_ibc_packet_receive(&packet).or_else(|err| {
                Ok(IbcReceiveResponse::new()
                    .set_ack(ack_fail(err.to_string()))
                    .add_attributes(vec![
                        attr("action", "receive"),
                        attr("success", "false"),
                        attr("error", err.to_string()),
                    ]))
            })
        }

        #[ink(message)]
        fn ibc_packet_ack(
            &mut self,
            msg: IbcPacketAckMsg,
        ) -> Result<IbcBasicResponse, trait_ibc::ibc::Error> {
            let ics20msg: Ics20Ack = from_binary(&msg.acknowledgement.data)?;
            match ics20msg {
                Ics20Ack::Result(_) => match self.on_packet_success(msg.original_packet) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(trait_ibc::ibc::Error::PacketAckError),
                },
                Ics20Ack::Error(err) => match self.on_packet_failure(msg.original_packet, err) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(trait_ibc::ibc::Error::PacketAckError),
                },
            }
        }

        #[ink(message)]
        fn ibc_packet_timeout(
            &mut self,
            msg: IbcPacketTimeoutMsg,
        ) -> Result<IbcBasicResponse, trait_ibc::ibc::Error> {
            // Ok(IbcBasicResponse {
            //     messages: Vec::new(),
            //     attributes: Vec::new(),
            //     events: Vec::new(),
            // })

            // TODO: trap error like in receive? (same question as ack above)
            let packet = msg.packet;
            match self.on_packet_failure(packet, "timeout".to_string()) {
                Ok(v) => Ok(v),
                Err(e) => Err(trait_ibc::ibc::Error::TimeoutError),
            }
        }
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(token_address: AccountId, msg: InitMsg) -> Self {
            let mut instance = Self::default();

            instance._init(token_address);

            instance
        }

        /// Exposes the `_recover` function for message caller
        #[ink(message)]
        pub fn recover(&mut self) -> Result<Balance, PSP22Error> {
            self._recover(Self::env().caller())
        }

        /// execute spec set function  for ExecuteMsg
        #[ink(message)]
        pub fn execute(&mut self, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, Error> {
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

        // set function list

        /// receive token, This accepts a properly-encoded ReceiveMsg from a cw20 contract
        #[ink(message)]
        pub fn execute_receive(
            &mut self,
            info: MessageInfo,
            wrapper: Cw20ReceiveMsg,
        ) -> Result<Response, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        /// transfer token, This allows us to transfer *exactly one* native token
        #[ink(message)]
        pub fn execute_transfer(
            &mut self,
            msg: TransferMsg,
            amount: Amount,
            sender: Addr,
        ) -> Result<Response, Error> {
            // construct MsgTransfer

            match amount {
                Amount::Native(coin) => {
                    let msg = MsgTransfer {
                        source_port: PortId::transfer(),
                        source_channel: ChannelId::from_str(&msg.channel).unwrap_or_default(),
                        token: coin,
                        sender: Signer::from_str(sender.as_str()).unwrap(),
                        receiver: Signer::from_str(&msg.remote_address).unwrap(),
                        timeout_height: TimeoutHeight::Never,
                        timeout_timestamp: ibc::timestamp::Timestamp::default(),
                    };
                    let raw_data = msg.encode();

                    let rt = self.env().extension().raw_tranfer_ex(raw_data);
                }
                _ => {}
            }

            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        /// This must be called by gov_contract, will allow a new cw20 token to be sent
        //// The gov contract can allow new contracts, or increase the gas limit on existing contracts.
        /// It cannot block or reduce the limit to avoid forcible sticking tokens in the channel.
        #[ink(message)]
        pub fn execute_allow(
            &mut self,
            info: MessageInfo,
            allow: AllowMsg,
        ) -> Result<Response, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        /// update admin address, Change the admin (must be called by current admin)
        #[ink(message)]
        pub fn execute_update_admin(&mut self, addr: Addr) -> Result<Response, Error> {
            Ok(Response {
                messages: Vec::new(),
                attributes: Vec::new(),
                events: Vec::new(),
                data: None,
            })
        }

        // query function list

        /// Return the port ID bound by this contract.
        #[ink(message)]
        pub fn query_port(&self) -> Result<PortResponse, Error> {
            Ok(PortResponse {
                port_id: 0.to_string(),
            })
        }

        /// Show all channels we have connected to.
        #[ink(message)]
        pub fn query_list(&self) -> Result<ListChannelsResponse, Error> {
            Ok(ListChannelsResponse {
                channels: Vec::new(),
            })
        }

        ///  Returns the details of the name channel, error if not created.
        #[ink(message)]
        pub fn query_channel(&self, id: String) -> Result<ChannelResponse, Error> {
            let info = self
                .channel_info
                .get(&id)
                .ok_or(Error::ChannelInfoNotFound)?;

            let channel_token_denom = self
                .channel_token_denom
                .get(&id)
                .ok_or(Error::ChannelTokenDenomNotFound)?;

            Ok(ChannelResponse {
                info: ChannelInfo {
                    id: "0".to_string(),
                    counterparty_endpoint: IbcEndpoint {
                        port_id: "0".to_string(),
                        channel_id: "0".to_string(),
                    },
                    connection_id: "0".to_string(),
                },
                balances: Vec::new(),
                total_sent: Vec::new(),
            })
        }

        /// Show the Config.
        #[ink(message)]
        pub fn query_config(&self) -> Result<ConfigResponse, Error> {
            Ok(ConfigResponse {
                default_timeout: 0,
                default_gas_limit: None,
                gov_contract: "".to_string(),
            })
        }

        /// Query if a given cw20 contract is allowed.
        #[ink(message)]
        pub fn query_allowed(&self) -> Result<AllowedResponse, Error> {
            Ok(AllowedResponse {
                is_allowed: false,
                gas_limit: None,
            })
        }

        /// List all allowed cw20 contracts.
        #[ink(message)]
        pub fn list_allowed(
            &self,
            start_after: Option<String>,
            limit: Option<u32>,
        ) -> Result<ListAllowedResponse, Error> {
            Ok(ListAllowedResponse { allow: Vec::new() })
        }

        /// Show current admin
        #[ink(message)]
        pub fn query_admin(&self) -> Result<Option<Addr>, Error> {
            Ok(Some(self.admin.clone()))
        }

        fn enforce_order_and_version(
            &self,
            channel: IbcChannel,
            counterparty_version: Option<&str>,
        ) -> Result<(), trait_ibc::ibc::Error> {
            if channel.version != ICS20_VERSION {
                return Err(trait_ibc::ibc::Error::InvalidIbcVersion {
                    version: channel.version.clone(),
                });
            }
            if let Some(version) = counterparty_version {
                if version != ICS20_VERSION {
                    return Err(trait_ibc::ibc::Error::InvalidIbcVersion {
                        version: version.to_string(),
                    });
                }
            }
            if channel.order != ICS20_ORDERING {
                return Err(trait_ibc::ibc::Error::OnlyOrderedChannel {});
            }
            Ok(())
        }

        fn do_ibc_packet_receive(
            &mut self,
            packet: &IbcPacket,
        ) -> Result<IbcReceiveResponse, Error> {
            let msg: trait_ibc::ibc::Ics20Packet = match from_binary(&packet.data) {
                Ok(v) => v,
                Err(e) => return Err(Error::IBCError(e)),
            };
            let channel = packet.dest.channel_id.clone();

            // If the token originated on the remote chain, it looks like "ucosm".
            // If it originated on our chain, it looks like "port/channel/ucosm".
            let denom = self.parse_voucher_denom(&msg.denom, &packet.src)?;

            // make sure we have enough balance for this
            self.reduce_channel_balance(&channel, denom, msg.amount)?;

            // we need to save the data to update the balances in reply
            let reply_args = ReplyArgs {
                channel,
                denom: denom.to_string(),
                amount: msg.amount,
            };
            self.reply_args = reply_args;

            let to_send = Amount::from_parts(denom.to_string(), msg.amount);
            let gas_limit = 0;
            let send = self.send_amount(to_send, msg.receiver.clone());
            let mut submsg = SubMsg::reply_on_error(send, RECEIVE_ID);
            submsg.gas_limit = Some(gas_limit);

            let res = IbcReceiveResponse::new()
                .set_ack(self.ack_success())
                .add_submessage(submsg)
                .add_attribute("action", "receive")
                .add_attribute("sender", msg.sender)
                .add_attribute("receiver", msg.receiver)
                .add_attribute("denom", denom)
                .add_attribute("amount", msg.amount.to_string())
                .add_attribute("success", "true");

            Ok(res)
        }

        pub fn increase_channel_balance(
            &mut self,
            channel: &str,
            denom: &str,
            amount: u128,
        ) -> Result<(), Error> {
            let mut state = self.channel_state.get((channel, denom)).unwrap_or_default();
            state.outstanding += amount;
            state.total_sent += amount;

            self.channel_state.insert((channel, denom), &state);
            Ok(())
        }

        pub fn reduce_channel_balance(
            &mut self,
            channel: &str,
            denom: &str,
            amount: u128,
        ) -> Result<(), Error> {
            let mut state = self.channel_state.get((channel, denom)).unwrap_or_default();
            state.outstanding = state
                .outstanding
                .checked_sub(amount)
                .expect("InsufficientFunds.");

            self.channel_state.insert((channel, denom), &state);
            Ok(())
        }

        // this is like increase, but it only "un-subtracts" (= adds) outstanding, not total_sent
        // calling `reduce_channel_balance` and then `undo_reduce_channel_balance` should leave state unchanged.
        pub fn undo_reduce_channel_balance(
            &mut self,
            channel: &str,
            denom: &str,
            amount: u128,
        ) -> Result<(), Error> {
            let mut state = self.channel_state.get((channel, denom)).unwrap_or_default();
            state.outstanding += amount;

            self.channel_state.insert((channel, denom), &state);
            Ok(())
        }

        fn parse_voucher_denom<'a>(
            &self,
            voucher_denom: &'a str,
            remote_endpoint: &IbcEndpoint,
        ) -> Result<&'a str, Error> {
            let split_denom: Vec<&str> = voucher_denom.splitn(3, '/').collect();
            if split_denom.len() != 3 {
                return Err(Error::NoForeignTokens {});
            }
            // a few more sanity checks
            if split_denom[0] != remote_endpoint.port_id {
                return Err(Error::FromOtherPort {
                    port: split_denom[0].into(),
                });
            }
            if split_denom[1] != remote_endpoint.channel_id {
                return Err(Error::FromOtherChannel {
                    channel: split_denom[1].into(),
                });
            }

            Ok(split_denom[2])
        }

        // create a serialized success message
        fn ack_success(&self) -> Vec<u8> {
            let res = Ics20Ack::Result(vec![0x1]);
            trait_ibc::ibc::to_binary(&res).unwrap()
        }

        // create a serialized error message
        fn ack_fail(&self, err: String) -> Vec<u8> {
            let res = Ics20Ack::Error(err);
            trait_ibc::ibc::to_binary(&res).unwrap()
        }

        // update the balance stored on this (channel, denom) index
        fn on_packet_success(&self, packet: IbcPacket) -> Result<IbcBasicResponse, Error> {
            let msg: trait_ibc::ibc::Ics20Packet = match from_binary(&packet.data) {
                Ok(v) => v,
                Err(e) => return Err(Error::IBCError(e)),
            };

            // similar event messages like ibctransfer module
            let attributes = vec![
                attr("action", "acknowledge"),
                attr("sender", &msg.sender),
                attr("receiver", &msg.receiver),
                attr("denom", &msg.denom),
                attr("amount", msg.amount.to_string()),
                attr("success", "true"),
            ];

            Ok(IbcBasicResponse::new().add_attributes(attributes))
        }

        // return the tokens to sender
        fn on_packet_failure(
            &mut self,
            packet: IbcPacket,
            err: String,
        ) -> Result<IbcBasicResponse, Error> {
            let msg: trait_ibc::ibc::Ics20Packet = match from_binary(&packet.data) {
                Ok(v) => v,
                Err(e) => return Err(Error::IBCError(e)),
            };

            // undo the balance update on failure (as we pre-emptively added it on send)
            self.reduce_channel_balance(&packet.src.channel_id, &msg.denom, msg.amount)?;

            let to_send = Amount::from_parts(msg.denom.clone(), msg.amount);
            let gas_limit = 0;
            let send = self.send_amount(to_send, msg.sender.clone());
            let mut submsg = SubMsg::reply_on_error(send, ACK_FAILURE_ID);
            submsg.gas_limit = Some(gas_limit);

            // similar event messages like ibctransfer module
            let res = IbcBasicResponse::new()
                .add_submessage(submsg)
                .add_attribute("action", "acknowledge")
                .add_attribute("sender", msg.sender)
                .add_attribute("receiver", msg.receiver)
                .add_attribute("denom", msg.denom)
                .add_attribute("amount", msg.amount.to_string())
                .add_attribute("success", "false")
                .add_attribute("error", err);

            Ok(res)
        }

        fn send_amount(&self, amount: Amount, recipient: String) -> CosmosMsg<Empty> {
            match amount {
                Amount::Native(coin) => BankMsg::Send {
                    to_address: recipient,
                    amount: vec![coin],
                }
                .into(),
                Amount::Cw20(coin) => {
                    // todo warp for psp22
                    let msg = Cw20ExecuteMsg::Transfer {
                        recipient,
                        amount: coin.amount,
                    };
                    WasmMsg::Execute {
                        contract_addr: coin.address,
                        msg: trait_ibc::ibc::to_binary(&msg).unwrap(),
                        funds: vec![],
                    }
                    .into()
                }
            }
        }
    }
}
