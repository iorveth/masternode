/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use runtime_primitives::traits::Hash;
use parity_codec::{Encode, Decode};
use support::{decl_module, ensure, decl_storage, decl_event, StorageMap, EnumerableStorageMap, dispatch::Result, StorageValue};
use rstd::prelude::Vec;
use system::ensure_signed;
use support::traits::{ReservableCurrency};

pub trait Trait: balances::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		MasternodeDeposit get(masternode_deposit) config(): T::Balance;
		MasternodeList: linked_map T::AccountId => Vec<T::AccountId>;
		RandomList get(random_list): map T::AccountId => Vec<T::Hash>; 

		Nonce: u64;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		pub fn create_masternode(origin, masternode_id: T::AccountId) -> Result {
			let who = ensure_signed(origin)?;
			for (_, masternodes) in <MasternodeList<T>>::enumerate() {
				for masternode in &masternodes {
					ensure!(who != *masternode, "Masternode cannot own another masternode");
				}
			}
			ensure!(who != masternode_id, "Cannot create masternode from yourself");
			let mut masternodes = <MasternodeList<T>>::get(&who);
			ensure!(masternodes.iter().all(|masternode| *masternode != masternode_id), "This account_id already associated with masternode");
			<balances::Module<T>>::reserve(&who, Self::masternode_deposit())?;
			masternodes.push(masternode_id.clone());
			<MasternodeList<T>>::insert(&who, masternodes);
			Self::deposit_event(RawEvent::MasternodeCreated(who, masternode_id));
			Ok(())
		}

		pub fn break_masternode(origin, masternode_id: T::AccountId) -> Result {
			let who = ensure_signed(origin)?;
			ensure!(<MasternodeList<T>>::exists(&who), "Masternode for this AccountID not found");
			let mut masternodes = <MasternodeList<T>>::get(&who);
			ensure!(masternodes.iter().any(|masternode| *masternode == masternode_id), "This account doesn`t own masternode with this masternode_id");
			for (i, masternode) in masternodes.clone().iter_mut().enumerate() {
				if *masternode == masternode_id {
					masternodes.remove(i);
				}
			}
			if masternodes.is_empty() {
				<MasternodeList<T>>::remove(&who);
			} else {
				<MasternodeList<T>>::insert(&who, masternodes);
			}
			<balances::Module<T>>::unreserve(&who, Self::masternode_deposit());
			Self::deposit_event(RawEvent::MasternodeBroke(who, masternode_id));
			Ok(())
		}

		pub fn generate_random_values(origin) -> Result {
			let who = ensure_signed(origin)?;
			let random_values = Self::gen_random_values();
			<RandomList<T>>::insert(&who, &random_values);
			Self::deposit_event(RawEvent::ValuesGenerated(who, random_values));
			Ok(())
		}

		pub fn change_deposit(new_deposit: T::Balance) {
			<MasternodeDeposit<T>>::put(new_deposit);
			Self::deposit_event(RawEvent::DepositChanged(new_deposit));
		}
	}
}

impl<T: Trait> Module<T> {

	fn gen_random_values() -> Vec<T::Hash> {
		let mut vec = Vec::new();
		for (_, masternodes) in <MasternodeList<T>>::enumerate() {
			for masternode_id in &masternodes {
				let nonce = <Nonce<T>>::get();
            	let random_hash = (<system::Module<T>>::random_seed(), masternode_id, nonce)
                	.using_encoded(<T as system::Trait>::Hashing::hash);
    			vec.push(random_hash);
				<Nonce<T>>::mutate(|n| *n += 1);
			}
		}
		vec
	}
}

decl_event!(
	pub enum Event<T> 
	where AccountId = <T as system::Trait>::AccountId,
		Hash = <T as system::Trait>::Hash,
		Balance = <T as balances::Trait>::Balance
	{
		MasternodeCreated(AccountId, AccountId),
		MasternodeBroke(AccountId, AccountId),
		ValuesGenerated(AccountId, Vec<Hash>),
		DepositChanged(Balance),
	}
);