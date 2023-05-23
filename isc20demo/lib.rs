#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]
#![feature(default_alloc_error_handler)]

#[openbrush::contract]
pub mod my_psp22_wrapper {
    use ink::prelude::borrow::ToOwned;
    use ink::prelude::{
        string::{String, ToString},
        vec::Vec,
    };
    use openbrush::{contracts::psp22::extensions::wrapper::*, traits::Storage};
    use scale::{Decode, Encode};

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Addr(String);

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct MessageInfo {
        pub sender: Addr,
        pub funds: Vec<Coin>,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Coin {
        pub denom: String,
        pub amount: u128,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Cw20Coin {
        pub address: String,
        pub amount: u128,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Cw20ReceiveMsg {
        pub sender: String,
        pub amount: u128,
        pub msg: Vec<u8>,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AllowMsg {
        pub contract: String,
        pub gas_limit: Option<u64>,
    }

    #[derive(scale::Decode, scale::Encode)]
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

    #[derive(scale::Decode, scale::Encode)]
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

    #[derive(scale::Decode, scale::Encode)]
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

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct IbcEndpoint {
        pub port_id: String,
        pub channel_id: String,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ChannelInfo {
        /// id of this channel
        pub id: String,
        /// the remote channel/port we connect to
        pub counterparty_endpoint: IbcEndpoint,
        /// the connection this exists on (you can use to query client/consensus info)
        pub connection_id: String,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AllowedInfo {
        pub contract: String,
        pub gas_limit: Option<u64>,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PortResponse {
        pub port_id: String,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ListChannelsResponse {
        pub channels: Vec<ChannelInfo>,
    }

    #[derive(scale::Decode, scale::Encode)]
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

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ConfigResponse {
        pub default_timeout: u64,
        pub default_gas_limit: Option<u64>,
        pub gov_contract: String,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AllowedResponse {
        pub is_allowed: bool,
        pub gas_limit: Option<u64>,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ListAllowedResponse {
        pub allow: Vec<AllowedInfo>,
    }

    // pub enum ExecuteMsg {
    //     /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    //     Receive(Cw20ReceiveMsg),
    //     /// This allows us to transfer *exactly one* native token
    //     Transfer(TransferMsg),
    //     /// This must be called by gov_contract, will allow a new cw20 token to be sent
    //     Allow(AllowMsg),
    //     /// Change the admin (must be called by current admin)
    //     UpdateAdmin { admin: String },
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

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        wrapper: wrapper::Data,
    }

    impl PSP22 for Contract {}

    impl PSP22Wrapper for Contract {}

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

        // set function list

        //receive token
        #[ink(message)]
        pub fn execute_receive(&self, info: MessageInfo, wrapper: Cw20ReceiveMsg) {}

        //transfer token
        #[ink(message)]
        pub fn execute_transfer(&self, msg: TransferMsg, amount: Amount, sender: Addr) {}

        #[ink(message)]
        pub fn execute_allow(&self, info: MessageInfo, allow: AllowMsg) {}

        //update admin address
        #[ink(message)]
        pub fn execute_update_admin(&self, addr: Addr) {}

        // ibc base function
        // #[ink(message)]
        // pub fn reply(reply: Reply) -> Result<Response, ContractError> {}

        // #[ink(message)]
        // pub fn ibc_channel_open(msg: IbcChannelOpenMsg) -> Result<(), ContractError> {}

        // #[ink(message)]
        // pub fn ibc_channel_connect(
        //     msg: IbcChannelConnectMsg,
        // ) -> Result<IbcBasicResponse, ContractError> {
        // }

        // #[ink(message)]
        // pub fn ibc_channel_close(
        //     _channel: IbcChannelCloseMsg,
        // ) -> Result<IbcBasicResponse, ContractError> {
        // }

        // #[ink(message)]
        // pub fn ibc_packet_receive(msg: IbcPacketReceiveMsg) -> Result<IbcReceiveResponse, Never> {}

        // #[ink(message)]
        // pub fn ibc_packet_ack(msg: IbcPacketAckMsg) -> Result<IbcBasicResponse, ContractError> {}

        // #[ink(message)]
        // pub fn ibc_packet_timeout(
        //     msg: IbcPacketTimeoutMsg,
        // ) -> Result<IbcBasicResponse, ContractError> {
        // }

        // query function list

        #[ink(message)]
        pub fn query_port(&self) -> PortResponse {
            PortResponse {
                port_id: 0.to_string(),
            }
        }

        #[ink(message)]
        pub fn query_list(&self) -> ListChannelsResponse {
            ListChannelsResponse {
                channels: Vec::new(),
            }
        }

        #[ink(message)]
        pub fn query_channel(&self, id: String) -> ChannelResponse {
            ChannelResponse {
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
            }
        }

        #[ink(message)]
        pub fn query_config(&self) -> ConfigResponse {
            ConfigResponse {
                default_timeout: 0,
                default_gas_limit: None,
                gov_contract: "".to_string(),
            }
        }

        #[ink(message)]
        pub fn query_allowed(&self) -> AllowedResponse {
            AllowedResponse {
                is_allowed: false,
                gas_limit: None,
            }
        }

        #[ink(message)]
        pub fn list_allowed(
            &self,
            start_after: Option<String>,
            limit: Option<u32>,
        ) -> ListAllowedResponse {
            ListAllowedResponse { allow: Vec::new() }
        }

        #[ink(message)]
        pub fn query_admin(&self) -> Option<Addr> {
            Some(Addr("".to_string()))
        }
    }
}
