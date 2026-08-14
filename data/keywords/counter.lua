-- Counter is an event operation. A specific Secret calls ctx:cancel_event(event)
-- from its before trigger, with no keyword-specific branch in Rust.
return { api_version = 1, module_type = "keyword", id = "counter", name = "Counter" }
