use crate::{Object, Args};
use crate::types::Boolean;
use tracing::instrument;

/// The base type that all other Quest types inherit from.
///
/// This class simply contains the bare minimum set of functions required to make Quest work.
///
/// Since everything inherits from it, you probably shouldn't edit it directly. If you want to make 
/// attributes that are globally visible, you should probably do that within [`Kernel`](
/// crate::types::Kernel) instead.
///
/// # Additional Attributes
/// 
/// In addition to those detailed in this class, there are two additional keys that are always
/// defined: `__id__` and `__parents__`.
///
/// ## `__id__`
///
/// `__id__` is an unique identifier for each object and cannot be changed. (You _can_ assign to
/// `__id__`, but if you ever try to read it, you'll end up with the object's original id.) This is
/// used in multiple places, including the default `inspect` and `==` implementations.
///
/// ## `__parents__`
///
/// The meat of Quest, `__parents__` is how dynamic attribute lookup happens. When fetching an
/// attribute, the following places are looked, in order: (Note that this only applies to 
/// fetching attributes; setting and deleting attributes only work on the base object.)
///
/// 1. Builtin attributes (i.e. `__id__`, `__parents__`). Additionally, there are two "special"
/// attributes that aren't considered to be a part of any particular object: `__this__` and
/// `__stack__`
///    - `__stack__` returns a list of all the stackframes so far, with `0` being the current one.
///    - `__this__` is the same as `__stack__.get(0)`. Currently, it's only defined for scopes, but
///      this may be changed in the future.
/// 2. Any attributes directly defined for the object. (e.g. `foo.bar = 3;`).
/// 3. If `__attr_missing__` is defined, it is called; if a non-[`Null`] response is given, then
///    that value is returned. (In the future there may be a way to mark `null` as a valid response,
///    possibly with something like the `undefined` of javascript?)
/// 4. Each parent, in order, is asked if they (Or any of their parents) have the attribute.
///    the first parental chain that has one is returned.
/// 5. If nothing succeeds, (either an error or [`Null`] is returned. I haven't figured out which
///    is the best yet.)
///
/// [`Null`]: crate::types::Null;
///
/// ## `:#`
/// 
/// Stack frame literal references have bene added to Quest: `:#` is identical to
/// `__stack__.get(#)`, but allows for shorter mannerisms such as `return(:1)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pristine;

#[allow(non_snake_case)]
impl Pristine {
	/// Gets an internal representation of this type as a [`Text`]
	///
	/// # Difference from `@text`
	///
	/// This differs from `@text` by their goals: `@text` is used to convert to a [`Text`] object,
	/// whereas `inspect` is used to get a debugging representation.
	///
	/// # Quest Examples
	/// ```quest
	/// assert(1.inspect() == "1");
	/// assert(2.inspect() == '"2"');
	/// assert(["2", 3].inspect() == '["2", 3]');
	/// ```
	///
	/// [`Text`]: crate::types::Text
	#[instrument(name="Pristine::hash", level="trace", skip(this), fields(self=?this))]
	pub fn qs_inspect(this: &Object, _: Args) -> crate::Result<Object> {
		// Ok(format!("<{}:{}>", this.typename(), this.id()).into())
		Ok(format!("{:?}", this).into())
	}

	/// Calls a given attribtue for this object
	///
	/// This is generally the same as getting an attribute and calling it (i.e. `foo.bar(3,4)` is
	/// generally the same as `foo.__call_attr__('bar', 3, 4)`, unless something has been manually
	/// overwritten).
	///
	/// # Arguments
	///
	/// 1. (required) The attribute to call.
	/// 2+ Any additional argumnts are forwarded along.
	///
	/// # Quest Examples
	/// ```quest
	/// assert( 12.__call_attr__('+', 4) == 16 )
	/// assert( "foobar".__call_attr__('get', 0, 2) == "foobar")
	/// ```
	#[instrument(name="Pristine::__call_attr__", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs___call_attr__<'a>(this: &'a Object, args: Args<'_, 'a>) -> crate::Result<Object> {
		let attr = args.try_arg(0)?;
		let rest = args.try_args(1..).unwrap_or_default();

		this.call_attr(attr, rest)
	}

	/// Retrieves an attribute from the object or one of its parents.
	///
	/// The `__get_attr__` method and the `::` infix operator are identical, but each has their own
	/// advantage: `__get_attr__` doesn't require specifying `__this__` (e.g. `__get_attr__('foo')`),
	/// whereas `::` is shorter and generally more idiomatic of other languages.
	///
	/// The `__get_attr__` method is useful when trying to reference a function without automatically
	/// having it become a [`BoundFunction`](crate::types::BoundFunction).
	///
	/// # Arguments
	///
	/// 1. (required) the argument to look up.
	///
	/// # Quest Examples
	/// ```quest
	/// print_fruit = {
	///     disp("I love to eat", _0);
	/// };
	///
	/// ["bananas", "oranges", "melons"].each(print_fruit);
	/// # => I love to eat bananas
	/// # => I love to eat oranges
	/// # => I love to eat melons
	/// ```
	#[instrument(name="Pristine::__get_attr__", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs___get_attr__(this: &Object, args: Args) -> crate::Result<Object> {
		let attr = args.try_arg(0)?;

		this.get_attr(attr)
	}

	/// Set an attribute on the object
	#[instrument(name="Pristine::__set_attr__", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs___set_attr__(this: &Object, args: Args) -> crate::Result<Object> {
		let attr = args.try_arg(0)?;
		let val = args.try_arg(1)?;

		this.set_attr(attr.clone(), val.clone()).map(|_| val.clone())
	}

	#[instrument(name="Pristine::__has_attr__", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs___has_attr__(this: &Object, args: Args) -> crate::Result<Object> {
		let attr = args.try_arg(0)?;

		this.has_attr(attr).map(Object::from)
	}

	#[instrument(name="Pristine::__del_attr__", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs___del_attr__(this: &Object, args: Args) -> crate::Result<Object> {
		let attr = args.try_arg(0)?;

		this.del_attr(attr)
	}

	#[instrument(name="Pristine::.", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs_dot_get_attr(this: &Object, args: Args) -> crate::Result<Object> {
		let attr = args.try_arg(0)?;

		this.dot_get_attr(attr)
	}

	#[instrument(name="Pristine::.?", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs_dot_get_attr_q(this: &Object, args: Args) -> crate::Result<Object> {
		let attr = args.try_arg(0)?;

		if this.has_attr(attr)? {
			this.get_attr(attr)
		} else {
			Ok(Object::default())
		}
	}

	#[instrument(name="Pristine::__keys__", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs___keys__(this: &Object, args: Args) -> crate::Result<Object> {
		let include_parents =
			args.arg(0)
				.map(|x| x.call_downcast::<Boolean>())
				.transpose()?
				.map(|x| x.into_inner())
				.unwrap_or(false);

		Ok(this.mapping_keys(include_parents)?
			.into_iter()
			.map(Object::from)
			.collect::<Vec<_>>()
			.into())
	}

	#[instrument(name="Pristine::instance_exec", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs_instance_exec(this: &Object, args: Args) -> crate::Result<Object> {
		let to_exec = args.try_arg(0)?;

		crate::Binding::run_stackframe(this.clone().into(), |_| {
			if to_exec.has_attr_lit("call_noscope")? {
				to_exec.call_attr_lit("call_noscope", &[])
			} else {
				to_exec.call_attr_lit(&crate::Literal::CALL, &[])
			} 
		})
	}
}

impl_object_type!{
for Pristine [(init_parent) (parents Pristine)]:
	"inspect" => method Self::qs_inspect,
	"__keys__" => method Self::qs___keys__,
	"__call_attr__" => method Self::qs___call_attr__,
	"__get_attr__" => method Self::qs___get_attr__,
	"__set_attr__" => method Self::qs___set_attr__,
	"__has_attr__" => method Self::qs___has_attr__,
	"__del_attr__" => method Self::qs___del_attr__,
	"::" => method Self::qs___get_attr__,
	".=" => method Self::qs___set_attr__,
	"." => method Self::qs_dot_get_attr,
	".?" => method Self::qs_dot_get_attr_q,
	"instance_exec" => method Self::qs_instance_exec,
	"instance_jump" => method |this, args| { // we're now into hacky territory. this is why quest2 is being made...
		let to_exec = args.try_arg(0)?;

		crate::Binding::with_stack(|stack| {
			stack.write().push(this.clone().into());
			let res = if to_exec.has_attr_lit("call_noscope")? {
				to_exec.call_attr_lit("call_noscope", &[])
			} else {
				to_exec.call_attr_lit(&crate::Literal::CALL, &[])
			}?;

			stack.write().push(this.clone().into());
			Ok(res)
		})
	},

	// this is mildly deprecated
	"::@" => method |this, _| {
		crate::Binding::with_stack(|stack| {
			stack.read()
				.first().expect("no stack?")
				.as_ref()
				.get_attr(this)
		})
	}
}
