# OBVIOUSLY DISCLAIMER
- PRIVATE PROJECT BUT READY TO BE USED AND ADAPTED.
- BUILD AT YOUR OWN RISK, I try to assist.  Message me if you find anything, which is criticial or against the rules.
- I don't provide releases, maybe some tags if a new feature was added but HEAD will always be stable.


# VOTERS VERDICT

User, which are invited, can vote on different candidates in different categories. The voting closes at a given time. While the voting is running, the results can be seen and filtered in different tables. An admin can create custom categories, candidate, voting and styles for each voting. All voters can vote as long as they have an invite code.

- Read the blog entry with images [Voting 2.0](https://confusedoncoffee.github.io/landingpage/)
- Raw link to the markdown file: [here](https://github.com/ConfusedOnCOFFEE/ssg-confusedoncoffee.github.io/blob/main/content/landingpage.md)

## Features

- Seeing all available votings and input the invite_code.
- User Result with sum and score. There are different tables and filters to get most out of the data.

### Admin

- Add user (A user is a voter or candidates) All user can vote if they have an invite_code.
- Add criteria.
- Color selection for his own voting with preview section.
- Checkbox the criterias and candidates, which are available for the vote.
- Setting an end date and an invite code
- Closing votes and accessing the created invite code in case you forgot


# BUILD

## Mandatory
- Requires `export CONTAINER_ENGINE=docker` or `export CONTAINER_ENGINE=podman`.
- Change the secret_key in `Rocket.toml`.

## How to build

- Native: `cargo build --features='templates' --release --bin voters-verdict-machine`
- container: `./manage.sh c`
- container-compose: `./manage.sh up --build`


# Use

Open the page `localhost:8300` or `localhost:9999` depending on the setup and do Admin stuff. Compose file also has port 9999.


# TESTS

## Rust

Cargo tests: `./manage.sh t`

## UI

Playwright E2E tests: `e2e-ui/run.sh`

# Environment variables

Mostly this works with ENV variables and not reading another config file.

## BUILD

| Key                        | Description                                                                  |
|----------------------------|------------------------------------------------------------------------------|
| VOTERS_VERDICT_ENVIRONMENT | Setting the environment (DEV, INT, PROD,...)                                 |
| VOTERS_VERDICT_DATE_DEPLOY | Setting the date to deploy, this shows up as version string in /info/version |


## REQUIRED

This can be changed in 'src/config';

| Key                             | Description                                                                     |
|---------------------------------|---------------------------------------------------------------------------------|
| VOTERS_VERDICT_ADMIN_TOKEN      | Using '/admin/manage?token=' you can get see invite codes and close votings.    |
| VOTERS_VERDICT_MAINTAINER_TOKEN | Using '/admin?token=' you can get access to create new votings, criteria, users |
| VOTERS_VERDICT_ASSET_DIR        | Set the dir, where the assets(js,css, default: static) are located.             |
| VOTERS_VERDICT_FILE_DIR         | Set the dir,** where the bucket (default: bucket) is located.                   |

## OPTIONAL

| Key                                 | Description                                                                                               |
|-------------------------------------|-----------------------------------------------------------------------------------------------------------|
| VOTERS_VERDICT_DEFAULT_VOTE_RUNNING | Setting the environment (DEV, INT, PROD,...)                                                              |
| VOTERS_VERDICT_STORAGE_MODE         | Used to tell the server, if file(2,file), database(1,db) or remote(3,remote) storage mode should be used. |
| VOTERS_VERDICT_DB_URL               | Give DB URL if STORAGE_MODE=1                                                                             |
| VOTERS_VERDICT_REMOTE_STORAGE       | Give URL if STORAGE_MODE=3                                                                                |
| VOTERS_VERDICT_REMOTE_AUTH          | Give credentials if STORAGE_MODE=2                                                                        |
| VOTERS_VERDICT_SQLITE_CONNECTION    | Path to sqlite.                                                                                           |
| VOTERS_VERDICT_SELF_CERT            | Used to tell the server, for a self signed cert.                                                          |
| DATABASE_URL                        | Path to sqlite, in pattern "sqlite://[PATH]"                                                              |


# Development 

## Data objects

**Category and Criteria are the same**

CastBallots -> KnownBallots -> Ballot > Vec<Vote>

- Users
  - Candidate
  - Voter
- Criteria: Vec<Criterion>
- Criterion: The category, to choose a value.
- Voting
- Ballots
  - Cast Ballots (connects to Voting)
  - Known Ballot (connects to a Voter)
  - Ballot
    - Candidate
    - Notes*
    - voted_on
    - votes
  - Vote (Point of one available category)
  - Various Ballots which contains the aggregated ballot with all details, used for table "ballots"
- Point

## API Routes

Console log from the inital startup:

```
(login) GET /
(version_handler) GET /info/version

// ADMIN requres token as queryParam with AuthGuard.
(render_admin_panel) GET /admin/
(render_admin_manage_panel) GET /admin/manage
(render_votings_dev_admin_panel) GET /admin/votings
(render_voting_dev_admin_panel) GET /admin/votings/<voting>

// Serve CSS, JS and emojis.json
(FileServer: static) GET /static/<path..>

// Render Voting(s)
(render_voting_index) GET /votings/
(render_voting) GET /votings/<voting_id>

// Render ballots from a voting
(render_ballots_by_voted_on) GET /ballots/<voting_id>
(render_ballots_sorted) GET /ballots/<voting_id>/results?<sort>
(render_ballots_by_voter) GET /ballots/<voting_id>/voters/<voter>
(render_ballots_by_candidate) GET /ballots/<voting_id>/candidates/<candidate>

// Users (Candidate, Voter)
(get_users) GET /api/v1/users/
(get_users_by_type) GET /api/v1/users/?<type>
(get_user) GET /api/v1/users/<id>
// POST
(post_user) POST /api/v1/users/ application/json


// Voting
(get_raw_vote) GET /api/v1/votings/raw/<voting>
(get_full_vote) GET /api/v1/votings/raw/<voting>?full
// POST requires invite_code
(post_vote) POST /api/v1/votings/ application/json
// PUT requires admin token
(modify_voting) PUT /api/v1/votings/<voting>/add application/json
(close_vote) PUT /api/v1/votings/<voting>/close
// Filtered data from a voting
(get_ballots_by_voted_on) GET /api/v1/ballots/<voting_id>
(get_ballots_by_voting) GET /api/v1/ballots/<voting>/ballots
(get_ballots_sorted) GET /api/v1/ballots/<voting_id>/results?<sort>
(get_ballots_by_voter) GET /api/v1/ballots/<voting_id>/voters/<voter>
(get_ballots_by_candidate) GET /api/v1/ballots/<voting_id>/candidates/<candidate>
// POST
(post_ballot) POST /api/v1/ballots/<voting_id> application/json 

// Criteria
(get_criterias) GET /api/v1/criteria/ application/json
(get_criterion) GET /api/v1/criteria/<criterion>
// POST
(post_criterion) POST /api/v1/criteria/ application/json
```

## SQLITE

Dependencies:
```
sudo apt-get install sqlite3 libsqlite3-dev
```

```
cargo install diesel_cli --no-default-features --features sqlite
DATABASE_URL="db/diesel/db.sqlite" diesel migration --migration-dir db/diesel/migrations redo
```


## JS

| File                 | Description                                               |
|----------------------|-----------------------------------------------------------|
| main                 | Used for voting                                           |
| admin                | Used for admin                                            |
| user-locator         | Used for enable/disable and updating iframe src and links |
| add-emojis-to-labels | Add emojis to labels in a voting                          |
| manage-votings       | Used to put the request to close a voting                 |


## HTML Templates

| File prefix    | Description                                         |
|----------------|-----------------------------------------------------|
| cast           | Root HTML for the tables                            |
| table          | Builds the tables or puts iframe tables             |
| admin          | Used for admin                                      |
| voting         | Render the voting or display the votings in a list. |
| manage-votings | Used to put the request to close a voting.          |
