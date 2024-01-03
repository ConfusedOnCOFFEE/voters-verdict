#!/bin/bash

VOTERS_VERDICT_DATE_DEPLOY=$(date +"%d.%m.%Y")
log=$RUST_LOG
echo_help_msg() {
    echo "$0 helps you to build and setup your voting machine.

    Subcommands:

     c|container               Builds the contianer with the desired tag, where ARG1 is container_name:container_tag
                               This requires CONTAINER_ENGINE variable to be set with docker or podman.
     up                        Uses the compose.yml to build and run the container.
     t|test                    Runs cargo tests.
     r|run                     Runs the binary of the server in debug compilation.
     db                        Short steps for diesel-cli.
     db:s, diesel:setup        Setup diesel.
     db:r, diesel:run          Run DB migration.
     db:reset, diesel:reset    Reest DB.
     db:g, diesel:generate     Generate a new DB migration step.
     db:redo, diesel:redo      Redo a complete DB migratin.

     Please take a look at src/config.rs. The following list is just the mandatory stuff.
     ENVIRONMENT Variable              Default
     VOTERS_VERDICT_FILE_DIR          `pwd`/test-data/
     VOTERS_VERDICT_ASSET_DI          `pwd`/static/
     VOTERS_VERDICT_ENVIRONMENT        test
     VOTERS_VERDICT_ADMIN_TOKEN        111
     VOTERS_VERDICT_MAINTAINER_TOKEN   123
"
    exit 0
}

export VOTERS_VERDICT_FILE_DIR=`pwd`/test-data/
export VOTERS_VERDICT_ASSET_DIR=`pwd`/static/
export VOTERS_VERDICT_ENVIRONMENT=test
export VOTERS_VERDICT_ADMIN_TOKEN=111
export VOTERS_VERDICT_MAINTAINER_TOKEN=123

echo "We have log level: $log"
case "$1" in
    "help"|"h"|"-h"|"--help")
        echo_help_msg
        ;;
    "c"|"container")
        if [[ -z "$CONTAINER_ENGINE" && -z "$2" ]]; then
            echo "Please set CONTAINER_ENGINE and provide container_name:contiane_tag."
            exit 0
        fi
        $CONTAINER_ENGINE build --build-arg DATE_DEPLOY="$VOTERS_VERDICT_DATE_DEPLOY" --build-arg TARGET=x86_64 -f Containerfile -t $2 .
        ;;
    "up")
        if [[ -z "$CONTAINER_ENGINE" && -z "$2" ]]; then
            echo "Please set CONTAINER_ENGINE"
            exit 0
        fi
        if [[ "$3" == "--build" ]]; then
            $CONTAINER_ENGINE-compose -f contianer-compose.yml up --build
        else
            $CONTAINER_ENGINE-compose -f contianer-compose.yml up
        fi
        ;;
    "t"|"test")
        cargo test --bin voters-verdict-machine --features=admin,db,sqlx_sqlite
        ;;
    "fix")
        cargo fix --bin "voters-verdict-machine" --features="admin,db,sqlx_sqlite"
        ;;
    "r"|"run")
        RUST_LOG=$log DATABASE_URL=sqlite://db/db.sqlite VOTERS_VERDICT_SQLITE_CONNECTION=db/db.sqlite cargo run --bin voters-verdict-machine --features=admin,db,sqlx_sqlite
        ;;
    "db:s"|"diesel:setup")
        DATABASE_URL="db/db.sqlite" diesel setup
        ;;
    "db:r"|"diesel:run")
        DATABASE_URL="db/db.sqlite" diesel migration run
        ;;
    "db:reset"|"diesel:reset")
        DATABASE_URL="db/db.sqlite" diesel database reset
        ;;
    "db:g"|"diesel:generate")
        DATABASE_URL="db/db.sqlite" diesel migration generate $2
        ;;
    "db:redo"|"diesel:redo")
        DATABASE_URL="db/db.sqlite" diesel migration --migration-dir migrations/ redo
        ;;
    "sql3")
        sqlite3 db/db.sqlite
        ;;
    "w")
        DATABASE_URL="db/db.sqlite" cargo-watch -x 'run --features=admin,db,sqlx_sqlite'
        ;;
    *)
        echo_help_msg
        ;;
esac
