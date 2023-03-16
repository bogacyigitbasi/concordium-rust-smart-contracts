use crate::types::{Account, ChainEvent, Contract, ContractModule};
use concordium_base::contracts_common::{
    AccountAddress, Amount, ContractAddress, ExchangeRate, ModuleReference, OwnedContractName,
    OwnedEntrypointName, SlotTime,
};
use concordium_smart_contract_engine::{
    v0,
    v1::{trie::MutableState, InvokeResponse},
    InterpreterEnergy,
};
use std::collections::BTreeMap;

/// The result of invoking an entrypoint.
pub(crate) struct InvokeEntrypointResult {
    /// The result from the invoke.
    pub(crate) invoke_response:  InvokeResponse,
    /// Logs created during the invocation.
    /// Has entries if and only if `invoke_response` is `Success`.
    pub(crate) logs:             v0::Logs,
    /// The remaining energy after the invocation.
    pub(crate) remaining_energy: InterpreterEnergy,
}

/// A type that supports invoking a contract entrypoint.
pub(crate) struct EntrypointInvocationHandler {
    pub(super) changeset:          ChangeSet,
    pub(super) accounts:           BTreeMap<AccountAddress, Account>,
    pub(super) modules:            BTreeMap<ModuleReference, ContractModule>,
    pub(super) contracts:          BTreeMap<ContractAddress, Contract>,
    pub(super) block_time:          SlotTime,
    pub(super) euro_per_energy:    ExchangeRate,
    pub(super) micro_ccd_per_euro: ExchangeRate,
}

/// The set of [`Changes`] represented as a stack.
#[derive(Debug, Clone)]
pub(crate) struct ChangeSet {
    /// The stack of changes.
    pub(super) stack: Vec<Changes>,
}

/// Data held for accounts and contracts during the execution of a contract
/// entrypoint.
#[derive(Clone, Debug)]
pub(super) struct Changes {
    /// The contracts which have changes.
    pub(super) contracts: BTreeMap<ContractAddress, ContractChanges>,
    /// The accounts which have changes.
    pub(super) accounts:  BTreeMap<AccountAddress, AccountChanges>,
}

/// Data held for an account during the execution of a contract entrypoint.
#[derive(Clone, Debug)]
pub(super) struct AccountChanges {
    /// Should never be modified.
    pub(super) original_balance: Amount,
    pub(super) balance_delta:    AmountDelta,
}

/// Data held for a contract during the execution of a contract entrypoint.
#[derive(Clone, Debug)]
pub(super) struct ContractChanges {
    /// An index that is used to check whether a caller contract has been
    /// modified after invoking another contract (due to reentrancy).
    pub(super) modification_index:    u32,
    /// Represents how much the contract's self balance has changed.
    pub(super) self_balance_delta:    AmountDelta,
    /// The original contract balance, i.e. the one that is persisted. Should
    /// never be modified.
    pub(super) self_balance_original: Amount,
    /// The potentially modified contract state.
    pub(super) state:                 Option<MutableState>,
    /// The potentially changed module.
    pub(super) module:                Option<ModuleReference>,
}

/// Data needed to recursively process a contract entrypoint to completion.
///
/// In particular, this keeps the data necessary for resuming a contract
/// entrypoint after an interrupt.
///
/// One `InvocationData` is created for each time
/// [`EntrypointInvocationHandler::invoke_entrypoint`] is called.
pub(super) struct InvocationData<'a> {
    /// The invoker.
    pub(super) invoker:            AccountAddress,
    /// The contract being called.
    pub(super) address:            ContractAddress,
    /// The name of the contract.
    pub(super) contract_name:      OwnedContractName,
    /// The amount sent from the sender to the contract.
    pub(super) amount:             Amount,
    /// The entrypoint to execute.
    pub(super) entrypoint:         OwnedEntrypointName,
    /// A reference to the [`EntrypointInvocationHandler`], which is used to for
    /// handling interrupts and for querying chain data.
    pub(super) invocation_handler: &'a mut EntrypointInvocationHandler,
    /// The current state.
    pub(super) state:              MutableState,
    /// Chain events that have occurred during the execution.
    pub(super) chain_events:       Vec<ChainEvent>,
}

/// A positive or negative delta in for an [`Amount`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AmountDelta {
    /// A posittive delta.
    Positive(Amount),
    /// A negative delta.
    Negative(Amount),
}

/// An underflow occurred.
#[derive(Debug)]
pub(super) struct UnderflowError;
