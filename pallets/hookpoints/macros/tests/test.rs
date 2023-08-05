use macros::hooks;

struct AccountId;
struct DispatchError;

#[hooks]
trait GenesisDAO {
	fn voting_power(&self, voter: AccountId, balance: u128) -> u128;
	// ... more fn's
}
