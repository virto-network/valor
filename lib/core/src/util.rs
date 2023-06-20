#[macro_export]
macro_rules! map {
	() => {{
		#[cfg(no_std)]
		extern crate alloc;
		#[cfg(no_std)]
		use alloc::collections::BTreeMap;

		#[cfg(not(no_std))]
		use std::collections::BTreeMap;

		BTreeMap::new()
	}};
	( $($k:expr => $v:expr),* ) => {
		{
			#[cfg(no_std)]
			extern crate alloc;
			#[cfg(no_std)]
			use alloc::collections::BTreeMap;

			#[cfg(not(no_std))]
			use std::collections::BTreeMap;

			let mut hm = BTreeMap::<String, String>::new();
			$(
				hm.insert($k.to_string(), $v.to_string());
			)*
			hm
		}
	};
}
