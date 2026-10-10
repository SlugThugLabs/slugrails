+++
modules = ["foundation", "cli"]
layers = ["foundation", "cli"]

[allowed_dependencies]
foundation = []
cli = ["foundation"]

enforce = ["structural/layer-dependencies"]
+++

# Architecture Contract
Layer dependencies contract.
