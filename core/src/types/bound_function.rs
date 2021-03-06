use crate::{Object, Result, Args, Literal};
use tracing::instrument;

/// A type representing a bound function.
///
/// This will be redesigned in the future, as it's not working ideally.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundFunction;

impl BoundFunction {
	/// Call this function with the specified args, passing them on to the unbound object.
	#[instrument(name="BoundFunction::()", level="trace", skip(this, args), fields(self=?this, ?args))]
	pub fn qs_call(this: &Object, args: Args) -> Result<Object> {
		let bound_owner = &this.get_attr_lit("__bound_object_owner__")?;
		let bound_object = this.get_attr_lit("__bound_object__")?;
		let args: Args = std::iter::once(bound_owner).chain(args.into_iter()).collect();
		bound_object.call_attr_lit(&Literal::CALL, args)
	}
}

impl_object_type!{
for BoundFunction [(parents super::Function)]:
// for BoundFunction [(parents super::Basic)]:
	"()" => method Self::qs_call,
}
