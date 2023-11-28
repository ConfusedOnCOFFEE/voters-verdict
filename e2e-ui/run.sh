#!/bin/bash

echo_help_msg() {
    echo "$0 helps you to run the server with the E2E test data and run E2E on it.

    Subcommands:

     s|server   Starts the server wih the required environment variables.
     t|test     Runs Playwright in headless mode for all browsers.
     ui         Opoens Playwright with one worker in UI mode.
"
    exit 0
}

case "$1" in
    "help"|"h"|"-h"|"--help")
        echo_help_msg
        ;;
    "s")
        cd  ../
        mkdir -p e2e-bucket/candidates e2e-bucket/criteria e2e-bucket/ballots e2e-bucket/votings
        export VOTERS_VERDICT_ADMIN_TOKEN=12345
        export VOTERS_VERDICT_ENVIRONMENT=dev
        export VOTERS_VERDICT_FILE_DIR=`pwd`/e2e-bucket/
        export VOTERS_VERDICT_ASSET_DIR=`pwd`/static/
        cargo run --bin voters-verdict-machine --features=templates
        exit 0
        ;;
    "t")
        npx playwright test setup.spec.ts
        npx playwright test voting.spec.ts
        npx playwright test ballots.spec.ts
        npx playwright test tables.spec.ts
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
