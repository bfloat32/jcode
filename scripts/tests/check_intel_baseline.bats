#!/usr/bin/env bats

setup() {
  guard="$BATS_TEST_DIRNAME/../check_intel_baseline.sh"
  work="$(mktemp -d "${BATS_TEST_TMPDIR:-/tmp}/jcode-intel-baseline.XXXXXX")"
  repo="$work/repo"
  stdout_file="$work/stdout"
  stderr_file="$work/stderr"
}

teardown() {
  rm -rf "$work"
}

fixture_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print tolower($1)}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | awk '{print tolower($1)}'
  else
    openssl dgst -sha256 | awk '{print tolower($NF)}'
  fi
}

commit_fixture() {
  GIT_MASTER=1 GIT_AUTHOR_NAME=Fixture GIT_AUTHOR_EMAIL=fixture@example.test \
    GIT_COMMITTER_NAME=Fixture GIT_COMMITTER_EMAIL=fixture@example.test \
    git -C "$repo" commit -q -m "$1"
}

write_manifest() {
  local revision="$1"
  local signature
  signature="$(printf 'BEGIN\ninside\n' | fixture_sha256)"

  cat > "$repo/tools/intel/baseline.toml" <<EOF
  schema_version = 2
signature_slice_schema = "jcode-semantic-slices-v2"
hash_algorithm = "sha256"
slice_order = "ascending order field"
marker_cardinality = "start and end must each match exactly one ASCII-trimmed line"
slice_bounds = "start inclusive, end exclusive, with start preceding end"

required_paths = [
  "critical.txt",
]

[jcode]
revision = "$revision"

[agentgrep]
version = "0.1.6"
tag = "v0.1.6"
revision = "b01b804008ab0662fa14e6b60b10bff61716e6f1"
repository = "https://github.com/1jehuang/agentgrep.git"

[normalization."rust-v1"]
decode = "strict UTF-8; strip one leading UTF-8 BOM if present"
line_endings = "replace CRLF and CR with LF"
marker_match = "exact equality after trimming ASCII space and tab from both ends"
extract = "include the unique start-marker line; exclude the unique end-marker line"
line_transform = "trim ASCII space and tab from both ends"
drop_empty_lines = true
drop_line_prefixes = ["//"]
join = "LF between retained lines and one terminal LF"
encode = "UTF-8 without BOM before SHA-256"

[normalization."toml-v1"]
decode = "strict UTF-8; strip one leading UTF-8 BOM if present"
line_endings = "replace CRLF and CR with LF"
marker_match = "exact equality after trimming ASCII space and tab from both ends"
extract = "include the unique start-marker line; exclude the unique end-marker line"
line_transform = "trim ASCII space and tab from both ends"
drop_empty_lines = true
drop_line_prefixes = ["#"]
join = "LF between retained lines and one terminal LF"
encode = "UTF-8 without BOM before SHA-256"

[normalization."text-v1"]
decode = "strict UTF-8; strip one leading UTF-8 BOM if present"
line_endings = "replace CRLF and CR with LF"
marker_match = "exact equality after trimming ASCII space and tab from both ends"
extract = "include the unique start-marker line; exclude the unique end-marker line"
line_transform = "trim ASCII space and tab from the right only"
drop_empty_lines = false
drop_line_prefixes = []
join = "LF between retained lines and one terminal LF"
encode = "UTF-8 without BOM before SHA-256"

[[signature_slice]]
order = 1
id = "critical"
path = "critical.txt"
normalization = "text-v1"
start_marker = "BEGIN"
end_marker = "END"
sha256 = "$signature"
EOF
}

write_workspace_cargo() {
  local state="$1"

  case "$state" in
    B0)
      printf '%s\n' '[workspace]' 'members = []' '[lib]' > "$repo/Cargo.toml"
      ;;
    B1)
      printf '%s\n' '[workspace]' 'members = [' \
        '    "crates/jcode-intel-types",' \
        '    "crates/jcode-intel-store",' \
        '    "crates/jcode-intel-search",' \
        '    "crates/jcode-intel-provider",' \
        '    "crates/jcode-intel-rust",' \
        '    "crates/jcode-intel-core",' \
        '    "crates/jcode-intel-eval",' \
        ']' '[lib]' > "$repo/Cargo.toml"
      ;;
    B3)
      printf '%s\n' '[workspace]' 'members = [' \
        '    "crates/jcode-intel-types",' \
        '    "crates/jcode-intel-store",' \
        '    "crates/jcode-intel-search",' \
        '    "crates/jcode-intel-provider",' \
        '    "crates/jcode-intel-rust",' \
        '    "crates/jcode-intel-core",' \
        '    "crates/jcode-intel-eval",' \
        '    "crates/jcode-intel-evil",' \
        ']' '[lib]' > "$repo/Cargo.toml"
      ;;
    B2)
      printf '%s\n' '[workspace]' 'members = [' \
        '    "crates/jcode-intel-types",' \
        '    "crates/jcode-intel-store",' \
        '    "crates/jcode-intel-search",' \
        '    "crates/jcode-intel-provider",' \
        '    "crates/jcode-intel-rust",' \
        '    "crates/jcode-intel-core",' \
        '    "crates/jcode-intel-eval",' \
        '    "crates/arbitrary",' \
        ']' '[lib]' > "$repo/Cargo.toml"
      ;;
    B4)
      printf '%s\n' '[workspace]' 'members = [' \
        '    "crates/jcode-intel-types",' \
        '    "crates/jcode-intel-store",' \
        '    "crates/jcode-intel-search",' \
        '    "crates/jcode-intel-provider",' \
        '    "crates/jcode-intel-rust",' \
        '    "crates/jcode-intel-core",' \
        ']' '[lib]' > "$repo/Cargo.toml"
      ;;
    *) return 1 ;;
  esac
}

write_workspace_manifest() {
  local revision="$1"
  local primary='01682718dc81bf6ca83b17e5a4b7fd755dc44c08357e0c817c20d66e5f474a78'
  local alternate='1cf8ad1c3f365dc8d6815a3e674639209b2a3c79581e348c97c41e4deb8a5529'

  cat > "$repo/tools/intel/baseline.toml" <<EOF
schema_version = 2
signature_slice_schema = "jcode-semantic-slices-v2"
hash_algorithm = "sha256"
slice_order = "ascending order field"
marker_cardinality = "start and end must each match exactly one ASCII-trimmed line"
slice_bounds = "start inclusive, end exclusive, with start preceding end"

required_paths = [
  "Cargo.toml",
]

[jcode]
revision = "$revision"

[agentgrep]
version = "0.1.6"
tag = "v0.1.6"
revision = "b01b804008ab0662fa14e6b60b10bff61716e6f1"
repository = "https://github.com/1jehuang/agentgrep.git"

[normalization."rust-v1"]
decode = "strict UTF-8; strip one leading UTF-8 BOM if present"
line_endings = "replace CRLF and CR with LF"
marker_match = "exact equality after trimming ASCII space and tab from both ends"
extract = "include the unique start-marker line; exclude the unique end-marker line"
line_transform = "trim ASCII space and tab from both ends"
drop_empty_lines = true
drop_line_prefixes = ["//"]
join = "LF between retained lines and one terminal LF"
encode = "UTF-8 without BOM before SHA-256"

[normalization."toml-v1"]
decode = "strict UTF-8; strip one leading UTF-8 BOM if present"
line_endings = "replace CRLF and CR with LF"
marker_match = "exact equality after trimming ASCII space and tab from both ends"
extract = "include the unique start-marker line; exclude the unique end-marker line"
line_transform = "trim ASCII space and tab from both ends"
drop_empty_lines = true
drop_line_prefixes = ["#"]
join = "LF between retained lines and one terminal LF"
encode = "UTF-8 without BOM before SHA-256"

[normalization."text-v1"]
decode = "strict UTF-8; strip one leading UTF-8 BOM if present"
line_endings = "replace CRLF and CR with LF"
marker_match = "exact equality after trimming ASCII space and tab from both ends"
extract = "include the unique start-marker line; exclude the unique end-marker line"
line_transform = "trim ASCII space and tab from the right only"
drop_empty_lines = false
drop_line_prefixes = []
join = "LF between retained lines and one terminal LF"
encode = "UTF-8 without BOM before SHA-256"

[[signature_slice]]
order = 1
id = "root-workspace-members"
path = "Cargo.toml"
normalization = "toml-v1"
start_marker = "[workspace]"
end_marker = "[lib]"
sha256 = "$primary"
compatible_descendant_sha256 = "$alternate"
EOF
}

new_workspace_repo() {
  mkdir -p "$repo/scripts" "$repo/tools/intel"
  cp "$guard" "$repo/scripts/check_intel_baseline.sh"
  chmod +x "$repo/scripts/check_intel_baseline.sh"
  GIT_MASTER=1 git -C "$repo" init -q
  write_workspace_cargo B0
  GIT_MASTER=1 git -C "$repo" add -- Cargo.toml
  commit_fixture B0-pinned-primary
  baseline="$(GIT_MASTER=1 git -C "$repo" rev-parse HEAD)"
  write_workspace_manifest "$baseline"
}

commit_workspace_state() {
  write_workspace_cargo "$1"
  GIT_MASTER=1 git -C "$repo" add -- Cargo.toml
  commit_fixture "$1"
}

replace_manifest_line() {
  local replacement="$1"
  awk -v replacement="$replacement" '
    /^compatible_descendant_sha256 = / { print replacement; next }
    { print }
  ' "$repo/tools/intel/baseline.toml" > "$work/manifest.toml"
  mv "$work/manifest.toml" "$repo/tools/intel/baseline.toml"
}

new_repo() {
  mkdir -p "$repo/scripts" "$repo/tools/intel"
  cp "$guard" "$repo/scripts/check_intel_baseline.sh"
  chmod +x "$repo/scripts/check_intel_baseline.sh"
  printf '%s\n' outside-before BEGIN inside END outside-after > "$repo/critical.txt"
  GIT_MASTER=1 git -C "$repo" init -q
  GIT_MASTER=1 git -C "$repo" add -- critical.txt
  commit_fixture baseline
  baseline="$(GIT_MASTER=1 git -C "$repo" rev-parse HEAD)"
  write_manifest "$baseline"
}

commit_path() {
  GIT_MASTER=1 git -C "$repo" add -- "$1"
  commit_fixture descendant
}

make_docs_descendant() {
  printf '%s\n' documentation > "$repo/docs.md"
  commit_path docs.md
}

invoke_guard() {
  if bash "$repo/scripts/check_intel_baseline.sh" "$@" > "$stdout_file" 2> "$stderr_file"; then
    guard_status=0
  else
    guard_status=$?
  fi
}

invoke_guard_with_path() {
  local custom_path="$1"
  shift
  if PATH="$custom_path" bash "$repo/scripts/check_intel_baseline.sh" "$@" > "$stdout_file" 2> "$stderr_file"; then
    guard_status=0
  else
    guard_status=$?
  fi
}

assert_success() {
  if [ "$guard_status" -ne 0 ]; then
    printf 'guard status: %s\nguard stdout:\n' "$guard_status" >&3
    cat "$stdout_file" >&3
    printf 'guard stderr:\n' >&3
    cat "$stderr_file" >&3
    return 1
  fi
  [ ! -s "$stdout_file" ]
  [ ! -s "$stderr_file" ]
}

assert_rejection() {
  [ "$guard_status" -eq 2 ]
  [ ! -s "$stdout_file" ]
}

@test "the guard is executable" {
  [ -x "$guard" ]
}

@test "accepts the exact pinned revision" {
  new_repo
  invoke_guard
  assert_success
}

@test "rejects a descendant by default with both revisions" {
  new_repo
  make_docs_descendant
  current="$(GIT_MASTER=1 git -C "$repo" rev-parse HEAD)"
  invoke_guard
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: pinned revision $baseline, current revision $current; use --allow-descendant to verify a compatible descendant" ]
}

@test "accepts a compatible descendant" {
  new_repo
  make_docs_descendant
  invoke_guard --allow-descendant
  assert_success
}

@test "accepts the committed reviewed seven-member workspace descendant only" {
  new_workspace_repo
  invoke_guard
  assert_success

  commit_workspace_state B1
  invoke_guard --allow-descendant
  assert_success

  commit_workspace_state B2
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current signature root-workspace-members: SHA-256 mismatch" ]

  commit_workspace_state B3
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current signature root-workspace-members: SHA-256 mismatch" ]

  commit_workspace_state B4
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current signature root-workspace-members: SHA-256 mismatch" ]
}

@test "rejects malformed compatible descendant hashes and schema versions" {
  local alternate='1cf8ad1c3f365dc8d6815a3e674639209b2a3c79581e348c97c41e4deb8a5529'
  local primary='01682718dc81bf6ca83b17e5a4b7fd755dc44c08357e0c817c20d66e5f474a78'
  local malformed

  new_workspace_repo
  for malformed in \
    'compatible_descendant_sha256 = ""' \
    'compatible_descendant_sha256 = "1CF8AD1C3F365DC8D6815A3E674639209B2A3C79581E348C97C41E4DEB8A5529"' \
    'compatible_descendant_sha256 = "1cf8ad1c"' \
    'compatible_descendant_sha256 = "gcf8ad1c3f365dc8d6815a3e674639209b2a3c79581e348c97c41e4deb8a5529"' \
    "compatible_descendant_sha256 = [ \"$alternate\" ]" \
    "compatible_descendant_sha256 = \"$primary\""; do
    write_workspace_manifest "$baseline"
    replace_manifest_line "$malformed"
    invoke_guard
    assert_rejection
    [[ "$(cat "$stderr_file")" == check_intel_baseline:\ manifest\ invalid:* ]]
  done

  write_workspace_manifest "$baseline"
  replace_manifest_line "compatible_descendant_sha257 = \"$alternate\""
  invoke_guard
  assert_rejection
  [[ "$(cat "$stderr_file")" == check_intel_baseline:\ manifest\ invalid:* ]]

  write_workspace_manifest "$baseline"
  awk '/^compatible_descendant_sha256 = / { print; print; next } { print }' \
    "$repo/tools/intel/baseline.toml" > "$work/manifest.toml"
  mv "$work/manifest.toml" "$repo/tools/intel/baseline.toml"
  invoke_guard
  assert_rejection
  [[ "$(cat "$stderr_file")" == check_intel_baseline:\ manifest\ invalid:* ]]

  write_workspace_manifest "$baseline"
  awk -v alternate="$alternate" '
    /^compatible_descendant_sha256 = / { next }
    /^revision = / && !moved { print; print "compatible_descendant_sha256 = \"" alternate "\""; moved=1; next }
    { print }
  ' "$repo/tools/intel/baseline.toml" > "$work/manifest.toml"
  mv "$work/manifest.toml" "$repo/tools/intel/baseline.toml"
  invoke_guard
  assert_rejection
  [[ "$(cat "$stderr_file")" == check_intel_baseline:\ manifest\ invalid:* ]]

  for malformed in 1 3 '"2"' '[2]'; do
    write_workspace_manifest "$baseline"
    awk -v schema="$malformed" '/^schema_version = / { print "schema_version = " schema; next } { print }' \
      "$repo/tools/intel/baseline.toml" > "$work/manifest.toml"
    mv "$work/manifest.toml" "$repo/tools/intel/baseline.toml"
    invoke_guard
    assert_rejection
    [[ "$(cat "$stderr_file")" == check_intel_baseline:\ manifest\ invalid:* ]]
  done
}

@test "allows an unrelated out-of-slice change" {
  new_repo
  printf '%s\n' changed-outside BEGIN inside END outside-after > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_success
}

@test "reports a missing required path exactly" {
  new_repo
  rm "$repo/critical.txt"
  GIT_MASTER=1 git -C "$repo" add -u -- critical.txt
  commit_fixture missing-critical-path
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current required path missing: critical.txt" ]
}

@test "rejects semantic signature drift" {
  new_repo
  printf '%s\n' outside-before BEGIN changed END outside-after > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current signature critical: SHA-256 mismatch" ]
}

@test "rejects duplicate start markers" {
  new_repo
  printf '%s\n' outside-before BEGIN inside BEGIN END outside-after > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current signature critical: start marker matched 2 lines" ]
}

@test "rejects reversed markers" {
  new_repo
  printf '%s\n' outside-before END BEGIN inside outside-after > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current signature critical: start marker does not precede end marker" ]
}

@test "rejects a missing marker" {
  new_repo
  printf '%s\n' outside-before inside END outside-after > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current signature critical: start marker matched 0 lines" ]
}

@test "normalizes one BOM and CRLF or CR line endings" {
  new_repo
  printf '\357\273\277outside-before\r\nBEGIN \r\ninside\r\nEND\r\noutside-after\r\n' > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_success

  printf 'outside-before\rBEGIN \rinside\rEND\routside-after\r' > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_success
}

@test "rejects invalid UTF-8 in a signature source" {
  new_repo
  printf 'outside-before\nBEGIN\n\377\nEND\noutside-after\n' > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: current signature critical: invalid UTF-8" ]
}

@test "rejects malformed and duplicate manifest data" {
  new_repo
  printf '%s\n' 'not valid TOML here' >> "$repo/tools/intel/baseline.toml"
  invoke_guard
  assert_rejection
  [[ "$(cat "$stderr_file")" == check_intel_baseline:\ manifest\ invalid:* ]]

  write_manifest "$baseline"
  awk 'NR == 1 { print; print; next } { print }' "$repo/tools/intel/baseline.toml" > "$work/duplicate.toml"
  mv "$work/duplicate.toml" "$repo/tools/intel/baseline.toml"
  invoke_guard
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: manifest invalid: duplicate key: schema_version" ]
}

@test "rejects a missing baseline object" {
  new_repo
  write_manifest 0000000000000000000000000000000000000000
  invoke_guard
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: baseline object missing: 0000000000000000000000000000000000000000" ]
}

@test "rejects a non-descendant without switching branches" {
  new_repo
  empty_tree="$(GIT_MASTER=1 git -C "$repo" mktree </dev/null)"
  unrelated="$(printf '%s\n' unrelated | GIT_MASTER=1 GIT_AUTHOR_NAME=Fixture GIT_AUTHOR_EMAIL=fixture@example.test GIT_COMMITTER_NAME=Fixture GIT_COMMITTER_EMAIL=fixture@example.test git -C "$repo" commit-tree "$empty_tree")"
  branch="$(GIT_MASTER=1 git -C "$repo" symbolic-ref --short HEAD)"
  GIT_MASTER=1 git -C "$repo" update-ref "refs/heads/$branch" "$unrelated"
  invoke_guard --allow-descendant
  assert_rejection
  [[ "$(cat "$stderr_file")" == *"is not a descendant of pinned revision $baseline" ]]
}

@test "rejects Git and SHA-256 tool failures" {
  new_repo
  mkdir "$work/mock"
  printf '%s\n' '#!/usr/bin/env bash' 'exit 79' > "$work/mock/git"
  chmod +x "$work/mock/git"
  invoke_guard_with_path "$work/mock:$PATH"
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: git query failed: rev-parse --show-toplevel" ]

  rm "$work/mock/git"
  printf '%s\n' '#!/usr/bin/env bash' 'exit 79' > "$work/mock/sha256sum"
  chmod +x "$work/mock/sha256sum"
  invoke_guard_with_path "$work/mock:$PATH"
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: SHA-256 tool failed: sha256sum" ]
}

@test "orders multiple diagnostics deterministically" {
  new_repo
  signature="$(printf 'BEGIN\ninside\n' | fixture_sha256)"
  cat >> "$repo/tools/intel/baseline.toml" <<EOF

[[signature_slice]]
order = 2
id = "critical-second"
path = "critical.txt"
normalization = "text-v1"
start_marker = "BEGIN"
end_marker = "END"
sha256 = "$signature"
EOF
  printf '%s\n' outside-before BEGIN changed END outside-after > "$repo/critical.txt"
  commit_path critical.txt
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "$(printf '%s\n%s' \
    'check_intel_baseline: current signature critical: SHA-256 mismatch' \
    'check_intel_baseline: current signature critical-second: SHA-256 mismatch')" ]
  first="$(cat "$stderr_file")"
  invoke_guard --allow-descendant
  assert_rejection
  [ "$(cat "$stderr_file")" = "$first" ]
}

@test "rejects unknown arguments without echoing a long value" {
  new_repo
  long_argument="$(printf '%08000d' 0)"
  invoke_guard "$long_argument"
  assert_rejection
  [ "$(cat "$stderr_file")" = "check_intel_baseline: usage: check_intel_baseline.sh [--allow-descendant]" ]
}
