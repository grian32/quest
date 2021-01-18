use crate::value::NamedType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Text {
	data: Vec<u8>
}

impl NamedType for Text {
	#[inline(always)]
	fn typename() -> &'static str {
		"Text"
	}
}

// impl ValueType for Text {
	
// }
