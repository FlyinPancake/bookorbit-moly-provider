# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3](https://github.com/FlyinPancake/bookorbit-moly-provider/compare/v0.0.2...v0.0.3) - 2026-10-30

### Fixed

- *(mapping)* match ISBNs across forms and keep the matched edition ([#7](https://github.com/FlyinPancake/bookorbit-moly-provider/pull/7))
- *(scraper)* fail on search pages with no results and no no-hits notice ([#8](https://github.com/FlyinPancake/bookorbit-moly-provider/pull/8))
- *(provider)* drop bookUrlTemplate so audiobook links aren't encoded
- *(scraper)* identify as bookorbit-moly-provider in User-Agent

### Other

- *(release)* update README image tag in release PR ([#10](https://github.com/FlyinPancake/bookorbit-moly-provider/pull/10))

## [0.0.2](https://github.com/FlyinPancake/bookorbit-moly-provider/compare/v0.0.1...v0.0.2) - 2026-10-28

### Fixed

- *(mapping)* omit absent fields from results ([#1](https://github.com/FlyinPancake/bookorbit-moly-provider/pull/1))

### Other

- update reusable workflow paths with $ syntax
