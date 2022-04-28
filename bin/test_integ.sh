#!/bin/bash
set -e

export PATH=target/debug:$PATH
mkdir -p target/html

for grammar_file in integration/test_ok/*.grammar; do
  echo "grammar $grammar_file"
  output=$(grammar "$grammar_file")
  exit_code="$?"
  if [[ "$exit_code" != 0 ]]
  then
     echo "Exit code = $exit_code"
     exit 1
  fi

  expected_file="${grammar_file%.*}.html"
  expected=$(cat "$expected_file")
  if [[ "$expected" != "$output" ]]
  then
     echo "HTML Output differs"
     exit 1
  fi
done

for grammar_file in integration/test_error/*.grammar; do
  echo "grammar $grammar_file"
  set +e
  output=$(grammar "$grammar_file" 2>&1)
  exit_code="$?"
  set -e

  expected_exit_code_file="${grammar_file%.*}.exit"
  expected=$(cat "$expected_exit_code_file")
  if [[ "$exit_code" != "$expected" ]]
  then
     echo "Exit code"
     echo "  actual: $exit_code"
     echo "  expected: $expected"
     exit 1
  fi

  expected_error_file="${grammar_file%.*}.error"
  expected=$(cat "$expected_error_file")
  if [[ "$output" != "$expected" ]]
  then
     echo "error message differs"
     exit 1
  fi

done


