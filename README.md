# cw-drip

This is a minimal implementation of a Community DAO Incentive Contract.

## State

The state of the contract is composed of 5 components as depicted in the schema below:

![caption](/assets/state.png "Contract state")

* `Config`: stores the smart contract configuration info.

* `PARTICIPANTS`: stores the vector of the participants addresses.

* `Participants Shares`: stores users shares for each (active & unactive) drip pool indexed on the user address.

## Design

1. `PARTICIPANTS` + `PARTICIPANTS_SHARES`: these two stores are used to manage different but related behaviours of the smart contract. `PARTICIPANTS` is used to manage only the list of actual addresses participanting to the distributions. If someone wants to participate it will be added to this list and if someone wants to remove participation it will be removed from this list. During shares distribution only address in thsi store will be accounted for. It is still important to consider address that has received shares but are no more participants. For this reason a second auxiliary object as been used: `PARTICIPANTS_SHARES`. This store manage the shares of every address that has ever participate to the distribution. With this two-objects configuration we can easily access the shares associated to an account that would like to withdraw and at the same time we can iterate just through active participants during shares distribution.

2. `DRIP_POOLS` & `DripPoolsShares` are connected with a string which is the address of the Cw20 contract or the native denome instead of a number. In this way we don't need a counter to instantiate a new pool and makes the indexing of a specific pool faster.

# Tests

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