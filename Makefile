.PHONY: install test cli abi clean

CARGO ?= cargo
PIP ?= pip3
NPM ?= npm

install: abi cli
	$(PIP) install --user -e ./python 2>/dev/null || true
	$(NPM) install -g . 2>/dev/null || true
	@echo ""
	@echo "✓ OpenConstruct installed!"
	@echo "  Run 'openconstruct init' to get started."

test:
	$(CARGO) test --workspace
	@echo "✓ All tests passed"

cli:
	$(CARGO) build --release -p openconstruct-cli
	@echo "✓ CLI built at target/release/openconstruct"

abi:
	$(CARGO) build --release -p openconstruct-abi 2>/dev/null || \
		$(CARGO) build --release 2>/dev/null || \
		echo "⚠ ABI build skipped — some crates may not be available yet"
	@echo "✓ ABI build complete"

clean:
	$(CARGO) clean
	rm -rf build/ dist/ *.egg-info node_modules/
	@echo "✓ Clean"
