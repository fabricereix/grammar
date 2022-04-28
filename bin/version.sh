#!/bin/bash
grep '^version' Cargo.toml | cut -f2 -d'"'
