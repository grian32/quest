#[cfg(test)]
macro_rules! assert_call_idempotent {
	(@INTO $ty:ident) => { $ty };
	(@INTO $_ty:ident $into:ty) => { $into };

	($ty:ident::$fn:ident($this:expr $(, $args:expr)*) $( $(-> $into:ty)?, $expected:expr)?) => {
		crate::initialize();

		let old = Object::from($this);
		let new = $ty::$fn(&old, args!($($args),*)).unwrap();
		assert!(!old.is_identical(&new));
		$(
			old.downcast_and_then(|x: &assert_call_idempotent!(@INTO $ty $($into)?)|
				assert_eq!(*x, $expected)
			).unwrap();
		)?
	};
}

#[cfg(test)]
macro_rules! assert_call_non_idempotent {
	($ty:ident::$fn:ident($this:expr $(, $args:expr)*) $( $(-> $into:ty)?, $expected:expr)?) => {{
		crate::initialize();

		let old = Object::from($this);
		let new = $ty::$fn(&old, args!($($args),*)).unwrap();
		assert!(old.is_identical(&new));
		$(
			old.downcast_and_then(|x: &assert_call_idempotent!(@INTO $ty $($into)?)|
				assert_eq!(*x, $expected)
			).unwrap();
		)?
	}};
}

#[cfg(test)]
macro_rules! call_unwrap {
	($ty:ident::$fn:ident($this:expr $(, $args:expr)*) $(-> $ret:ty)?; $block:expr) => {{
		crate::initialize();
		#[allow(unused_imports)]
		use crate::types::*;

		$ty::$fn(&$this.into(), args!($($args),*)).unwrap()
			.downcast_and_then($block)
			.unwrap()
	}};
}

#[cfg(test)]
macro_rules! call_unwrap_err {
	($ty:ident::$fn:ident($this:expr $(, $args:expr)*)) => {{
		crate::initialize();
		#[allow(unused_imports)]
		use crate::types::*;

		$ty::$fn(&$this.into(), args!($($args),*)).unwrap_err()
	}};
}

#[cfg(test)]
macro_rules! assert_call {
	($ty:ident::$fn:ident($this:expr $(, $args:expr)*) $(-> $ret:ty)?; $block:expr) => {
		assert!(call_unwrap!($ty::$fn($this $(, $args)*) $(-> $ret)?; $block));
	};
}

#[cfg(test)]
macro_rules! assert_call_err {
	($ty:ident::$fn:ident($this:expr $(, $args:expr)*), $($tt:tt)*) => {{
		assert_matches!(call_unwrap_err!($ty::$fn($this $(, $args)*)), $($tt)*)
	}};
}

#[cfg(test)]
macro_rules! assert_call_eq {
	($ty:ident::$fn:ident($this:expr $(, $args:expr)*) $(-> $ret:ty)?, $rhs:expr) => {{
		call_unwrap!($ty::$fn($this $(, $args)*) $(-> $ret)?; |lhs $(: &$ret)?| {
			assert_eq!(*lhs, $rhs)
		})
	}};
}

#[cfg(test)]
macro_rules! assert_call_missing_parameter {
	($ty:ident::$fn:ident($this:expr $(, $args:expr)*), $idx:expr $(, len=$len:pat)?) => {{
		crate::initialize();

		assert_matches!(
			$ty::$fn(&$this.into(), args!($($args),*)),
				Err($crate::Error::KeyError($crate::error::KeyError::OutOfBounds {
					idx: $idx, $(len: $len,)? .. }))
		);
	}};
}

#[cfg(test)]
macro_rules! assert_matches {
	($lhs:expr, $($rest:tt)*) => {{
		let lhs = $lhs;
		assert!(
			matches!(lhs, $($rest)*),
			concat!("values don't match\nlhs: {:?}\npat: {}"),
			lhs,
			stringify!($($rest)*)
		);
	}};
}

#[cfg(test)]
macro_rules! args {
	() => { $crate::Args::default() };
	($($args:expr),+) => {
		$crate::Args::new(vec![$(&$args.into()),*])
	};
}

#[macro_export]
/// Create a new object type.
///
/// This is soft-deprecated.
macro_rules! impl_object_type {
	(@CONVERTIBLE $_obj:ty;) => {};
	(@CONVERTIBLE $obj:ty; (convert $convert_func:expr) $($_rest:tt)*) => {
		impl $crate::types::Convertible for $obj {
			const CONVERT_FUNC: &'static str = $convert_func;
		}
	};
	(@CONVERTIBLE $obj:ty; $_b:tt $($rest:tt)*) => {
		impl_object_type!(@CONVERTIBLE $obj; $($rest)*);
	};
	(@CONVERTIBLE $($tt:tt)*) => {
		compile_error!(concat!("bad CONVERTIBLE: ", stringify!($($tt)*)))
	};

	(@PARENT_DEFAULT) => { compile_error!("A parent is needed to create an object"); };
	(@PARENT_DEFAULT (parents $parent:path) $($_rest:tt)*) => { <$parent as Default>::default() };
	(@PARENT_DEFAULT $_b:tt $($rest:tt)*) => { impl_object_type!(@PARENT_DEFAULT $($rest)*); };

	(@SET_PARENT $class:ident) => { () };
	(@SET_PARENT $class:ident (init_parent) $($_rest:tt)*) => { () };
	(@SET_PARENT $class:ident (init_parent $($init_parent:path)+) $($_rest:tt)*) => {
		vec![$(<$init_parent as $crate::types::ObjectType>::mapping()),+]
	};
	(@SET_PARENT $class:ident (parents $parent:path) $($_rest:tt)*) => {
		impl_object_type!(@SET_PARENT $class (init_parent $parent));
	};

	(@SET_PARENT $class:ident $_b:tt $($rest:tt)*) => {
		impl_object_type!(@SET_PARENT $class $($rest)*)
	};

	(@SET_ATTRS $class:ident $obj:ty;) => {};
	(@SET_ATTRS $class:ident $obj:ty; $attr:expr => const $val:expr $(, $($args:tt)*)?) => {{
		$class.set_attr_lit($attr, Object::from($val))?;
		impl_object_type!(@SET_ATTRS $class $obj; $($($args)*)?);
	}};

	(@SET_ATTRS $class:ident $obj:ty; $attr:expr => function $val:expr $(, $($args:tt)*)?) => {{
		$class.set_value_lit($attr, $crate::types::RustFn::new(
			concat!(stringify!($obj), "::", $attr), $val)
		)?;
		impl_object_type!(@SET_ATTRS $class $obj; $($($args)*)?);
	}};

	(@SET_ATTRS $class:ident $obj:ty; $attr:expr => method_old $val:expr $(, $($args:tt)*)?) => {{
		$class.set_value_lit($attr, $crate::types::RustFn::new(
			concat!(stringify!($obj), "::", $attr),
			|this, args| {
				this.try_downcast_and_then(|this| $val(this, args)
					.map(Object::from)
					.map_err($crate::Error::from))
			}
		))?;
		impl_object_type!(@SET_ATTRS $class $obj; $($($args)*)?);
	}};


	(@SET_ATTRS $class:ident $obj:ty; $attr:expr => method_old_mut $val:expr $(, $($args:tt)*)?) => {{
		$class.set_value_lit($attr, $crate::types::RustFn::new(
			concat!(stringify!($obj), "::", $attr),
			|this, args| {
				this.try_downcast_mut_and_then(|data| $val(data, args).map(Object::from).map_err($crate::Error::from))
			}
		))?;
		impl_object_type!(@SET_ATTRS $class $obj; $($($args)*)?);
	}};

	(@SET_ATTRS $_class:ident $_obj:ty; $($tt:tt)*) => {
		compile_error!(concat!("Bad attrs given:", stringify!($($tt)*)));
	};


	(for $obj:ty $({$new_object:item})? [ $($args:tt)* ]: $($body:tt)*) => {
		impl_object_type!(@CONVERTIBLE $obj; $($args)* );

		impl $crate::types::ObjectType for $obj {
			fn initialize() -> $crate::Result<()> {
				// `Once` wouldn't allow for returning an error.
				use ::std::sync::atomic::{AtomicBool, Ordering};

				const UNINIT: bool = false;
				const INIT: bool = true;
				static INITIALIZE: AtomicBool = AtomicBool::new(UNINIT);

				if INITIALIZE.compare_and_swap(UNINIT, INIT, Ordering::SeqCst) == INIT {
					return Ok(());
				}

				let class = Self::mapping();
				class.set_attr_lit("name", stringify!($obj).into())?;

				impl_object_type!(@SET_ATTRS class $obj; $($body)*);
				Ok(())
			}

			fn mapping() -> $crate::Object {
				lazy_static::lazy_static! {
					static ref CLASS: $crate::Object = $crate::Object::new_with_parent(
						impl_object_type!(@PARENT_DEFAULT $($args)*),
						impl_object_type!(@SET_PARENT class $($args)*)
					);
				}

				CLASS.clone()
			}			

			$($new_object)?
		}
	};
}
