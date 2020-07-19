use crate::{Object, Args};
use std::fmt::{self, Debug, Display, Formatter};
use std::convert::TryFrom;
use crate::types::Text;

mod flag;

pub use flag::{Flag, Flags};

/// An error that is caused by a bad regex being parsed.
pub use ::regex::Error as RegexError;

#[derive(Debug, Clone)]
pub struct Regex(regex::Regex, Flags);

impl Default for Regex {
	#[inline]
	fn default() -> Self {
		Self::new("").expect("default shouldn't fail")
	}
}

impl Display for Regex {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "/{}/{}", self.0.as_str(), self.1)
	}
}

impl Eq for Regex {}
impl PartialEq for Regex {
	#[inline]
	fn eq(&self, rhs: &Regex) -> bool {
		self.0.as_str() == rhs.0.as_str() && self.1 == rhs.1
	}
}

impl Regex {
	#[inline]
	pub fn new(rxp: &str) -> Result<Self, RegexError> {
		Self::try_from(rxp)
	}

	#[inline]
	pub fn new_with_options(rxp: &str, flags: Flags) -> Result<Self, RegexError> {
		let mut builder = ::regex::RegexBuilder::new(rxp);
		for flag in flags {
			flag.set_option(&mut builder);
		}
		Ok(Self(builder.build()?, flags))
	}
}

impl AsRef<regex::Regex> for Regex {
	#[inline]
	fn as_ref(&self) -> &regex::Regex {
		&self.0
	}
}

impl<'a> TryFrom<&'a str> for Regex {
	type Error = RegexError;

	#[inline]
	fn try_from(rxp: &'a str) -> Result<Self, RegexError> {
		Self::new_with_options(rxp, Flags::default())
	}
}

impl From<Regex> for Text {
	#[inline]
	fn from(rxp: Regex) -> Self {
		Self::from(rxp.to_string())
	}
}

/// Quest functions
impl Regex {
	/// Inspects the [`Regex`].
	#[inline]
	pub fn qs_inspect(this: &Object, args: Args) -> crate::Result<Object> {
		Self::qs_at_text(this, args)
	}

	/// Convert this into a [`Text`].
	#[inline]
	pub fn qs_at_text(this: &Object, _: Args) -> crate::Result<Object> {
		this.try_downcast_map(|this: &Self| Text::from(this.to_string()).into())
	}

	/// Compares two [`Regex`]s
	pub fn qs_eql(this: &Object, args: Args) -> crate::Result<Object> {
		let rhs = args.arg(0)?;
		this.try_downcast_and_then(|this: &Self| {
			rhs.try_downcast_map(|rhs: &Self| {
				(this == rhs).into()
			})
		})
	}

	/// Returns an Array of matched values.
	///
	/// The first argument is converted to a [`Text`] before matching.
	pub fn qs_match(this: &Object, args: Args) -> crate::Result<Object> {
		let rhs = args.arg(0)?;

		this.try_downcast_and_then(|this: &Self| {
			rhs.call_downcast_map(|rhs: &Text| {
				this.0
					.captures(rhs.as_ref())
					.map(|x| x.iter().map(|m| {
							m.map(|m| Object::from(m.as_str().to_string()))
								.unwrap_or_default()
						}).collect::<Vec<_>>()
						.into()
					).unwrap_or_default()
			})
		})
	}

	/// Checks to see if the first argument matches.
	///
	/// The first argument is converted to a [`Text`] before matching.
	pub fn qs_does_match(this: &Object, args: Args) -> crate::Result<Object> {
		let rhs = args.arg(0)?;

		this.try_downcast_and_then(|this: &Self| {
			rhs.call_downcast_map(|rhs: &Text| {
				this.0.is_match(rhs.as_ref()).into()
			})
		})
	}
}

impl_object_type!{
for Regex [(parents super::Basic) (convert "@regex")]:
	"@text"   => function Regex::qs_at_text,
	"inspect" => function Regex::qs_inspect,
	"=="      => function Regex::qs_eql,
	"does_match" => function Regex::qs_does_match,
	"match" => function Regex::qs_match,
}