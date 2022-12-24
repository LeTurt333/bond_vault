# bond_vault - WIP

This contract takes in any amount of Tokens (currently Native only)

These tokens can then be bought at any arbitrary price (this can come from an Oracle or be set as Config / Const)

Purchased tokens are vested linearly based on a buyers vesting choice

The length of the vesting period a buyer chooses gives them a "boost" to the amount they purchase when compared to the arbitrary price, equal to the # of weeks (100,000 block periods) in the vesting lockup

EX:

- Contract holds JUNO (sale asset)

- The user chooses a 10 week vesting period

- The user buys 500 JUNO with 500 USXDC, at a price of 1 : 1

- The user will get 550 JUNO, vested linearly per block over 10 weeks

- **To Do**: The user can "sell" the vesting position at any time on the Juno Vaults marketplace