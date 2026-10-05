# Development helpers. Packaging for end users (deb) is not here yet.

BIN := $(CURDIR)/target/debug/wissel
APPS_DIR := $(if $(XDG_DATA_HOME),$(XDG_DATA_HOME),$(HOME)/.local/share)/applications
DESKTOP := $(APPS_DIR)/wissel.desktop

.PHONY: build dev-install dev-uninstall

build:
	cargo build

# Register the debug binary as a browser candidate for the current user.
# The desktop entry points at the absolute path of the binary because
# target/debug is not on PATH. Does not change the default browser.
dev-install: build
	mkdir -p $(APPS_DIR)
	sed 's|^Exec=wissel |Exec=$(BIN) |' data/wissel.desktop > $(DESKTOP)
	update-desktop-database $(APPS_DIR)
	@echo "installed $(DESKTOP)"

# Remove the desktop entry. If wissel is still the default browser, the
# desktop falls back to the next candidate; see docs/guides/20-default-browser.md
# for restoring the previous default explicitly.
dev-uninstall:
	rm -f $(DESKTOP)
	update-desktop-database $(APPS_DIR)
	@echo "removed $(DESKTOP)"
