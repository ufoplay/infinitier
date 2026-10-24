# Formats the code
[group('code_check')]
fmt args="":
  cargo fmt --all {{args}}


# Runs clippy on the code
[group('code_check')]
clippy args="":
  cargo clippy --all-features --all-targets {{args}}


# Runs fmt and clippy on the code. It fails if any of the checks fail
[group('code_check')]
check_code:
    just fmt "--check"
    just clippy "-- -D warnings"