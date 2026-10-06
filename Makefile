# Development helpers.
#
# Machine-specific settings go in local.mk, which Git ignores; see
# docs/guides/20-default-browser.md. It may set CARGO or export PATH.
-include local.mk

CARGO ?= cargo
BIN := $(CURDIR)/target/debug/wissel
APPS_DIR := $(if $(XDG_DATA_HOME),$(XDG_DATA_HOME),$(HOME)/.local/share)/applications
DESKTOP := $(APPS_DIR)/io.github.x7c1.wissel.desktop

.PHONY: build check dev-install dev-uninstall

build:
	$(CARGO) build

# The canonical gate. CI runs this target, so the gate has one definition.
check:
	$(CARGO) fmt --check
	$(CARGO) build --locked
	$(CARGO) test --locked
	$(CARGO) clippy --all-targets -- -D warnings
	bash .github/scripts/release-summary.test.sh
	bash .github/scripts/validate-release-pr-title.test.sh
	for f in .github/scripts/*.sh scripts/*.sh; do bash -n "$$f" || exit 1; done

# Register the debug binary as a browser candidate for the current user.
# The desktop entry points at the absolute path of the binary because
# target/debug is not on PATH. Does not change the default browser.
dev-install: build
	mkdir -p $(APPS_DIR)
	sed 's|^Exec=wissel |Exec=$(BIN) |' data/io.github.x7c1.wissel.desktop > $(DESKTOP)
	update-desktop-database $(APPS_DIR)
	@echo "installed $(DESKTOP)"

# Remove the desktop entry. Restore the previous default browser first;
# see docs/guides/20-default-browser.md.
dev-uninstall:
	rm -f $(DESKTOP)
	update-desktop-database $(APPS_DIR)
	@echo "removed $(DESKTOP)"
