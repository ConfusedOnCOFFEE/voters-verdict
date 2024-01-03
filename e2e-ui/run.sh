#!/bin/bash

echo_help_msg() {
    echo "$0 helps you to run the server with the E2E test data and run E2E on it.

    Subcommands:

     s|server   Starts the server wih the required environment variables.
      s:file    Starts server with file feature.
      s:db      Starts server wit db sqlite feature.
                Second argument 'redo' cleans the DB.
     t|test     Runs Playwright in headless mode for all browsers.
                Requires f(irefox), c(hrome), w(ebkit), s(safari) as first argument.
     t:single   Runs Playwright in headless mode for all browsers.
                Requires f(irefox), c(hrome), w(ebkit), s(safari) as first argument.
                Requires file identifier as second argument.
     ui         Opoens Playwright with one worker in UI mode.
"
    exit 0
}

empty_json() {
    echo "[]" >> e2e-bucket/$1/index.json
}

execute_test_by_filter() {
    npx playwright test $1.spec.ts --project $2
}
execute_test_run() {
    browser=
    case "$1" in
        "firefox"|"f")
            browser=firefox
            ;;
        "chromium"|"c")
            browser=chromium
            ;;
        "webkit"|"s")
            browser=webkit
            ;;
    esac
    set -e
    for file in setup voting ballots tables
    do
        execute_test_by_filter $file $browser
    done
}
case "$1" in
    "help"|"h"|"-h"|"--help")
        echo_help_msg
        ;;
    "s:file")
        cd  ../
        mkdir -p e2e-bucket/candidates e2e-bucket/criteria e2e-bucket/ballots e2e-bucket/votings
        empty_json candidates
        empty_json criteria
        empty_json ballots
        empty_json votings
        export VOTERS_VERDICT_ADMIN_TOKEN=12345
        export VOTERS_VERDICT_ENVIRONMENT=dev
        export VOTERS_VERDICT_FILE_DIR_DIR=`pwd`/e2e-bucket/
        export VOTERS_VERDICT_ASSET_DIR=`pwd`/static/
        RUST_LOG=info cargo run --bin voters-verdict-machine --features=templates,local,admin
        exit 0
        ;;
    "s:db")
        cd ../
        export VOTERS_VERDICT_ADMIN_TOKEN=12345
        export VOTERS_VERDICT_ENVIRONMENT=dev
        export VOTERS_VERDICT_STORAGE_MODE=1
        export VOTERS_VERDICT_ASSET_DIR=`pwd`/static/
        export DATABASE_URL=sqlite://db/e2e.sqlite
        export VOTERS_VERDICT_SQLITE_CONNECTION=db/e2e.sqlite
        if [[ "$2" == "redo" ]]; then
            DATABASE_URL=db/e2e.sqlite diesel migration --migration-dir migrations/ redo
        fi
        cargo run --bin voters-verdict-machine --features=admin,db,sqlx_sqlite
        exit 0
        ;;
    "t"|"test")
        if [[ -z "$2" ]]; then
            echo "Please provide a browser f(irefox), c(hrome), w(ebkit), s(safari)."
            exit 1
        fi
        execute_test_run $2
        rm -rf e2e-bucket
        exit 0
        ;;
    "t:single")

        if [[ -z "$2" && "$1" ]]; then
            echo "Please provide a browser f(irefox), c(hrome), w(ebkit), s(safari)."
            echo "Please provide a valid file name."
            exit 1
        fi
        execute_test_by_filter $3 $2
        rm -rf e2e-bucket
        exit 0
        ;;
    "ui")
        npx playwright test --ui
        exit 0
        ;;
    *)
        echo_help_msg
        ;;
esac
