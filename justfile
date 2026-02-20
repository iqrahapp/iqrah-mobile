set shell := ["bash", "-cu"]

default:
    @just --list

# Regenerate Dart API client from latest openapi.json
gen-api:
    flutter pub run build_runner build --delete-conflicting-outputs
