Set-StrictMode -Version latest
$ErrorActionPreference = 'Stop'

hurl --no-output tests_ok/query_cache.hurl
