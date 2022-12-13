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
