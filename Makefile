run-common-tests:
	@echo "Running common tests..."
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test.ps1

compile-latex-doc:
	pdflatex -output-directory=docs/latex/build docs/latex/raw/main.tex