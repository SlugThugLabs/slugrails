+++
modules = ["foundation", "core"]
layers = ["foundation", "core"]

[allowed_dependencies]
foundation = []
core = ["foundation"]

enforce = [
    "complexity/source-limits",
    "complexity/function-limits",
    "complexity/deep-nesting",
    "languages/rust/no-unwrap",
    "languages/rust/no-panic",
    "languages/rust/unsafe-policy"
]
+++

# Architecture Contract
Clean valid architecture specification.
