# This is how you would represent classical "classes" within Quest by having
# the "class" be its own object too.
#
# Like in other examples, a "class" is really just an executed block of code
# which returns the local scope.
Person = object() {
	# For `Scope`s, the `@text` attribute checks to see if a `name` field is set.
	# If we set one here, whenever we call `Person.@text`, we'll get this name.
	name = "Person";

	# There is no "constructor," per se. Generally, overloading the "call"
	# operator (i.e. `()`) is used to construct a class (by modifying the scope
	# of the function and returning `:0` (which means `self`/`this` in other
	# languages) at the end), but this is just a convention.
	'()' = (class, first, last) -> {
		# Since we're within a scope, `__parents__` defaults to `Scope`. We want to
		# change that so our parents is just `Person`'s instance methods.
		__parents__ = [class.instance_methods];

		# Idiomatically, you would use the varidict `becomes` function instead of directly modifying `__parents__`:
		:0.becomes(class.instance_methods);

		# We return the current scope, as it's the current object.
		:0
	};

	instance_methods = object() {
		@text = person -> {
			person.first + " " + person.last
		};
	};
};

sam = Person("Sam", "W");

print(Person); # => Person
print(sam); # => Sam W
print(sam.class == Person); # => true

# Tests
assert(Person.@text() == "Person");
assert(sam.@text() == "Sam W");
assert(sam.class == Person);
