run-common-tests:
	@echo "Running common tests..."
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test.ps1

compile-latex-doc:
	pdflatex -output-directory=docs/latex/build docs/latex/raw/main.tex

build-py:
	maturin build --release --manifest-path crates/bindings/rnb_py/Cargo.toml

publish-py:
	maturin publish --release --manifest-path crates/bindings/rnb_py/Cargo.toml

test-docs:
	@echo "Building Zensical site..."
	zensical build
	@echo "Compiling LaTeX spec..."
	$(MAKE) compile-latex-doc

dev-web:
	@echo "Starting live-server for web (requires npm live-server)"
	cd web && live-server --port=9017

test-all:
	$(MAKE) run-common-tests
	$(MAKE) test-docs
