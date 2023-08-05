#![feature(let_chains)]

use proc_macro::TokenStream as TS1;
use proc_macro2::TokenStream as TS;
// use proc_macro2::{Ident, Span, TokenStream as TS, TokenTree as TT};
use quote::quote;
use syn::{
	spanned::Spanned, Error, FnArg, Ident, ItemTrait, Result, ReturnType, TraitItem, TraitItemFn,
};

/// this macro expands to one callback for each trait method but returns Result (or maybe Option):
/// fn voting_power(voter: AccountId, balance: u128) -> Result<u128, ()> {
/// 	// on any error return Err
/// 	// retrieve the contract registered
/// 	// encode the parameters
/// 	// execute the callback to on_vote
/// 	// decode the return value
/// }
#[proc_macro_attribute]
pub fn hooks(_args: TS1, input: TS1) -> TS1 {
	let input = syn::parse_macro_input!(input as ItemTrait);
	expand_hooks(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn expand_hooks(t: ItemTrait) -> Result<TS> {
	let trait_name = &t.ident;
	let e: Vec<_> =
		t.items.into_iter().map(|f| expand_hook(f, trait_name)).collect::<Result<_>>()?;
	Ok(quote! { #( #e )* })
}

fn expand_hook(item: TraitItem, trait_name: &Ident) -> Result<TS> {
	match item {
		TraitItem::Fn(f) => expand_fn(f, trait_name),
		_ => unimplemented!(),
	}
}

fn expand_fn(f: TraitItemFn, trait_name: &Ident) -> Result<TS> {
	let expected_self = "expected `&self`";
	let args_span = f.sig.inputs.span();
	let mut params = f.sig.inputs.into_iter();
	// check that first param is self
	if let Some(param_self) = params.next() {
		if !matches!(param_self, FnArg::Receiver(_)) {
			return Err(Error::new(param_self.span(), expected_self))
		}
	} else {
		return Err(Error::new(args_span, expected_self))
	}
	// and gather other params
	let params: Vec<_> = params.collect();

	let name = f.sig.ident;
	let return_ty = match f.sig.output {
		ReturnType::Type(_, r) => r,
		_ => return Err(Error::new(f.sig.output.span(), "expected return type")),
	};
	let selector: [u8; 4] = {
		use blake2::Digest;
		let mut hasher = blake2::Blake2s256::new();
		hasher.update(format!("{trait_name}::{name}"));
		hasher.finalize()[..4].try_into().unwrap()
	};

	Ok(
		quote! { fn #name (contract: AccountId, #( #params ),* ) -> Result<#return_ty, DispatchError> {
			// enable debug in debug mode and disable in release mode
			//#[cfg(debug_assertions)]
			let debug = true;
			//#[cfg(not(debug_assertions))]
			let debug = false;
			// the selector for "GenesisDAO::calculate_voting_power"

			Ok(Default::default())
		}
		},
	)
}

//code to generate:
/*
fn #name(contract: AccountId, #other_args_from_trait) -> Result<#ret> {
	// enable debug in debug mode and disable in release mode
	#[cfg(debug_assertions)]
	let debug = true;
	#[cfg(not(debug_assertions))]
	let debug = false;
	// the selector for "GenesisDAO::calculate_voting_power"
	let mut data = 0xa68e4cba_u32.to_be_bytes().to_vec();
	data.append(&mut voter.encode()); // argument AccountId
	data.append(&mut token_balance.encode()); // argument u128
	let contract_exec_result = Contracts::<T>::bare_call(
		voter.clone(),
		contract,
		0_u32.into(),              // value to transfer
		Weight::from_all(10_000_000_000), // gas limit
		Some(0_u32.into()),        // storage deposit limit
		data,
		debug,
		Determinism::Enforced,
	);
	// check debug message
	#[cfg(debug_assertions)]
	assert_eq!(String::from_utf8_lossy(&contract_exec_result.debug_message), "");
	let result = contract_exec_result.result?;
	<Result<AssetBalanceOf<T>, _>>::decode(&mut &result.data[..])
		.map_err(|_| DispatchError::Other("decoding error"))?
}
*/
