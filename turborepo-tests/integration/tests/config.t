Setup
  $ . ${TESTDIR}/../../helpers/setup.sh
  $ . ${TESTDIR}/_helpers/setup_monorepo.sh $(pwd)

Run test run
  $ ${TURBO} run build --__test-run | jq .remote_config
  {
    "token": null,
    "team_id": null,
    "team_slug": null,
    "api_url": "https://vercel.com/api"
  }

Run test run with api overloaded
  $ ${TURBO} run build --__test-run --api http://localhost:8000 | jq .remote_config
  {
    "token": null,
    "team_id": null,
    "team_slug": null,
    "api_url": "http://localhost:8000"
  }

Run test run with token overloaded
  $ ${TURBO} run build --__test-run --token 1234567890 | jq .remote_config
  {
    "token": "1234567890",
    "team_id": null,
    "team_slug": null,
    "api_url": "https://vercel.com/api"
  }

Run test run with team overloaded
  $ ${TURBO} run build --__test-run --team vercel | jq .remote_config
  {
    "token": null,
    "team_id": null,
    "team_slug": "vercel",
    "api_url": "https://vercel.com/api"
  }

Run test run with team overloaded from both env and flag (flag should take precedence)
  $ TURBO_TEAM=vercel ${TURBO} run build --__test-run --team turbo | jq .remote_config
  {
    "token": null,
    "team_id": null,
    "team_slug": "turbo",
    "api_url": "https://vercel.com/api"
  }

Run test run with remote cache timeout env variable set
  $ TURBO_REMOTE_CACHE_TIMEOUT=123 ${TURBO} run build --__test-run | jq .cli_args.remote_cache_timeout
  123

Run test run with remote cache timeout from both env and flag (flag should take precedence)
  $ TURBO_REMOTE_CACHE_TIMEOUT=123 ${TURBO} run build --__test-run --remote-cache-timeout 456 | jq .cli_args.remote_cache_timeout
  456