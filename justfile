doc_html_dir := ""
runtime_api := doc_html_dir / "runtime-api.json"
prototype_api := doc_html_dir / "prototype-api.json"

dev-build:
    fmtk package --outdir build

generate-info-fresh:
    SEDC_CREATE_CACHE=1 rust-script generate-info.rs

generate-info-cached:
    SEDC_CACHED=1 SEDC_CREATE_CACHE=1 rust-script generate-info.rs

fresh-build: generate-info-fresh dev-build

cached-build: generate-info-cached dev-build

setup-dev-workspace:
    test -f {{ runtime_api }}
    test -f {{ prototype_api }}
    fmtk luals-addon -d {{ runtime_api }} -p {{ prototype_api }} ./fmtk
