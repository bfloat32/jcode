#!/usr/bin/env bash
set -o pipefail
export LC_ALL=C

tmpdir="" normal_index=0
errors=()
seen=()
required_paths=()
slice_ids=() slice_paths=() slice_modes=() slice_starts=() slice_ends=() slice_hashes=() slice_alternates=()

cleanup() { [ -z "$tmpdir" ] || rm -rf "$tmpdir"; }
die() { printf 'check_intel_baseline: %s\n' "$1" >&2; exit 2; }
add_error() { errors[${#errors[@]}]="$1"; }
usage() { die 'usage: check_intel_baseline.sh [--allow-descendant]'; }
trap cleanup EXIT
trap 'exit 2' HUP INT TERM

trim() {
  local value="$1"
  value="${value#"${value%%[!$' \t']*}"}"
  value="${value%"${value##*[!$' \t']}"}"
  printf '%s' "$value"
}

rtrim() {
  local value="$1"
  value="${value%"${value##*[!$' \t']}"}"
  printf '%s' "$value"
}

contains() {
  local wanted="$1" value
  shift
  for value in "$@"; do [ "$value" = "$wanted" ] && return 0; done
  return 1
}

mark_seen() {
  contains "$1" "${seen[@]}" && return 1
  seen[${#seen[@]}]="$1"
}

valid_path() {
  [[ "$1" =~ ^[A-Za-z0-9][A-Za-z0-9._/-]*$ ]] && [[ "$1" != *//* ]] &&
    [[ "$1" != ../* ]] && [[ "$1" != */../* ]] && [[ "$1" != */.. ]]
}

toml_string() {
  local value="$1" character escaped=0
  decoded=""
  [ "${value:0:1}" = '"' ] && [ "${value: -1}" = '"' ] || return 1
  value="${value:1:${#value}-2}"
  while [ -n "$value" ]; do
    character="${value:0:1}"; value="${value:1}"
    if [ "$escaped" -eq 1 ]; then
      { [ "$character" = '"' ] || [ "$character" = '\' ]; } || return 1
      decoded+="$character"; escaped=0
    elif [ "$character" = '\' ]; then
      escaped=1
    elif [ "$character" = '"' ]; then
      return 1
    else
      decoded+="$character"
    fi
  done
  [ "$escaped" -eq 0 ]
}

normalize_utf8() {
  local input="$1" output="$2" utf8 payload character pending=0 bytes
  normal_index=$((normal_index + 1)); utf8="$tmpdir/utf8.$normal_index"; payload="$utf8"
  command -v iconv >/dev/null 2>&1 || { normal_error='iconv is unavailable'; return 1; }
  iconv -f UTF-8 -t UTF-8 "$input" > "$utf8" 2>/dev/null || { normal_error='invalid UTF-8'; return 1; }
  bytes="$(od -An -tx1 -N3 "$utf8" 2>/dev/null | tr -d '[:space:]')" || { normal_error='cannot inspect UTF-8 input'; return 1; }
  if [ "$bytes" = efbbbf ]; then
    payload="$tmpdir/payload.$normal_index"
    dd if="$utf8" of="$payload" bs=3 skip=1 2>/dev/null || { normal_error='cannot remove UTF-8 BOM'; return 1; }
  fi
  : > "$output" || { normal_error='cannot create normalized input'; return 1; }
  while IFS= read -r -N 1 character || [ -n "$character" ]; do
    if [ "$pending" -eq 1 ]; then
      if [ "$character" = $'\n' ]; then printf '\n' >> "$output"; pending=0; continue; fi
      printf '\n' >> "$output"; pending=0
    fi
    if [ "$character" = $'\r' ]; then pending=1; else printf '%s' "$character" >> "$output"; fi
  done < "$payload"
  [ "$pending" -eq 0 ] || printf '\n' >> "$output"
}

normalization_value() {
  case "$2" in
    decode) expected='strict UTF-8; strip one leading UTF-8 BOM if present' ;;
    line_endings) expected='replace CRLF and CR with LF' ;;
    marker_match) expected='exact equality after trimming ASCII space and tab from both ends' ;;
    extract) expected='include the unique start-marker line; exclude the unique end-marker line' ;;
    join) expected='LF between retained lines and one terminal LF' ;;
    encode) expected='UTF-8 without BOM before SHA-256' ;;
    line_transform) [ "$1" = text-v1 ] && expected='trim ASCII space and tab from the right only' || expected='trim ASCII space and tab from both ends' ;;
    drop_empty_lines) [ "$1" = text-v1 ] && expected=false || expected=true ;;
    drop_line_prefixes) case "$1" in rust-v1) expected='["//"]' ;; toml-v1) expected='["#"]' ;; text-v1) expected='[]' ;; esac ;;
    *) return 1 ;;
  esac
}

invalid_manifest() { parse_error="$1"; return 1; }

finalize_slice() {
  local key
  [ "$slice_active" -eq 1 ] || return 0
  for key in order id path normalization start_marker end_marker sha256; do
    contains "slice:$slice_index:$key" "${seen[@]}" || { invalid_manifest "missing slice key: $key"; return 1; }
  done
  [ "$current_order" -gt "$last_order" ] || { invalid_manifest 'slice order is not strictly ascending'; return 1; }
  contains "$current_id" "${slice_ids[@]}" && { invalid_manifest "duplicate slice id: $current_id"; return 1; }
  valid_path "$current_path" && contains "$current_path" "${required_paths[@]}" || { invalid_manifest "slice path is not required: $current_path"; return 1; }
  case "$current_mode" in rust-v1|toml-v1|text-v1) ;; *) invalid_manifest "unknown normalization: $current_mode"; return 1 ;; esac
  [[ "$current_hash" =~ ^[0-9a-f]{64}$ ]] || { invalid_manifest "invalid slice hash: $current_id"; return 1; }
  ! contains "slice:$slice_index:compatible_descendant_sha256" "${seen[@]}" || { [[ "$current_alternate" =~ ^[0-9a-f]{64}$ ]] && [ "$current_alternate" != "$current_hash" ]; } || { invalid_manifest "invalid compatible descendant hash: $current_id"; return 1; }
  slice_ids[${#slice_ids[@]}]="$current_id"; slice_paths[${#slice_paths[@]}]="$current_path"
  slice_modes[${#slice_modes[@]}]="$current_mode"; slice_starts[${#slice_starts[@]}]="$current_start"
  slice_ends[${#slice_ends[@]}]="$current_end"; slice_hashes[${#slice_hashes[@]}]="$current_hash"; slice_alternates[${#slice_alternates[@]}]="$current_alternate"
  last_order="$current_order"; slice_active=0
}

parse_manifest() {
  local manifest="$1" parsed="$tmpdir/manifest" line key raw section=top in_paths=0 line_number=0
  local norm_key required_key agent_key current_seen
  normalize_utf8 "$manifest" "$parsed" || { parse_error="$normal_error"; return 1; }
  slice_active=0; slice_index=0; last_order=0
  while IFS= read -r line || [ -n "$line" ]; do
    line_number=$((line_number + 1)); line="$(trim "$line")"
    [ -z "$line" ] || [ "${line:0:1}" = '#' ] && continue
    if [ "$in_paths" -eq 1 ]; then
      if [ "$line" = ']' ]; then in_paths=0; continue; fi
      [[ "$line" == *, ]] || { invalid_manifest "line $line_number: invalid required path"; return 1; }
      toml_string "$(trim "${line%,}")" || { invalid_manifest "line $line_number: invalid required path"; return 1; }
      valid_path "$decoded" && ! contains "$decoded" "${required_paths[@]}" || { invalid_manifest "duplicate or invalid required path: $decoded"; return 1; }
      required_paths[${#required_paths[@]}]="$decoded"; continue
    fi
    if [ "$line" = '[[signature_slice]]' ]; then
      finalize_slice || return 1; slice_index=$((slice_index + 1)); slice_active=1; section=slice
      current_order=0; current_id=; current_path=; current_mode=; current_start=; current_end=; current_hash=; current_alternate=; continue
    fi
    if [[ "$line" == \[*\] ]]; then
      finalize_slice || return 1
      case "$line" in
        '[jcode]'|'[agentgrep]') section="${line:1:${#line}-2}" ;;
        '[normalization."rust-v1"]') section=rust-v1 ;;
        '[normalization."toml-v1"]') section=toml-v1 ;;
        '[normalization."text-v1"]') section=text-v1 ;;
        *) invalid_manifest "line $line_number: unsupported section"; return 1 ;;
      esac
      mark_seen "section:$section" || { invalid_manifest "duplicate section: $section"; return 1; }; continue
    fi
    [[ "$line" == *=* ]] || { invalid_manifest "line $line_number: expected assignment"; return 1; }
    key="$(trim "${line%%=*}")"; raw="$(trim "${line#*=}")"
    case "$section" in
      top)
        mark_seen "top:$key" || { invalid_manifest "duplicate key: $key"; return 1; }
        case "$key" in
          schema_version) [ "$raw" = 2 ] || { invalid_manifest 'unsupported schema version'; return 1; } ;;
          signature_slice_schema|hash_algorithm|slice_order|marker_cardinality|slice_bounds)
            toml_string "$raw" || { invalid_manifest "invalid value: $key"; return 1; }
            case "$key:$decoded" in signature_slice_schema:jcode-semantic-slices-v2|hash_algorithm:sha256|slice_order:'ascending order field'|marker_cardinality:'start and end must each match exactly one ASCII-trimmed line'|slice_bounds:'start inclusive, end exclusive, with start preceding end') ;; *) invalid_manifest "unsupported value: $key"; return 1 ;; esac ;;
          required_paths) [ "$raw" = '[' ] || { invalid_manifest 'invalid required_paths'; return 1; }; in_paths=1 ;;
          *) invalid_manifest "unsupported top-level key: $key"; return 1 ;;
        esac ;;
      jcode)
        [ "$key" = revision ] && mark_seen 'jcode:revision' && toml_string "$raw" && [[ "$decoded" =~ ^[0-9a-f]{40}$ ]] || { invalid_manifest 'invalid jcode revision'; return 1; }; pinned="$decoded" ;;
      agentgrep)
        case "$key" in version|tag|repository|revision) ;; *) invalid_manifest "unsupported agentgrep key: $key"; return 1 ;; esac
        mark_seen "agentgrep:$key" && toml_string "$raw" && [ -n "$decoded" ] || { invalid_manifest "invalid agentgrep key: $key"; return 1; }
        [ "$key" != revision ] || [[ "$decoded" =~ ^[0-9a-f]{40}$ ]] || { invalid_manifest 'invalid agentgrep revision'; return 1; } ;;
      rust-v1|toml-v1|text-v1)
        mark_seen "norm:$section:$key" && normalization_value "$section" "$key" || { invalid_manifest "invalid normalization key: $key"; return 1; }
        case "$key" in drop_empty_lines|drop_line_prefixes) [ "$raw" = "$expected" ] || { invalid_manifest "invalid normalization value: $key"; return 1; } ;; *) toml_string "$raw" && [ "$decoded" = "$expected" ] || { invalid_manifest "invalid normalization value: $key"; return 1; } ;; esac ;;
      slice)
        mark_seen "slice:$slice_index:$key" || { invalid_manifest "duplicate slice key: $key"; return 1; }
        case "$key" in
          order) [[ "$raw" =~ ^[1-9][0-9]*$ ]] || { invalid_manifest 'invalid slice order'; return 1; }; current_order="$raw" ;;
          id|path|normalization|start_marker|end_marker|sha256|compatible_descendant_sha256) toml_string "$raw" || { invalid_manifest "invalid slice key: $key"; return 1; }; case "$key" in id) [[ "$decoded" =~ ^[a-z0-9][a-z0-9-]*$ ]] || { invalid_manifest 'invalid slice id'; return 1; }; current_id="$decoded" ;; path) current_path="$decoded" ;; normalization) current_mode="$decoded" ;; start_marker) [ -n "$decoded" ] || { invalid_manifest 'empty start marker'; return 1; }; current_start="$decoded" ;; end_marker) [ -n "$decoded" ] || { invalid_manifest 'empty end marker'; return 1; }; current_end="$decoded" ;; sha256) current_hash="$decoded" ;; compatible_descendant_sha256) current_alternate="$decoded" ;; esac ;;
          *) invalid_manifest "unsupported slice key: $key"; return 1 ;;
        esac ;;
      *) invalid_manifest "line $line_number: assignment outside a section"; return 1 ;;
    esac
  done < "$parsed"
  [ "$in_paths" -eq 0 ] || { invalid_manifest 'unterminated required_paths'; return 1; }
  finalize_slice || return 1
  for required_key in schema_version signature_slice_schema hash_algorithm slice_order marker_cardinality slice_bounds required_paths; do contains "top:$required_key" "${seen[@]}" || { invalid_manifest "missing key: $required_key"; return 1; }; done
  for agent_key in version tag revision repository; do contains "agentgrep:$agent_key" "${seen[@]}" || { invalid_manifest "missing agentgrep key: $agent_key"; return 1; }; done
  for norm_key in rust-v1 toml-v1 text-v1; do for required_key in decode line_endings marker_match extract line_transform drop_empty_lines drop_line_prefixes join encode; do contains "norm:$norm_key:$required_key" "${seen[@]}" || { invalid_manifest "missing normalization key: $norm_key.$required_key"; return 1; }; done; done
  contains 'jcode:revision' "${seen[@]}" && [ "${#required_paths[@]}" -gt 0 ] && [ "${#slice_ids[@]}" -gt 0 ] || { invalid_manifest 'missing required manifest data'; return 1; }
}

sha256_file() {
  local file="$1" result
  if command -v sha256sum >/dev/null 2>&1; then result="$(sha256sum "$file" 2>/dev/null)" || { hash_error=sha256sum; return 1; }
  elif command -v shasum >/dev/null 2>&1; then result="$(shasum -a 256 "$file" 2>/dev/null)" || { hash_error=shasum; return 1; }
  elif command -v openssl >/dev/null 2>&1; then result="$(openssl dgst -sha256 "$file" 2>/dev/null | awk '{print tolower($NF)}')" || { hash_error=openssl; return 1; }
  else hash_error='no SHA-256 tool available'; return 1; fi
  digest="${result%%[[:space:]]*}"; digest="$(printf '%s' "$digest" | tr '[:upper:]' '[:lower:]')"
  [[ "$digest" =~ ^[0-9a-f]{64}$ ]] || { hash_error='invalid SHA-256 output'; return 1; }
}

check_slice() {
  local label="$1" id="$2" mode="$3" start="$4" end="$5" expected_hash="$6" alternate_hash="$7" source="$8" output="$9"
  local line number=0 start_count=0 end_count=0 start_line=0 end_line=0 valid=1 transformed
  while IFS= read -r line || [ -n "$line" ]; do
    number=$((number + 1)); line="$(trim "$line")"
    [ "$line" != "$start" ] || { start_count=$((start_count + 1)); start_line="$number"; }
    [ "$line" != "$end" ] || { end_count=$((end_count + 1)); end_line="$number"; }
  done < "$source"
  [ "$start_count" -eq 1 ] || { add_error "$label signature $id: start marker matched $start_count lines"; valid=0; }
  [ "$end_count" -eq 1 ] || { add_error "$label signature $id: end marker matched $end_count lines"; valid=0; }
  [ "$valid" -eq 0 ] || [ "$start_line" -lt "$end_line" ] || { add_error "$label signature $id: start marker does not precede end marker"; valid=0; }
  [ "$valid" -eq 1 ] || return
  : > "$output" || die 'cannot create signature input'
  number=0
  while IFS= read -r line || [ -n "$line" ]; do
    number=$((number + 1)); [ "$number" -ge "$start_line" ] && [ "$number" -lt "$end_line" ] || continue
    [ "$mode" = text-v1 ] && transformed="$(rtrim "$line")" || transformed="$(trim "$line")"
    [ "$mode" = text-v1 ] || [ -n "$transformed" ] || continue
    [[ "$mode" != rust-v1 || "$transformed" != //* ]] || continue
    [[ "$mode" != toml-v1 || "$transformed" != \#* ]] || continue
    printf '%s\n' "$transformed" >> "$output"
  done < "$source"
  sha256_file "$output" || die "SHA-256 tool failed: $hash_error"
  [ "$digest" = "$expected_hash" ] || { [ "$label" = current ] && [ -n "$alternate_hash" ] && [ "$digest" = "$alternate_hash" ]; } || add_error "$label signature $id: SHA-256 mismatch"
}

check_revision() {
  local label="$1" revision="$2" index path raw normalized output
  local missing_paths=()
  for path in "${required_paths[@]}"; do
    GIT_MASTER=1 git -C "$repo_root" cat-file -e "$revision:$path" 2>/dev/null || { add_error "$label required path missing: $path"; missing_paths[${#missing_paths[@]}]="$path"; }
  done
  for ((index = 0; index < ${#slice_ids[@]}; index++)); do
    path="${slice_paths[index]}"; contains "$path" "${missing_paths[@]}" && continue
    raw="$tmpdir/$label.raw.$index"; normalized="$tmpdir/$label.normalized.$index"; output="$tmpdir/$label.signature.$index"
    GIT_MASTER=1 git -C "$repo_root" show "$revision:$path" > "$raw" 2>/dev/null || { add_error "git query failed: read $label path $path"; continue; }
    normalize_utf8 "$raw" "$normalized" || { [ "$normal_error" = 'invalid UTF-8' ] && add_error "$label signature ${slice_ids[index]}: invalid UTF-8" || add_error "$label signature ${slice_ids[index]}: normalization failed"; continue; }
    check_slice "$label" "${slice_ids[index]}" "${slice_modes[index]}" "${slice_starts[index]}" "${slice_ends[index]}" "${slice_hashes[index]}" "${slice_alternates[index]}" "$normalized" "$output"
  done
}

allow_descendant=0
case "$#" in 0) ;; 1) [ "$1" = --allow-descendant ] && allow_descendant=1 || usage ;; *) usage ;; esac
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)" || die 'cannot resolve script directory'
repo_root="$(cd "$script_dir/.." && pwd -P)" || die 'cannot resolve repository directory'
command -v git >/dev/null 2>&1 || die 'git is unavailable'
GIT_MASTER=1 git -C "$repo_root" rev-parse --show-toplevel >/dev/null 2>&1 || die 'git query failed: rev-parse --show-toplevel'
tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/jcode-intel-baseline.XXXXXX")" || die 'cannot create temporary directory'
manifest="$repo_root/tools/intel/baseline.toml"
[ -f "$manifest" ] || die 'manifest missing: tools/intel/baseline.toml'
parse_manifest "$manifest" || die "manifest invalid: $parse_error"
GIT_MASTER=1 git -C "$repo_root" cat-file -e "$pinned^{commit}" 2>/dev/null || die "baseline object missing: $pinned"
current="$(GIT_MASTER=1 git -C "$repo_root" rev-parse --verify 'HEAD^{commit}' 2>/dev/null)" || die 'git query failed: rev-parse HEAD'
if [ "$current" != "$pinned" ]; then
  [ "$allow_descendant" -eq 1 ] || die "pinned revision $pinned, current revision $current; use --allow-descendant to verify a compatible descendant"
  GIT_MASTER=1 git -C "$repo_root" merge-base --is-ancestor "$pinned" "$current" 2>/dev/null
  merge_status=$?
  [ "$merge_status" -ne 1 ] || die "current revision $current is not a descendant of pinned revision $pinned"
  [ "$merge_status" -eq 0 ] || die 'git query failed: merge-base --is-ancestor'
fi
check_revision pinned "$pinned"
[ "$current" = "$pinned" ] || check_revision current "$current"
[ "${#errors[@]}" -eq 0 ] || { for error in "${errors[@]}"; do printf 'check_intel_baseline: %s\n' "$error" >&2; done; exit 2; }
