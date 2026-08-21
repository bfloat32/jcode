param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$GuardArgs
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$script:Utf8NoBom = New-Object System.Text.UTF8Encoding($false, $true)
$script:AsciiTrim = [char[]]@([char]0x20, [char]0x09)

function Stop-Guard([string]$Message) { throw $Message }

function New-OrdinalMap { return New-Object 'System.Collections.Generic.Dictionary[string,object]' ([System.StringComparer]::Ordinal) }

function ConvertTo-ProcessArgument([string]$Value) {
    if ($Value.Length -eq 0) { return '""' }
    if ($Value -cnotmatch '[\s"]') { return $Value }
    $escaped = [regex]::Replace($Value, '(\\*)"', '$1$1\\"')
    return '"' + [regex]::Replace($escaped, '(\\*)$', '$1$1') + '"'
}

function ConvertFrom-StrictUtf8([byte[]]$Bytes) {
    $offset = 0
    if ($Bytes.Length -ge 3 -and $Bytes[0] -eq 0xEF -and $Bytes[1] -eq 0xBB -and $Bytes[2] -eq 0xBF) { $offset = 3 }
    try { return $script:Utf8NoBom.GetString($Bytes, $offset, $Bytes.Length - $offset) }
    catch { Stop-Guard "strict UTF-8 decode failed: $($_.Exception.Message)" }
}

function Get-LfLines([string]$Text) {
    return @($Text.Replace("`r`n", "`n").Replace("`r", "`n").Split([string[]]@("`n"), [System.StringSplitOptions]::None))
}

function Get-TomlString([string]$Value, [string]$Where) {
    if ($Value.Length -lt 2 -or $Value[0] -ne [char]34 -or $Value[$Value.Length - 1] -ne [char]34) { Stop-Guard "malformed manifest: $Where must be a basic string" }
    $decoded = New-Object System.Text.StringBuilder
    for ($index = 1; $index -lt $Value.Length - 1; $index += 1) {
        $character = $Value[$index]
        if ($character -eq [char]92) {
            $index += 1
            if ($index -ge $Value.Length - 1 -or ($Value[$index] -ne [char]34 -and $Value[$index] -ne [char]92)) { Stop-Guard "malformed manifest: $Where has unsupported escape" }
            [void]$decoded.Append($Value[$index])
        } elseif ($character -eq [char]34) { Stop-Guard "malformed manifest: $Where has an unescaped quote" }
        else { [void]$decoded.Append($character) }
    }
    return $decoded.ToString()
}

function Set-ManifestValue($Table, [string]$Key, [string]$Value, [string]$Where) {
    if ($Table.ContainsKey($Key)) { Stop-Guard "malformed manifest: duplicate $Where.$Key" }
    $Table[$Key] = $Value
}

function Get-ManifestValue($Table, [string]$Key, [string]$Where) {
    if (-not $Table.ContainsKey($Key)) { Stop-Guard "malformed manifest: missing $Where.$Key" }
    return [string]$Table[$Key]
}

function Assert-RepositoryPath([string]$Path, [string]$Where) {
    if ($Path -notmatch '^[A-Za-z0-9][A-Za-z0-9._/-]*$' -or $Path -match '(^|/)\.\./|/\.\.$|//') {
        Stop-Guard "malformed manifest: invalid repository path in $($Where): $Path"
    }
}

function ConvertFrom-BaselineManifest([string]$ManifestPath) {
    if (-not (Test-Path -LiteralPath $ManifestPath -PathType Leaf)) { Stop-Guard 'manifest missing: tools/intel/baseline.toml' }
    $root = New-OrdinalMap; $jcode = New-OrdinalMap; $agentgrep = New-OrdinalMap; $paths = New-Object 'System.Collections.Generic.List[string]'; $pathSet = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal)
    $slices = New-Object 'System.Collections.Generic.List[object]'; $normalizations = New-OrdinalMap; $sections = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal); $section = 'root'; $slice = $null; $normalizationName = $null; $inPaths = $false; $pathsDeclared = $false
    try { $lines = Get-LfLines (ConvertFrom-StrictUtf8 ([System.IO.File]::ReadAllBytes($ManifestPath))) }
    catch { Stop-Guard "malformed manifest: $($_.Exception.Message)" }
    for ($lineNumber = 0; $lineNumber -lt $lines.Count; $lineNumber += 1) {
        $line = $lines[$lineNumber].Trim($script:AsciiTrim)
        if ($line.Length -eq 0 -or $line.StartsWith('#', [System.StringComparison]::Ordinal)) { continue }
        if ($inPaths) {
            if ($line -eq ']') { $inPaths = $false; $section = 'root'; continue }
            if ($line -cnotmatch '^"(?:[^"\\]|\\.)*",$') { Stop-Guard "malformed manifest: invalid required_paths entry at line $($lineNumber + 1)" }
            $path = Get-TomlString $line.TrimEnd(',') "required_paths line $($lineNumber + 1)"; Assert-RepositoryPath $path 'required_paths'
            if (-not $pathSet.Add($path)) { Stop-Guard "malformed manifest: duplicate required path: $path" }; [void]$paths.Add($path); continue
        }
        if ($line -cmatch '^\[\[signature_slice\]\]$') { $slice = New-OrdinalMap; [void]$slices.Add($slice); $section = 'slice'; continue }
        if ($line -cmatch '^\[jcode\]$') { if (-not $sections.Add('jcode')) { Stop-Guard 'malformed manifest: duplicate section: jcode' }; $section = 'jcode'; continue }
        if ($line -cmatch '^\[agentgrep\]$') { if (-not $sections.Add('agentgrep')) { Stop-Guard 'malformed manifest: duplicate section: agentgrep' }; $section = 'agentgrep'; continue }
        if ($line -cmatch '^\[normalization\."(rust-v1|toml-v1|text-v1)"\]$') { $normalizationName = $Matches[1]; if ($normalizations.ContainsKey($normalizationName)) { Stop-Guard "malformed manifest: duplicate normalization.$normalizationName" }; $normalizations[$normalizationName] = New-OrdinalMap; $section = 'normalization'; continue }
        if ($section -ceq 'root' -and $line -cmatch '^required_paths\s*=\s*\[$') { if ($pathsDeclared) { Stop-Guard 'malformed manifest: duplicate key: required_paths' }; $pathsDeclared = $true; $inPaths = $true; continue }
        if ($line -cnotmatch '^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+)$') { Stop-Guard "malformed manifest: invalid line $($lineNumber + 1)" }
        $key = $Matches[1]; $value = $Matches[2]
        switch ($section) {
            'root' { Set-ManifestValue $root $key $value 'root' }
            'jcode' { Set-ManifestValue $jcode $key $value 'jcode' }
            'agentgrep' { Set-ManifestValue $agentgrep $key $value 'agentgrep' }
            'slice' { Set-ManifestValue $slice $key $value "signature_slice $($slices.Count)" }
            'normalization' { Set-ManifestValue $normalizations[$normalizationName] $key $value "normalization.$normalizationName" }
            default { Stop-Guard "malformed manifest: unexpected section at line $($lineNumber + 1)" }
        }
    }
    if ($inPaths) { Stop-Guard 'malformed manifest: unterminated required_paths' }
    $rootKeys = @('schema_version', 'signature_slice_schema', 'hash_algorithm', 'slice_order', 'marker_cardinality', 'slice_bounds')
    if (@($root.Keys | Where-Object { $_ -cnotin $rootKeys }).Count -gt 0) { Stop-Guard 'malformed manifest: unsupported top-level key' }
    if ((Get-ManifestValue $root 'schema_version' 'root') -ne '1') { Stop-Guard 'malformed manifest: unsupported schema_version' }
    $expectations = [ordered]@{ signature_slice_schema = 'jcode-semantic-slices-v1'; hash_algorithm = 'sha256'; slice_order = 'ascending order field'; marker_cardinality = 'start and end must each match exactly one ASCII-trimmed line'; slice_bounds = 'start inclusive, end exclusive, with start preceding end' }
    foreach ($key in $expectations.Keys) {
        if ((Get-TomlString (Get-ManifestValue $root $key 'root') "root.$key") -cne $expectations[$key]) { Stop-Guard "malformed manifest: unsupported root.$key" }
    }
    if (@($jcode.Keys | Where-Object { $_ -cne 'revision' }).Count -gt 0) { Stop-Guard 'malformed manifest: unsupported jcode key' }
    $revision = Get-TomlString (Get-ManifestValue $jcode 'revision' 'jcode') 'jcode.revision'
    if ($revision -cnotmatch '^[0-9a-f]{40}$') { Stop-Guard 'malformed manifest: invalid jcode.revision' }
    $agent = [pscustomobject]@{
        Version = Get-TomlString (Get-ManifestValue $agentgrep 'version' 'agentgrep') 'agentgrep.version'
        Tag = Get-TomlString (Get-ManifestValue $agentgrep 'tag' 'agentgrep') 'agentgrep.tag'
        Revision = Get-TomlString (Get-ManifestValue $agentgrep 'revision' 'agentgrep') 'agentgrep.revision'
        Repository = Get-TomlString (Get-ManifestValue $agentgrep 'repository' 'agentgrep') 'agentgrep.repository'
    }
    if (@($agentgrep.Keys | Where-Object { $_ -cnotin @('version', 'tag', 'revision', 'repository') }).Count -gt 0 -or [string]::IsNullOrEmpty($agent.Version) -or [string]::IsNullOrEmpty($agent.Tag) -or [string]::IsNullOrEmpty($agent.Repository) -or $agent.Revision -cnotmatch '^[0-9a-f]{40}$') { Stop-Guard 'malformed manifest: invalid agentgrep identity' }
    if ($paths.Count -eq 0 -or $slices.Count -eq 0) { Stop-Guard 'malformed manifest: required_paths and signature_slice must be nonempty' }
    $checked = New-Object 'System.Collections.Generic.List[object]'; $orders = New-Object 'System.Collections.Generic.HashSet[long]'; $ids = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal); [long]$lastOrder = 0
    foreach ($raw in $slices) {
        $sliceKeys = @('order', 'id', 'path', 'normalization', 'start_marker', 'end_marker', 'sha256')
        if (@($raw.Keys | Where-Object { $_ -cnotin $sliceKeys }).Count -gt 0) { Stop-Guard 'malformed manifest: unsupported slice key' }
        foreach ($key in $sliceKeys) { [void](Get-ManifestValue $raw $key 'signature_slice') }
        [long]$order = 0; $rawOrder = Get-ManifestValue $raw 'order' 'signature_slice'
        if ($rawOrder -cnotmatch '^[1-9][0-9]*$' -or -not [long]::TryParse($rawOrder, [ref]$order) -or $order -le $lastOrder -or $orders.Contains($order)) { Stop-Guard 'malformed manifest: slice order is not strictly ascending' }
        $id = Get-TomlString (Get-ManifestValue $raw 'id' 'signature_slice') 'signature_slice.id'
        $path = Get-TomlString (Get-ManifestValue $raw 'path' 'signature_slice') 'signature_slice.path'
        $mode = Get-TomlString (Get-ManifestValue $raw 'normalization' 'signature_slice') 'signature_slice.normalization'
        $start = Get-TomlString (Get-ManifestValue $raw 'start_marker' 'signature_slice') 'signature_slice.start_marker'
        $end = Get-TomlString (Get-ManifestValue $raw 'end_marker' 'signature_slice') 'signature_slice.end_marker'
        $sha256 = Get-TomlString (Get-ManifestValue $raw 'sha256' 'signature_slice') 'signature_slice.sha256'
        Assert-RepositoryPath $path 'signature_slice.path'
        if ($id -cnotmatch '^[a-z0-9][a-z0-9-]*$' -or -not $ids.Add($id) -or -not $pathSet.Contains($path) -or $mode -cnotin @('rust-v1', 'toml-v1', 'text-v1') -or $start.Length -eq 0 -or $end.Length -eq 0 -or $start -match '[\r\n]' -or $end -match '[\r\n]' -or $sha256 -cnotmatch '^[0-9a-f]{64}$') { Stop-Guard 'malformed manifest: invalid signature_slice' }
        [void]$orders.Add($order); $lastOrder = $order
        [void]$checked.Add([pscustomobject]@{ Order = $order; Id = $id; Path = $path; Mode = $mode; Start = $start; End = $end; Sha256 = $sha256 })
    }
    $normalExpected = @{ 'rust-v1' = @{ decode = 'strict UTF-8; strip one leading UTF-8 BOM if present'; line_endings = 'replace CRLF and CR with LF'; marker_match = 'exact equality after trimming ASCII space and tab from both ends'; extract = 'include the unique start-marker line; exclude the unique end-marker line'; line_transform = 'trim ASCII space and tab from both ends'; drop_empty_lines = 'true'; drop_line_prefixes = '["//"]'; join = 'LF between retained lines and one terminal LF'; encode = 'UTF-8 without BOM before SHA-256' }; 'toml-v1' = @{ decode = 'strict UTF-8; strip one leading UTF-8 BOM if present'; line_endings = 'replace CRLF and CR with LF'; marker_match = 'exact equality after trimming ASCII space and tab from both ends'; extract = 'include the unique start-marker line; exclude the unique end-marker line'; line_transform = 'trim ASCII space and tab from both ends'; drop_empty_lines = 'true'; drop_line_prefixes = '["#"]'; join = 'LF between retained lines and one terminal LF'; encode = 'UTF-8 without BOM before SHA-256' }; 'text-v1' = @{ decode = 'strict UTF-8; strip one leading UTF-8 BOM if present'; line_endings = 'replace CRLF and CR with LF'; marker_match = 'exact equality after trimming ASCII space and tab from both ends'; extract = 'include the unique start-marker line; exclude the unique end-marker line'; line_transform = 'trim ASCII space and tab from the right only'; drop_empty_lines = 'false'; drop_line_prefixes = '[]'; join = 'LF between retained lines and one terminal LF'; encode = 'UTF-8 without BOM before SHA-256' } }
    $normalKeys = @('decode', 'line_endings', 'marker_match', 'extract', 'line_transform', 'drop_empty_lines', 'drop_line_prefixes', 'join', 'encode')
    foreach ($mode in @('rust-v1', 'toml-v1', 'text-v1')) {
        if (-not $normalizations.ContainsKey($mode)) { Stop-Guard "malformed manifest: missing normalization.$mode" }
        foreach ($key in $normalizations[$mode].Keys) { if ($key -cnotin $normalKeys) { Stop-Guard "malformed manifest: invalid normalization key: $key" } }
        foreach ($key in $normalKeys) { $rawValue = Get-ManifestValue $normalizations[$mode] $key "normalization.$mode"; $actual = if ($key -cin @('drop_empty_lines', 'drop_line_prefixes')) { $rawValue } else { Get-TomlString $rawValue "normalization.$mode.$key" }; if ($actual -cne $normalExpected[$mode][$key]) { Stop-Guard "malformed manifest: invalid normalization value: $mode.$key" } }
    }
    return [pscustomobject]@{ Revision = $revision; AgentGrep = $agent; Paths = @($paths); Slices = @($checked | Sort-Object Order) }
}

function Invoke-GuardGit([string]$Root, [string[]]$Arguments) {
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = 'git'; $info.Arguments = (($Arguments | ForEach-Object { ConvertTo-ProcessArgument $_ }) -join ' '); $info.WorkingDirectory = $Root
    $info.UseShellExecute = $false; $info.RedirectStandardOutput = $true; $info.RedirectStandardError = $true; $info.CreateNoWindow = $true
    $info.EnvironmentVariables['GIT_MASTER'] = '1'
    $process = New-Object System.Diagnostics.Process; $process.StartInfo = $info; $stream = New-Object System.IO.MemoryStream
    try {
        [void]$process.Start(); $errorTask = $process.StandardError.ReadToEndAsync(); $outputTask = $process.StandardOutput.BaseStream.CopyToAsync($stream); [System.Threading.Tasks.Task]::WaitAll([System.Threading.Tasks.Task[]]@($errorTask, $outputTask)); $process.WaitForExit(); $errorText = $errorTask.GetAwaiter().GetResult()
        return [pscustomobject]@{ ExitCode = $process.ExitCode; Output = $stream.ToArray(); Error = $errorText.Trim() }
    } catch { Stop-Guard "missing tool: git ($($_.Exception.Message))" }
    finally { $stream.Dispose(); $process.Dispose() }
}

function Get-SliceResult([byte[]]$Bytes, $Slice, [string]$Label) {
    try {
        $lines = Get-LfLines (ConvertFrom-StrictUtf8 $Bytes); $starts = New-Object 'System.Collections.Generic.List[int]'; $ends = New-Object 'System.Collections.Generic.List[int]'
        for ($index = 0; $index -lt $lines.Count; $index += 1) {
            $candidate = $lines[$index].Trim($script:AsciiTrim)
            if ($candidate -ceq $Slice.Start) { [void]$starts.Add($index) }
            if ($candidate -ceq $Slice.End) { [void]$ends.Add($index) }
        }
        if ($starts.Count -ne 1) { return [pscustomobject]@{ Error = "$Label signature $($Slice.Id): start marker matched $($starts.Count) lines" } }
        if ($ends.Count -ne 1) { return [pscustomobject]@{ Error = "$Label signature $($Slice.Id): end marker matched $($ends.Count) lines" } }
        if ($starts[0] -ge $ends[0]) { return [pscustomobject]@{ Error = "$Label signature $($Slice.Id): start marker does not precede end marker" } }
        $retained = New-Object 'System.Collections.Generic.List[string]'; $prefix = if ($Slice.Mode -eq 'rust-v1') { '//' } elseif ($Slice.Mode -eq 'toml-v1') { '#' } else { $null }
        for ($index = $starts[0]; $index -lt $ends[0]; $index += 1) {
            $line = if ($Slice.Mode -eq 'text-v1') { $lines[$index].TrimEnd($script:AsciiTrim) } else { $lines[$index].Trim($script:AsciiTrim) }
            if ($Slice.Mode -ne 'text-v1' -and $line.Length -eq 0) { continue }
            if ($prefix -and $line.StartsWith($prefix, [System.StringComparison]::Ordinal)) { continue }
            [void]$retained.Add($line)
        }
        $normalizedText = ''
        if ($retained.Count -gt 0) { $normalizedText = ($retained -join "`n") + "`n" }
        $normalized = $script:Utf8NoBom.GetBytes($normalizedText); $stream = New-Object System.IO.MemoryStream
        try { $stream.Write($normalized, 0, $normalized.Length); $stream.Position = 0; return [pscustomobject]@{ Error = $null; Hash = (Get-FileHash -InputStream $stream -Algorithm SHA256).Hash.ToLowerInvariant() } }
        finally { $stream.Dispose() }
    } catch { if ($_.Exception.Message -like 'strict UTF-8*') { return [pscustomobject]@{ Error = "$Label signature $($Slice.Id): invalid UTF-8" } }; return [pscustomobject]@{ Error = "$Label signature $($Slice.Id): normalization failed" } }
}

try {
    [string[]]$guardArguments = @()
    if ($null -ne $GuardArgs) { $guardArguments = [string[]]$GuardArgs }
    $allowDescendant = $guardArguments.Count -eq 1 -and $guardArguments[0] -ceq '--allow-descendant'
    if ($guardArguments.Count -gt 1 -or ($guardArguments.Count -eq 1 -and -not $allowDescendant)) { Stop-Guard 'usage: check_intel_baseline.ps1 [--allow-descendant]' }
    if (-not (Get-Command git -CommandType Application -ErrorAction SilentlyContinue)) { Stop-Guard 'git is unavailable' }
    $repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
    $rootResult = Invoke-GuardGit $repoRoot @('rev-parse', '--show-toplevel')
    if ($rootResult.ExitCode -ne 0) { Stop-Guard 'git query failed: rev-parse --show-toplevel' }
    try { $manifest = ConvertFrom-BaselineManifest (Join-Path $repoRoot 'tools/intel/baseline.toml') }
    catch { if ($_.Exception.Message -like 'manifest missing:*') { Stop-Guard $_.Exception.Message }; Stop-Guard "manifest invalid: $($_.Exception.Message -replace '^malformed manifest: ', '')" }
    $baseResult = Invoke-GuardGit $repoRoot @('cat-file', '-e', "$($manifest.Revision)^{commit}")
    if ($baseResult.ExitCode -ne 0) { Stop-Guard "baseline object missing: $($manifest.Revision)" }
    $headResult = Invoke-GuardGit $repoRoot @('rev-parse', '--verify', 'HEAD^{commit}')
    if ($headResult.ExitCode -ne 0) { Stop-Guard 'git query failed: rev-parse HEAD' }
    $head = (ConvertFrom-StrictUtf8 $headResult.Output).Trim()
    $errors = New-Object 'System.Collections.Generic.List[string]'
    if ($head -cne $manifest.Revision) {
        if (-not $allowDescendant) { Stop-Guard "pinned revision $($manifest.Revision), current revision $head; use --allow-descendant to verify a compatible descendant" }
        else {
            $ancestry = Invoke-GuardGit $repoRoot @('merge-base', '--is-ancestor', $manifest.Revision, $head)
            if ($ancestry.ExitCode -eq 1) { Stop-Guard "current revision $head is not a descendant of pinned revision $($manifest.Revision)" }
            elseif ($ancestry.ExitCode -ne 0) { Stop-Guard 'git query failed: merge-base --is-ancestor' }
        }
    }
    foreach ($target in @([pscustomobject]@{ Label = 'pinned'; Revision = $manifest.Revision }, [pscustomobject]@{ Label = 'current'; Revision = $head })) {
        if ($target.Label -eq 'current' -and $head -ceq $manifest.Revision) { continue }
        $contents = New-OrdinalMap; $missing = New-OrdinalMap
        foreach ($path in $manifest.Paths) {
            $exists = Invoke-GuardGit $repoRoot @('cat-file', '-e', "$($target.Revision)`:$path")
            if ($exists.ExitCode -ne 0) { $missing[$path] = $true; [void]$errors.Add("$($target.Label) required path missing: $path") }
        }
        foreach ($slice in $manifest.Slices) {
            if ($missing.ContainsKey($slice.Path)) { continue }
            if (-not $contents.ContainsKey($slice.Path)) {
                $blob = Invoke-GuardGit $repoRoot @('show', "$($target.Revision)`:$($slice.Path)")
                if ($blob.ExitCode -ne 0) { $missing[$slice.Path] = $true; [void]$errors.Add("git query failed: read $($target.Label) path $($slice.Path)"); continue }
                $contents[$slice.Path] = $blob.Output
            }
            $result = Get-SliceResult $contents[$slice.Path] $slice $target.Label
            if ($result.Error) { [void]$errors.Add($result.Error) }
            elseif ($result.Hash -cne $slice.Sha256) { [void]$errors.Add("$($target.Label) signature $($slice.Id): SHA-256 mismatch") }
        }
    }
    if ($errors.Count -gt 0) { foreach ($diagnostic in $errors) { [Console]::Error.WriteLine("check_intel_baseline: $diagnostic") }; exit 2 }
    exit 0
} catch {
    [Console]::Error.WriteLine("check_intel_baseline: $($_.Exception.Message)")
    exit 2
}
