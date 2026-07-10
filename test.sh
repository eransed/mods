#!/bin/bash

if cargo check; then
  echo "Cargo check passed."
else
  echo "Cargo check failed."
  exit 1
fi

if cargo clippy; then
  echo "Cargo clippy passed."
else
  echo "Cargo clippy failed."
  exit 1
fi

if cargo test; then
  echo "Cargo tests passed."
else
  echo "Cargo tests failed."
  exit 1
fi

cd ui

if npx playwright test; then
  echo "Playwright tests passed."
else
  echo "Playwright tests failed."
  exit 1
fi
