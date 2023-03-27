Setup
  $ . ${TESTDIR}/../../setup.sh
  $ . ${TESTDIR}/../setup.sh $(pwd)

# Delete all run summaries
  $ rm -rf .turbo/runs

  $ TURBO_RUN_SUMMARY=true ${TURBO} run build -- someargs > /dev/null # first run (should be cache miss)

# no output, just check for 0 status code, which means the directory was created
  $ test -d .turbo/runs
# expect 2 run summaries are created
  $ ls .turbo/runs/*.json | wc -l
  \s*1 (re)

# get jq-parsed output of each run summary
  $ FIRST=$(/bin/ls .turbo/runs/*.json | head -n1)

# some top level run summary validation
  $ cat $FIRST | jq '.tasks | length'
  2
  $ cat $FIRST | jq '.version'
  "0"
  $ cat $FIRST | jq '.executionSummary.exitCode'
  0
  $ cat $FIRST | jq '.executionSummary.attempted'
  2
  $ cat $FIRST | jq '.executionSummary.cached'
  0
  $ cat $FIRST | jq '.executionSummary.failed'
  0
  $ cat $FIRST | jq '.executionSummary.success'
  2
  $ cat $FIRST | jq '.executionSummary.startTime'
  [0-9]+ (re)
  $ cat $FIRST | jq '.executionSummary.endTime'
  [0-9]+ (re)

# Extract some task-specific summaries from each
  $ FIRST_APP_BUILD=$("$TESTDIR/get-build.sh" "$FIRST" "my-app")  
  $ FIRST_UTIL_BUILD=$("$TESTDIR/get-build.sh" "$FIRST" "util")

  $ echo $FIRST_APP_BUILD | jq '.execution'
  {
    "startTime": [0-9]+, (re)
    "endTime": [0-9]+, (re)
    "status": "built",
    "error": null,
    "exitCode": 0
  }
  $ echo $FIRST_APP_BUILD | jq '.commandArguments'
  [
    "someargs"
  ]

  $ echo $FIRST_APP_BUILD | jq '.hashOfExternalDependencies'
  "ccab0b28617f1f56"
  $ echo $FIRST_APP_BUILD | jq '.expandedOutputs'
  [
    "apps/my-app/.turbo/turbo-build.log"
  ]
  
  $ echo $FIRST_APP_BUILD | jq '.cacheState'
  {
    "local": false,
    "remote": false
  }
  $ echo $SECOND_APP_BUILD | jq '.cacheState'
  {
    "local": true,
    "remote": false
  }

# Some validation of util#build
  $ echo $FIRST_UTIL_BUILD | jq '.execution'
  {
    "startTime": [0-9]+, (re)
    "endTime": [0-9]+, (re)
    "status": "built",
    "error": null,
    "exitCode": 0
  }

# another#build is not in tasks, because it didn't execute (script was not implemented)
  $ "$TESTDIR/get-build.sh" $FIRST "another"
  null

# Without env var, no summary file is generated
  $ rm -rf .turbo/runs
  $ ${TURBO} run build > /dev/null
# validate with exit code so the test works on macOS and linux
  $ test -d .turbo/runs
  [1]