#!/usr/bin/env fish

function clippy
  cargo clippy --all-targets --all-features -- -D warnings
end

function fmt
  cargo +nightly fmt --all -- $argv
end

function usage
  echo "Quick access to clippy and rustfmt
Usage: cq.fish COMMAND

COMMAND:
  clippy        lint code using clippy
  fmt [args..]  format code using rustfmt"
end

function main
  set -x subcmd $argv[1]
  switch $subcmd
    case clippy
      clippy
    case fmt
      fmt $argv[1..-1]
    case -h --help
      usage
    case '*'
      echo "Unknown command: $subcmd.\nTry --help" >&2
      exit 1
  end
end

main $argv
