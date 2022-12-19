# cw-drip

This is a minimal implementation of a Community DAO Incentive Contract. The contract will allow to distribute any native or cw-20 tokens to the community via a linear vesting. Any user interested in receiving tokens have to send a tx for the participation in the distribution. The contract will distribute shares to participants that fullfill a minimum requirement on native tokens staked every epoch. Every participant can decide to burn its shares to withdraw distributed tokens.

## How it works

The owner of the contract will be the sender of the `InstantiateMsg` tx. During the instantiation must be provided the parameters common to every distribtuion. They are the minimum required staked tokens and the duration of a single epoch expressed in seconds. The staked tokens of every participant are computed summing up all the delegations made by the address.

```rust
pub struct InstantiateMsg {
    pub min_staking_amount: Uint128,
    pub epoch_duration: u64,
}
```

Once instantiated the contract, community members can decide to participate in the drip by sending a `ExecuteMsg::Participate {}` tx. The partecipation to the drip distribution means a participation to every drip pool. It is not possibile to decide to participate just to specific distributions. Participants can decide to exit from the distribution in any time by sending a `ExecuteMsg::RemoveParticipation {}` tx.

The creation of a drip pool can be made only by the owner of the contract and is subordinate to the presence of the distributed tokens inside the contract. This means that, in order to create a $1\_000\_000\_000$\$JUNO distribution, the contract must be the onwer of $1_000_000_000$\$JUNO. The creation of a drip pool can be made by sending the following tx:

```rust
ExecuteMsg:: CreateDripPool {
    token_info: UncheckedDripToken,
    tokens_per_epoch: Uint128,
    epochs_number: u64,
}
```

where `UncheckedDripToken`is:

```rust
pub enum UncheckedDripToken {
    Native { denom: String, initial_amount: Uint128 },
    Cw20 { address: String, initial_amount: Uint128 }
}
```

This message requires to specify the token and the total amount of the distribution along with the tokens per epoch and number of epoch. Since the number of epochs times the tokens per epoch must be equal to the total initial amount, the message impose the sender to double check the pool specifications.

In order to distribute the shares a `ExecuteMsg::DistributeShares` tx must be sent to the contract. There is no constraint on who can trigger the distribution. To better understand how tokens will be distributed let's make an example with a drip pool of $200$\$TOKEN distributed in $2$ epochs. This means $100$\$TOKEN distributed every epoch. Let's consider the first two distribution epoch with a minimum required staked tokens of $10$.

| EPOCH | Bob staking | Alice staking | Bob shares | Alice shares | Total shares | Distributed tokens |
| ----- | ----------- | ------------- | ---------- | ------------ | ------------ | ------------------ |
| 1     | 5           | 12            | 0          | 12           | 12           | 100                |
| 2     | 15          | 20            | 15         | 12 + 20 = 32 | 32 + 15 = 47 | 100 + 100 = 200    |

At the end of this distribution, we will have:

$$ \text{Bob \$TOKEN} = floor\Big(\frac{32}{47} \times 200\Big) = 136 $$

$$ \text{Alice \$TOKEN} = floor\Big(\frac{15}{47} \times 200\Big) = 63 $$

The remaining $200 - 136 - 63 = 1$\$TOKEN will be withdrawable from the owner of the contract.

## What is missing

The following messages handler are still to be implemented:

* `UpdateDripPool {}`: update a pool configuration;

* `RemoveDripPool {}`: remove an active pool;

* `SendShares {}`: transfer the accrued shares to another address.

🚧 🚧 🚧 WORK IN PROGRESS 🚧 🚧 🚧
## State

The state of the contract is composed of 5 components as depicted in the schema below:

![caption](/assets/state.png "Contract state")

* `Config`: stores the smart contract configuration info.

* `PARTICIPANTS`: stores the vector of the participants addresses.

* `Participants Shares`: stores users shares for each (active & unactive) drip pool indexed on the user address.

## Design

1. `PARTICIPANTS` + `PARTICIPANTS_SHARES`: these two stores are used to manage different but related behaviours of the smart contract. `PARTICIPANTS` is used to manage only the list of actual addresses participanting to the distributions. If someone wants to participate it will be added to this list and if someone wants to remove participation it will be removed from this list. During shares distribution only address in thsi store will be accounted for. It is still important to consider address that has received shares but are no more participants. For this reason a second auxiliary object as been used: `PARTICIPANTS_SHARES`. This store manage the shares of every address that has ever participate to the distribution. With this two-objects configuration we can easily access the shares associated to an account that would like to withdraw and at the same time we can iterate just through active participants during shares distribution.

2. `DRIP_POOLS` & `DripPoolsShares` are connected with a string which is the address of the Cw20 contract or the native denome instead of a number. In this way we don't need a counter to instantiate a new pool and makes the indexing of a specific pool faster.

## Tests

* [x] `drip_pool_basic_checks`: only owner can create a drip pool and no drip pool with 0 epochs allowed;

* [x] `zero_initial_amount`: creating a drip pool with 0 tokens is not allowed (Native + Cw20);

* [x] `no_funded_contract`: creating a drip pool when the contract has less tokens than the required
for the distribution (Native + Cw20);

* [x] `wrong_tokens_amount`: creating a drip pool with $\frac{initial\_amount}{epochs} \neq tokens\_per\_epoch$ is not allowed (Native + Cw20);

* [x] `funded_contract`: create a drip pool (Native + Cw20);

* [x] `zero_active_pool`: error if no active pool;

* [x] `no_distribution_time`: cannot distribute before distribution time;

* [x] `no_min_staking`: if no minimum staking for stakers no shares issued;

* [x] `distribute_single`: shares are distributed correctly for a single user and a single drip pool for the first epoch;

* [x] `distribute_multiple`: shares are distributed correctly to 3 users and after the last epoch the pool is no more active;

* [x] `multiple_drip_pools`: shares are distributed correctly with 2 pools and a single user;

* [ ] Create participant A, distribute shares, remove participation of A and than create again the
participation of A.

# Feedback

Please, feel free to send any feedback to stepyt@mib.tech or contributed with PR(s).