#TODO

1. Get the ReadState stuff to a point where it compiles
1. Get the test using ReadState and ::send() running smoothly
1. Check correctness of ReadState
1. Figure out a consistent naming convention
1. pull out the sending / receiving functions into a trait (think of good names)
1. Build the WrapperBidir struct. determine if you need marker::Sized. implement traits
1. Build the WrapperWriter, WrapperReader and WrapperPair that appropriately implement traits
1. Implement suitable transitions between the wrappers etc.
1. Add some ultility functions
1. Create a bunch of tests and comments on the tests to illustrate use cases