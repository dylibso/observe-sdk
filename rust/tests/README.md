# Integration Tests

These integration tests run the examples from `rust/examples/` using the modules from `test/`. The test runner uses `std::process::Command`, captures stdout, and parses that to verify the activity of the example. This means the integration tests are tightly coupled to the examples!

If it becomes burdensome to change the examples then the entire example main functions should be brought into their respective test. This will require a different manner for grabbing the output of the adapters, which itself introduces differences in the test.