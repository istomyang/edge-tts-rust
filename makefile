SHELL := /usr/bin/env bash

# Makefile settings
ifndef V
MAKEFLAGS += --no-print-directory
endif

ARROW_STRING := ===========>

.DEFAULT_GOAL := help

# Useful when replace string.
COMMA := ,
SPACE :=
SPACE +=

### git-tag: Create git tag.
.PHONY : git-tag
git-tag :
	$(eval version := $(shell cargo read-manifest --manifest-path Cargo.toml | jq -r '.version'))
	@echo "${ARROW_STRING} Creating git tag v${version}."
	@git tag -a v${version} -m "v${version}"


### help: Show this help info.
.PHONY: help
help: Makefile
	@printf "\nUsage: make <TARGETS> <OPTIONS> ...\n\nTargets:\n"
	@sed -n 's/^###//p' $< | column -t -s ':' | sed -e 's/^/ /'
	@echo ""
	@echo "$$USAGE_OPTIONS"