param([string]$Filter)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$script:Utf8NoBom = New-Object System.Text.UTF8Encoding($false, $true)
$script:TempRoots = New-Object 'System.Collections.Generic.List[string]'
$script:Failures = New-Object 'System.Collections.Generic.List[string]'
$script:Passed = 0
$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$script:GuardSource = Join-Path $repositoryRoot 'scripts/check_intel_baseline.ps1'

function Assert-Equal {
    param([string]$Label, $Expected, $Actual)

    if ($Expected -ne $Actual) {
        throw "$Label expected '$Expected' but got '$Actual'"
    }
}

function Assert-Contains {
    param([string]$Label, [string]$Text, [string]$Needle)

    if ($Text.IndexOf($Needle, [System.StringComparison]::Ordinal) -lt 0) {
        throw "$Label did not contain '$Needle': $Text"
    }
}

function Assert-Empty {
    param([string]$Label, [string]$Text)

    if ($Text.Length -ne 0) {
        throw "$Label expected empty output but got: $Text"
    }
}

function ConvertFrom-StrictUtf8 {
    param([byte[]]$Bytes)

    $offset = 0
    if ($Bytes.Length -ge 3 -and $Bytes[0] -eq 0xEF -and $Bytes[1] -eq 0xBB -and $Bytes[2] -eq 0xBF) {
        $offset = 3
    }
    return $script:Utf8NoBom.GetString($Bytes, $offset, $Bytes.Length - $offset)
}

function ConvertTo-Utf8Bytes {
    param([string]$Text, [switch]$WithBom)

    $body = $script:Utf8NoBom.GetBytes($Text)
    if (-not $WithBom) {
        return ,$body
    }

    $bytes = New-Object byte[] ($body.Length + 3)
    $bytes[0] = 0xEF
    $bytes[1] = 0xBB
    $bytes[2] = 0xBF
    [System.Array]::Copy($body, 0, $bytes, 3, $body.Length)
    return ,$bytes
}

function Get-AsciiTrimmed {
    param([string]$Line, [switch]$RightOnly)

    $characters = [char[]]@([char]0x20, [char]0x09)
    if ($RightOnly) {
        return $Line.TrimEnd($characters)
    }
    return $Line.Trim($characters)
}

function Get-NormalizedSliceHash {
    param(
        [byte[]]$Bytes,
        [ValidateSet('rust-v1', 'toml-v1', 'text-v1')][string]$Mode,
        [string]$StartMarker,
        [string]$EndMarker
    )

    $text = ConvertFrom-StrictUtf8 -Bytes $Bytes
    $lines = @($text.Replace("`r`n", "`n").Replace("`r", "`n").Split([string[]]@("`n"), [System.StringSplitOptions]::None))
    $start = @()
    $end = @()
    for ($index = 0; $index -lt $lines.Count; $index += 1) {
        $candidate = Get-AsciiTrimmed -Line $lines[$index]
        if ($candidate -ceq $StartMarker) { $start += $index }
        if ($candidate -ceq $EndMarker) { $end += $index }
    }
    if ($start.Count -ne 1 -or $end.Count -ne 1 -or $start[0] -ge $end[0]) {
        throw 'fixture slice markers are invalid'
    }

    $retained = New-Object 'System.Collections.Generic.List[string]'
    $dropEmpty = $Mode -ne 'text-v1'
    $prefix = if ($Mode -eq 'rust-v1') { '//' } elseif ($Mode -eq 'toml-v1') { '#' } else { $null }
    for ($index = $start[0]; $index -lt $end[0]; $index += 1) {
        $line = Get-AsciiTrimmed -Line $lines[$index] -RightOnly:($Mode -eq 'text-v1')
        if ($dropEmpty -and $line.Length -eq 0) { continue }
        if ($prefix -and $line.StartsWith($prefix, [System.StringComparison]::Ordinal)) { continue }
        [void]$retained.Add($line)
    }

    $normalized = if ($retained.Count -eq 0) { '' } else { ($retained -join "`n") + "`n" }
    $stream = New-Object System.IO.MemoryStream
    try {
        $normalizedBytes = $script:Utf8NoBom.GetBytes($normalized)
        $stream.Write($normalizedBytes, 0, $normalizedBytes.Length)
        $stream.Position = 0
        return (Get-FileHash -InputStream $stream -Algorithm SHA256).Hash.ToLowerInvariant()
    } finally {
        $stream.Dispose()
    }
}

function Write-FixtureBytes {
    param([string]$Root, [string]$RelativePath, [byte[]]$Bytes)

    $path = Join-Path $Root ($RelativePath -replace '/', [System.IO.Path]::DirectorySeparatorChar)
    $directory = Split-Path -Parent $path
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    [System.IO.File]::WriteAllBytes($path, $Bytes)
    return $path
}

function Invoke-TestGit {
    param([string]$Root, [string[]]$Arguments)

    $previous = [Environment]::GetEnvironmentVariable('GIT_MASTER', [EnvironmentVariableTarget]::Process)
    try {
        [Environment]::SetEnvironmentVariable('GIT_MASTER', '1', [EnvironmentVariableTarget]::Process)
        $output = & git -C $Root @Arguments 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        [Environment]::SetEnvironmentVariable('GIT_MASTER', $previous, [EnvironmentVariableTarget]::Process)
    }
    $text = (@($output) -join "`n").TrimEnd("`r", "`n")
    if ($exitCode -ne 0) {
        throw "git $($Arguments -join ' ') failed: $text"
    }
    return $text
}

function Get-CommandLineArgument {
    param([string]$Value)

    if ($Value.Length -eq 0) { return '""' }
    if ($Value -notmatch '[\s"]') { return $Value }
    return '"' + $Value.Replace('"', '\"') + '"'
}

function Normalize-Output([string]$Text) { return $Text.Replace("`r`n", "`n").Replace("`r", "`n") }

function Invoke-Guard {
    param([pscustomobject]$Fixture, [string[]]$Arguments = @(), [switch]$WithoutGit)

    $enginePath = (Get-Process -Id $PID).Path
    if (-not $enginePath) {
        $isWindows = [Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT
        $engineName = if ($PSVersionTable.PSEdition -eq 'Core') {
            if ($isWindows) { 'pwsh.exe' } else { 'pwsh' }
        } else {
            'powershell.exe'
        }
        $enginePath = Join-Path $PSHOME $engineName
    }

    $startInfo = New-Object System.Diagnostics.ProcessStartInfo
    $startInfo.FileName = $enginePath
    $startInfo.Arguments = (@('-NoProfile', '-File', $Fixture.Guard) + $Arguments | ForEach-Object { Get-CommandLineArgument -Value $_ }) -join ' '
    $startInfo.WorkingDirectory = $Fixture.Root
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.CreateNoWindow = $true
    if ($WithoutGit) { $startInfo.EnvironmentVariables['PATH'] = '' }

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $startInfo
    try {
        [void]$process.Start()
        $stdout = $process.StandardOutput.ReadToEnd()
        $stderr = $process.StandardError.ReadToEnd()
        $process.WaitForExit()
        return [pscustomobject]@{
            ExitCode = $process.ExitCode
            Stdout = Normalize-Output $stdout
            Stderr = Normalize-Output $stderr
        }
    } finally {
        $process.Dispose()
    }
}

function Write-BaselineManifest {
    param(
        [pscustomobject]$Fixture,
        [string]$Revision = $Fixture.BaselineRevision,
        [string]$MissingPath,
        [string]$CompatibleDescendantSha256
    )

    $paths = @('    "Cargo.toml",', '    "Cargo.lock",', '    "src/example.rs",', '    "notes.txt",')
    if ($MissingPath) { $paths += "    `"$MissingPath`"," }
    $pathRows = $paths -join "`n"
    $compatibleDigestRow = if ($CompatibleDescendantSha256) { "`ncompatible_descendant_sha256 = `"$CompatibleDescendantSha256`"" } else { '' }
    $manifest = @"
schema_version = 2
signature_slice_schema = "jcode-semantic-slices-v2"
hash_algorithm = "sha256"
slice_order = "ascending order field"
marker_cardinality = "start and end must each match exactly one ASCII-trimmed line"
slice_bounds = "start inclusive, end exclusive, with start preceding end"

required_paths = [
$pathRows
]

[jcode]
revision = "$Revision"

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
order = 10
id = "root-workspace-members"
path = "Cargo.toml"
normalization = "toml-v1"
start_marker = "[workspace]"
end_marker = "[lib]"
sha256 = "$($Fixture.Hashes.Toml)"$compatibleDigestRow

[[signature_slice]]
order = 20
id = "utf8-text"
path = "notes.txt"
normalization = "text-v1"
start_marker = "begin text"
end_marker = "end text"
sha256 = "$($Fixture.Hashes.Text)"

[[signature_slice]]
order = 30
id = "beta-rust"
path = "src/example.rs"
normalization = "rust-v1"
start_marker = "pub fn beta() {"
end_marker = "pub fn done() {}"
sha256 = "$($Fixture.Hashes.Beta)"

[[signature_slice]]
order = 40
id = "alpha-rust"
path = "src/example.rs"
normalization = "rust-v1"
start_marker = "pub fn alpha() {"
end_marker = "pub fn beta() {"
sha256 = "$($Fixture.Hashes.Alpha)"

[[signature_slice]]
order = 50
id = "empty-rust"
path = "src/example.rs"
normalization = "rust-v1"
start_marker = "// EMPTY_START"
end_marker = "// EMPTY_END"
sha256 = "$($Fixture.Hashes.Empty)"
"@
    [System.IO.File]::WriteAllBytes($Fixture.Manifest, (ConvertTo-Utf8Bytes -Text $manifest))
}

function Replace-ManifestText {
    param([pscustomobject]$Fixture, [string]$Find, [string]$Replace)

    $text = ConvertFrom-StrictUtf8 -Bytes ([System.IO.File]::ReadAllBytes($Fixture.Manifest))
    if ($text.IndexOf($Find, [System.StringComparison]::Ordinal) -lt 0) {
        throw "manifest fixture text was not found: $Find"
    }
    [System.IO.File]::WriteAllBytes($Fixture.Manifest, (ConvertTo-Utf8Bytes -Text $text.Replace($Find, $Replace)))
}

function Commit-FixtureBytes {
    param([pscustomobject]$Fixture, [string]$Path, [byte[]]$Bytes, [string]$Message)

    [void](Write-FixtureBytes -Root $Fixture.Root -RelativePath $Path -Bytes $Bytes)
    [void](Invoke-TestGit -Root $Fixture.Root -Arguments @('add', '--', $Path))
    [void](Invoke-TestGit -Root $Fixture.Root -Arguments @('commit', '--quiet', '-m', $Message))
}

function New-Fixture {
    if (-not (Test-Path -LiteralPath $script:GuardSource)) {
        throw "guard under test is absent: $script:GuardSource"
    }

    $root = Join-Path ([System.IO.Path]::GetTempPath()) ("jcode-intel-guard-{0}" -f [guid]::NewGuid().ToString('N'))
    [void]$script:TempRoots.Add($root)
    New-Item -ItemType Directory -Force -Path $root | Out-Null
    [void](Invoke-TestGit -Root $root -Arguments @('init', '--quiet'))
    [void](Invoke-TestGit -Root $root -Arguments @('config', 'user.email', 'guard@example.invalid'))
    [void](Invoke-TestGit -Root $root -Arguments @('config', 'user.name', 'Guard Fixture'))

    $tomlBytes = ConvertTo-Utf8Bytes -Text "[workspace]`r`n# ignored`r`nmembers = []  `r`n[lib]`r`n" -WithBom
    $rustBytes = ConvertTo-Utf8Bytes -Text "// ignored`r`n// EMPTY_START`r`n// EMPTY_END`r`npub fn alpha() {`r`n    alpha_value();`r`n}`r`npub fn beta() {`r`n    beta_value();`r`n}`r`npub fn done() {}`r`n"
    $textBytes = ConvertTo-Utf8Bytes -Text ("  begin text  `r`nvalue {0}  `r`n`r`n  end text`r`n" -f [char]0x03B1) -WithBom
    $lockBytes = ConvertTo-Utf8Bytes -Text "[[package]]`nname = `"agentgrep`"`nversion = `"0.1.6`"`nsource = `"git+https://github.com/1jehuang/agentgrep.git?tag=v0.1.6#b01b804008ab0662fa14e6b60b10bff61716e6f1`"`n"
    $tomlPath = Write-FixtureBytes -Root $root -RelativePath 'Cargo.toml' -Bytes $tomlBytes
    $rustPath = Write-FixtureBytes -Root $root -RelativePath 'src/example.rs' -Bytes $rustBytes
    $textPath = Write-FixtureBytes -Root $root -RelativePath 'notes.txt' -Bytes $textBytes
    [void](Write-FixtureBytes -Root $root -RelativePath 'Cargo.lock' -Bytes $lockBytes)
    [void](Invoke-TestGit -Root $root -Arguments @('add', 'Cargo.toml', 'Cargo.lock', 'src/example.rs', 'notes.txt'))
    [void](Invoke-TestGit -Root $root -Arguments @('commit', '--quiet', '-m', 'baseline fixture'))

    $fixture = [pscustomobject]@{
        Root = $root
        Guard = Join-Path $root 'scripts/check_intel_baseline.ps1'
        Manifest = Join-Path $root 'tools/intel/baseline.toml'
        BaselineRevision = Invoke-TestGit -Root $root -Arguments @('rev-parse', 'HEAD')
        RustPath = $rustPath
        Hashes = [pscustomobject]@{
            Toml = '01682718dc81bf6ca83b17e5a4b7fd755dc44c08357e0c817c20d66e5f474a78'
            CompatibleToml = '1cf8ad1c3f365dc8d6815a3e674639209b2a3c79581e348c97c41e4deb8a5529'
            Text = Get-NormalizedSliceHash -Bytes $textBytes -Mode 'text-v1' -StartMarker 'begin text' -EndMarker 'end text'
            Alpha = Get-NormalizedSliceHash -Bytes $rustBytes -Mode 'rust-v1' -StartMarker 'pub fn alpha() {' -EndMarker 'pub fn beta() {'
            Beta = Get-NormalizedSliceHash -Bytes $rustBytes -Mode 'rust-v1' -StartMarker 'pub fn beta() {' -EndMarker 'pub fn done() {}'
            Empty = Get-NormalizedSliceHash -Bytes $rustBytes -Mode 'rust-v1' -StartMarker '// EMPTY_START' -EndMarker '// EMPTY_END'
        }
    }
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $fixture.Guard) | Out-Null
    Copy-Item -LiteralPath $script:GuardSource -Destination $fixture.Guard
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $fixture.Manifest) | Out-Null
    Write-BaselineManifest -Fixture $fixture
    return $fixture
}

function Add-Descendant {
    param([pscustomobject]$Fixture, [string]$Path = 'README.md', [string]$Content = 'descendant')

    [void](Write-FixtureBytes -Root $Fixture.Root -RelativePath $Path -Bytes (ConvertTo-Utf8Bytes -Text $Content))
    [void](Invoke-TestGit -Root $Fixture.Root -Arguments @('add', $Path))
    [void](Invoke-TestGit -Root $Fixture.Root -Arguments @('commit', '--quiet', '-m', 'descendant fixture'))
    return Invoke-TestGit -Root $Fixture.Root -Arguments @('rev-parse', 'HEAD')
}

function Get-WorkspaceStateBytes {
    param([ValidateSet('B1', 'B2', 'B3', 'B4')][string]$State)

    $members = @(
        '    "crates/jcode-intel-types",'
        '    "crates/jcode-intel-store",'
        '    "crates/jcode-intel-search",'
        '    "crates/jcode-intel-provider",'
        '    "crates/jcode-intel-rust",'
        '    "crates/jcode-intel-core",'
        '    "crates/jcode-intel-eval",'
    )
    if ($State -eq 'B2') { $members += '    "crates/arbitrary",' }
    if ($State -eq 'B3') { $members += '    "crates/jcode-intel-evil",' }
    if ($State -eq 'B4') { $members = @($members[0..5]) }
    return ConvertTo-Utf8Bytes -Text ((@('[workspace]', 'members = [') + $members + @(']', '[lib]', '')) -join "`n")
}

function Commit-WorkspaceState {
    param([pscustomobject]$Fixture, [ValidateSet('B1', 'B2', 'B3', 'B4')][string]$State)

    Commit-FixtureBytes -Fixture $Fixture -Path 'Cargo.toml' -Bytes (Get-WorkspaceStateBytes -State $State) -Message $State
}

function Get-FixtureState {
    param([pscustomobject]$Fixture)

    return [pscustomobject]@{
        Head = Invoke-TestGit -Root $Fixture.Root -Arguments @('rev-parse', 'HEAD')
        Status = Invoke-TestGit -Root $Fixture.Root -Arguments @('status', '--porcelain=v2')
        Index = Invoke-TestGit -Root $Fixture.Root -Arguments @('ls-files', '--stage')
        Config = Invoke-TestGit -Root $Fixture.Root -Arguments @('config', '--local', '--list')
        Remotes = Invoke-TestGit -Root $Fixture.Root -Arguments @('remote', '-v')
    }
}

function Assert-FixtureState {
    param([string]$Label, [pscustomobject]$Before, [pscustomobject]$After)

    foreach ($name in @('Head', 'Status', 'Index', 'Config', 'Remotes')) {
        Assert-Equal -Label "$Label $name" -Expected $Before.$name -Actual $After.$name
    }
}

function Invoke-TestCase {
    param([string]$Name, [scriptblock]$Action)

    if ($Filter -and $Name -cne $Filter) { return }
    try {
        & $Action
        $script:Passed += 1
        Write-Host "PASS: $Name" -ForegroundColor Green
    } catch {
        [void]$script:Failures.Add("$($Name): $($_.Exception.Message)")
        Write-Host "FAIL: $($Name): $($_.Exception.Message)" -ForegroundColor Red
    }
}

try {
    Invoke-TestCase -Name 'exact revision passes with strict UTF-8, BOM, CRLF, and all normalizers' -Action {
        $fixture = New-Fixture
        $result = Invoke-Guard -Fixture $fixture
        Assert-Equal -Label 'exact exit code' -Expected 0 -Actual $result.ExitCode
        Assert-Empty -Label 'exact stdout' -Text $result.Stdout
        Assert-Empty -Label 'exact stderr' -Text $result.Stderr
    }

    Invoke-TestCase -Name 'default descendant rejection and compatible descendant acceptance' -Action {
        $fixture = New-Fixture
        $head = Add-Descendant -Fixture $fixture
        $rejected = Invoke-Guard -Fixture $fixture
        Assert-Equal -Label 'default descendant exit code' -Expected 2 -Actual $rejected.ExitCode
        Assert-Equal -Label 'default descendant stderr' -Expected "check_intel_baseline: pinned revision $($fixture.BaselineRevision), current revision $head; use --allow-descendant to verify a compatible descendant`n" -Actual $rejected.Stderr
        Assert-Empty -Label 'default descendant stdout' -Text $rejected.Stdout
        $accepted = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'compatible descendant exit code' -Expected 0 -Actual $accepted.ExitCode
        Assert-Empty -Label 'compatible descendant stdout' -Text $accepted.Stdout
        Assert-Empty -Label 'compatible descendant stderr' -Text $accepted.Stderr
    }

    Invoke-TestCase -Name 'accepts the committed reviewed seven-member workspace descendant only' -Action {
        $fixture = New-Fixture
        Write-BaselineManifest -Fixture $fixture -CompatibleDescendantSha256 $fixture.Hashes.CompatibleToml
        $pinned = Invoke-Guard -Fixture $fixture
        if ($pinned.ExitCode -ne 0) { throw "B0 pinned-primary expected exit 0 but got $($pinned.ExitCode); stderr: $($pinned.Stderr.Trim())" }
        Assert-Empty -Label 'B0 pinned-primary stdout' -Text $pinned.Stdout
        Assert-Empty -Label 'B0 pinned-primary stderr' -Text $pinned.Stderr

        Commit-WorkspaceState -Fixture $fixture -State B1
        $reviewed = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'B1 reviewed alternate exit code' -Expected 0 -Actual $reviewed.ExitCode
        Assert-Empty -Label 'B1 reviewed alternate stdout' -Text $reviewed.Stdout
        Assert-Empty -Label 'B1 reviewed alternate stderr' -Text $reviewed.Stderr

        Commit-WorkspaceState -Fixture $fixture -State B2
        $eighth = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'B2 arbitrary eighth exit code' -Expected 2 -Actual $eighth.ExitCode
        Assert-Empty -Label 'B2 arbitrary eighth stdout' -Text $eighth.Stdout
        Assert-Equal -Label 'B2 arbitrary eighth stderr' -Expected "check_intel_baseline: current signature root-workspace-members: SHA-256 mismatch`n" -Actual $eighth.Stderr

        Commit-WorkspaceState -Fixture $fixture -State B3
        $evilEighth = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'B3 evil eighth exit code' -Expected 2 -Actual $evilEighth.ExitCode
        Assert-Empty -Label 'B3 evil eighth stdout' -Text $evilEighth.Stdout
        Assert-Equal -Label 'B3 evil eighth stderr' -Expected "check_intel_baseline: current signature root-workspace-members: SHA-256 mismatch`n" -Actual $evilEighth.Stderr

        Commit-WorkspaceState -Fixture $fixture -State B4
        $omitted = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'B4 omitted member exit code' -Expected 2 -Actual $omitted.ExitCode
        Assert-Empty -Label 'B4 omitted member stdout' -Text $omitted.Stdout
        Assert-Equal -Label 'B4 omitted member stderr' -Expected "check_intel_baseline: current signature root-workspace-members: SHA-256 mismatch`n" -Actual $omitted.Stderr
    }

    Invoke-TestCase -Name 'rejects malformed compatible descendant hashes and schema versions' -Action {
        $alternate = '1cf8ad1c3f365dc8d6815a3e674639209b2a3c79581e348c97c41e4deb8a5529'
        $primary = '01682718dc81bf6ca83b17e5a4b7fd755dc44c08357e0c817c20d66e5f474a78'
        $malformedRows = @(
            'compatible_descendant_sha256 = ""'
            'compatible_descendant_sha256 = "1CF8AD1C3F365DC8D6815A3E674639209B2A3C79581E348C97C41E4DEB8A5529"'
            'compatible_descendant_sha256 = "1cf8ad1c"'
            'compatible_descendant_sha256 = "gcf8ad1c3f365dc8d6815a3e674639209b2a3c79581e348c97c41e4deb8a5529"'
            "compatible_descendant_sha256 = [ `"$alternate`" ]"
            "compatible_descendant_sha256 = `"$primary`""
        )
        foreach ($row in $malformedRows) {
            $fixture = New-Fixture
            Write-BaselineManifest -Fixture $fixture -CompatibleDescendantSha256 $alternate
            Replace-ManifestText -Fixture $fixture -Find "compatible_descendant_sha256 = `"$alternate`"" -Replace $row
            $result = Invoke-Guard -Fixture $fixture
            Assert-Equal -Label "malformed alternate '$row' exit code" -Expected 2 -Actual $result.ExitCode
            Assert-Contains -Label 'malformed alternate stderr' -Text $result.Stderr -Needle 'manifest invalid'
            Assert-Empty -Label 'malformed alternate stdout' -Text $result.Stdout
        }

        $unknownFixture = New-Fixture
        Write-BaselineManifest -Fixture $unknownFixture -CompatibleDescendantSha256 $alternate
        Replace-ManifestText -Fixture $unknownFixture -Find 'compatible_descendant_sha256' -Replace 'compatible_descendant_sha257'
        $unknown = Invoke-Guard -Fixture $unknownFixture
        Assert-Equal -Label 'unknown alternate key exit code' -Expected 2 -Actual $unknown.ExitCode
        Assert-Contains -Label 'unknown alternate key stderr' -Text $unknown.Stderr -Needle 'manifest invalid'

        $duplicateFixture = New-Fixture
        Write-BaselineManifest -Fixture $duplicateFixture -CompatibleDescendantSha256 $alternate
        Replace-ManifestText -Fixture $duplicateFixture -Find "compatible_descendant_sha256 = `"$alternate`"" -Replace "compatible_descendant_sha256 = `"$alternate`"`ncompatible_descendant_sha256 = `"$alternate`""
        $duplicate = Invoke-Guard -Fixture $duplicateFixture
        Assert-Equal -Label 'duplicate alternate key exit code' -Expected 2 -Actual $duplicate.ExitCode
        Assert-Contains -Label 'duplicate alternate key stderr' -Text $duplicate.Stderr -Needle 'manifest invalid'

        $sectionFixture = New-Fixture
        Write-BaselineManifest -Fixture $sectionFixture -CompatibleDescendantSha256 $alternate
        Replace-ManifestText -Fixture $sectionFixture -Find "compatible_descendant_sha256 = `"$alternate`"" -Replace ''
        Replace-ManifestText -Fixture $sectionFixture -Find "revision = `"$($sectionFixture.BaselineRevision)`"" -Replace "revision = `"$($sectionFixture.BaselineRevision)`"`ncompatible_descendant_sha256 = `"$alternate`""
        $wrongSection = Invoke-Guard -Fixture $sectionFixture
        Assert-Equal -Label 'wrong-section alternate exit code' -Expected 2 -Actual $wrongSection.ExitCode
        Assert-Contains -Label 'wrong-section alternate stderr' -Text $wrongSection.Stderr -Needle 'manifest invalid'

        foreach ($schema in @('1', '3', '"2"', '[2]')) {
            $schemaFixture = New-Fixture
            Replace-ManifestText -Fixture $schemaFixture -Find 'schema_version = 2' -Replace "schema_version = $schema"
            $schemaResult = Invoke-Guard -Fixture $schemaFixture
            Assert-Equal -Label "schema $schema exit code" -Expected 2 -Actual $schemaResult.ExitCode
            Assert-Contains -Label 'schema stderr' -Text $schemaResult.Stderr -Needle 'manifest invalid'
            Assert-Empty -Label 'schema stdout' -Text $schemaResult.Stdout
        }
    }

    Invoke-TestCase -Name 'missing required path rejects with exit 2 and the exact path' -Action {
        $fixture = New-Fixture
        Write-BaselineManifest -Fixture $fixture -MissingPath 'missing/integration.rs'
        $result = Invoke-Guard -Fixture $fixture
        Assert-Equal -Label 'missing path exit code' -Expected 2 -Actual $result.ExitCode
        Assert-Contains -Label 'missing path stderr' -Text $result.Stderr -Needle 'pinned required path missing: missing/integration.rs'
        Assert-Empty -Label 'missing path stdout' -Text $result.Stdout
    }

    Invoke-TestCase -Name 'hash drift and marker drift reject compatible descendants' -Action {
        $hashFixture = New-Fixture
        $changed = ConvertTo-Utf8Bytes -Text "// ignored`r`npub fn alpha() {`r`n    changed_alpha();`r`n}`r`npub fn beta() {`r`n    beta_value();`r`n}`r`npub fn done() {}`r`n"
        [System.IO.File]::WriteAllBytes($hashFixture.RustPath, $changed)
        [void](Invoke-TestGit -Root $hashFixture.Root -Arguments @('add', 'src/example.rs'))
        [void](Invoke-TestGit -Root $hashFixture.Root -Arguments @('commit', '--quiet', '-m', 'hash drift'))
        $hashResult = Invoke-Guard -Fixture $hashFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'hash drift exit code' -Expected 2 -Actual $hashResult.ExitCode
        Assert-Contains -Label 'hash drift stderr' -Text $hashResult.Stderr -Needle 'current signature alpha-rust: SHA-256 mismatch'
        Assert-Empty -Label 'hash drift stdout' -Text $hashResult.Stdout

        $markerFixture = New-Fixture
        $broken = ConvertTo-Utf8Bytes -Text "// ignored`r`npub fn alpha() {`r`n    alpha_value();`r`n}`r`npub fn renamed_beta() {`r`n    beta_value();`r`n}`r`npub fn done() {}`r`n"
        [System.IO.File]::WriteAllBytes($markerFixture.RustPath, $broken)
        [void](Invoke-TestGit -Root $markerFixture.Root -Arguments @('add', 'src/example.rs'))
        [void](Invoke-TestGit -Root $markerFixture.Root -Arguments @('commit', '--quiet', '-m', 'marker drift'))
        $markerResult = Invoke-Guard -Fixture $markerFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'marker drift exit code' -Expected 2 -Actual $markerResult.ExitCode
        Assert-Contains -Label 'marker drift stderr' -Text $markerResult.Stderr -Needle 'current signature beta-rust: start marker matched 0 lines'
        Assert-Empty -Label 'marker drift stdout' -Text $markerResult.Stdout
    }

    Invoke-TestCase -Name 'malformed manifest and missing object reject without success output' -Action {
        $malformedFixture = New-Fixture
        [System.IO.File]::WriteAllBytes($malformedFixture.Manifest, (ConvertTo-Utf8Bytes -Text 'schema_version ='))
        $malformed = Invoke-Guard -Fixture $malformedFixture
        Assert-Equal -Label 'malformed manifest exit code' -Expected 2 -Actual $malformed.ExitCode
        Assert-Contains -Label 'malformed manifest stderr' -Text $malformed.Stderr -Needle 'manifest invalid'
        Assert-Empty -Label 'malformed manifest stdout' -Text $malformed.Stdout

        $objectFixture = New-Fixture
        Write-BaselineManifest -Fixture $objectFixture -Revision (('0' * 40) -join '')
        $missingObject = Invoke-Guard -Fixture $objectFixture
        Assert-Equal -Label 'missing object exit code' -Expected 2 -Actual $missingObject.ExitCode
        Assert-Contains -Label 'missing object stderr' -Text $missingObject.Stderr -Needle 'baseline object missing:'
        Assert-Empty -Label 'missing object stdout' -Text $missingObject.Stdout
    }

    Invoke-TestCase -Name 'missing Git tool and unknown arguments reject with exit 2' -Action {
        $fixture = New-Fixture
        $missingTool = Invoke-Guard -Fixture $fixture -WithoutGit
        Assert-Equal -Label 'missing tool exit code' -Expected 2 -Actual $missingTool.ExitCode
        Assert-Contains -Label 'missing tool stderr' -Text $missingTool.Stderr -Needle 'git is unavailable'
        Assert-Empty -Label 'missing tool stdout' -Text $missingTool.Stdout
        $unknown = Invoke-Guard -Fixture $fixture -Arguments @('--not-an-option')
        Assert-Equal -Label 'unknown argument exit code' -Expected 2 -Actual $unknown.ExitCode
        Assert-Equal -Label 'unknown argument stderr' -Expected "check_intel_baseline: usage: check_intel_baseline.ps1 [--allow-descendant]`n" -Actual $unknown.Stderr
        Assert-Empty -Label 'unknown argument stdout' -Text $unknown.Stdout
    }

    Invoke-TestCase -Name 'multiple errors are deterministic and Git target state is preserved' -Action {
        $fixture = New-Fixture
        $changed = ConvertTo-Utf8Bytes -Text "// ignored`r`npub fn alpha() {`r`n    changed_alpha();`r`n}`r`npub fn beta() {`r`n    changed_beta();`r`n}`r`npub fn done() {}`r`n"
        [System.IO.File]::WriteAllBytes($fixture.RustPath, $changed)
        [void](Invoke-TestGit -Root $fixture.Root -Arguments @('add', 'src/example.rs'))
        [void](Invoke-TestGit -Root $fixture.Root -Arguments @('commit', '--quiet', '-m', 'multiple drift'))
        $beforeFailure = Get-FixtureState -Fixture $fixture
        $first = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        $afterFailure = Get-FixtureState -Fixture $fixture
        $second = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'multiple drift first exit code' -Expected 2 -Actual $first.ExitCode
        Assert-Equal -Label 'multiple drift second exit code' -Expected 2 -Actual $second.ExitCode
        Assert-Equal -Label 'deterministic stderr' -Expected $first.Stderr -Actual $second.Stderr
        Assert-Empty -Label 'multiple drift stdout' -Text $first.Stdout
        Assert-FixtureState -Label 'failing guard state' -Before $beforeFailure -After $afterFailure
        $betaPosition = $first.Stderr.IndexOf('current signature beta-rust: SHA-256 mismatch', [System.StringComparison]::Ordinal)
        $alphaPosition = $first.Stderr.IndexOf('current signature alpha-rust: SHA-256 mismatch', [System.StringComparison]::Ordinal)
        if ($betaPosition -lt 0 -or $alphaPosition -lt 0 -or $betaPosition -ge $alphaPosition) {
            throw "slice diagnostics were not ordered by ascending order field: $($first.Stderr)"
        }

        $compatibleFixture = New-Fixture
        [void](Add-Descendant -Fixture $compatibleFixture)
        $beforeSuccess = Get-FixtureState -Fixture $compatibleFixture
        $success = Invoke-Guard -Fixture $compatibleFixture -Arguments @('--allow-descendant')
        $afterSuccess = Get-FixtureState -Fixture $compatibleFixture
        Assert-Equal -Label 'compatible state exit code' -Expected 0 -Actual $success.ExitCode
        Assert-FixtureState -Label 'successful guard state' -Before $beforeSuccess -After $afterSuccess
    }

    Invoke-TestCase -Name 'bare CR input normalizes to the pinned text signature' -Action {
        $fixture = New-Fixture
        $bareCr = ConvertTo-Utf8Bytes -Text ("  begin text  `rvalue {0}  `r`r  end text`r" -f [char]0x03B1)
        Commit-FixtureBytes -Fixture $fixture -Path 'notes.txt' -Bytes $bareCr -Message 'bare cr'
        $result = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'bare CR exit code' -Expected 0 -Actual $result.ExitCode
        Assert-Empty -Label 'bare CR stdout' -Text $result.Stdout
        Assert-Empty -Label 'bare CR stderr' -Text $result.Stderr
    }

    Invoke-TestCase -Name 'out-of-slice edits in a required source file preserve compatibility' -Action {
        $fixture = New-Fixture
        Commit-FixtureBytes -Fixture $fixture -Path 'src/example.rs' -Bytes (ConvertTo-Utf8Bytes -Text "// changed outside signatures`n// ignored`n// EMPTY_START`n// EMPTY_END`npub fn alpha() {`n    alpha_value();`n}`npub fn beta() {`n    beta_value();`n}`npub fn done() {}`n") -Message 'out of slice'
        $result = Invoke-Guard -Fixture $fixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'out-of-slice exit code' -Expected 0 -Actual $result.ExitCode
        Assert-Empty -Label 'out-of-slice stdout' -Text $result.Stdout
        Assert-Empty -Label 'out-of-slice stderr' -Text $result.Stderr
    }

    Invoke-TestCase -Name 'invalid UTF-8 and every marker cardinality failure reject descendants' -Action {
        $invalidFixture = New-Fixture
        $valid = ConvertTo-Utf8Bytes -Text "// ignored`npub fn alpha() {`n    alpha_value();`n}`npub fn beta() {`n    beta_value();`n}`npub fn done() {}`n"
        $invalid = New-Object byte[] ($valid.Length + 1)
        [System.Array]::Copy($valid, $invalid, $valid.Length)
        $invalid[$invalid.Length - 1] = 0xFF
        Commit-FixtureBytes -Fixture $invalidFixture -Path 'src/example.rs' -Bytes $invalid -Message 'invalid utf8'
        $invalidResult = Invoke-Guard -Fixture $invalidFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'invalid UTF-8 exit code' -Expected 2 -Actual $invalidResult.ExitCode
        Assert-Contains -Label 'invalid UTF-8 stderr' -Text $invalidResult.Stderr -Needle 'current signature beta-rust: invalid UTF-8'

        $duplicateFixture = New-Fixture
        Commit-FixtureBytes -Fixture $duplicateFixture -Path 'src/example.rs' -Bytes (ConvertTo-Utf8Bytes -Text "// ignored`npub fn alpha() {`n    alpha_value();`n}`npub fn alpha() {`n    duplicate();`n}`npub fn beta() {`n    beta_value();`n}`npub fn done() {}`n") -Message 'duplicate marker'
        $duplicateResult = Invoke-Guard -Fixture $duplicateFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'duplicate marker exit code' -Expected 2 -Actual $duplicateResult.ExitCode
        Assert-Contains -Label 'duplicate marker stderr' -Text $duplicateResult.Stderr -Needle 'current signature alpha-rust: start marker matched 2 lines'

        $reversedFixture = New-Fixture
        Commit-FixtureBytes -Fixture $reversedFixture -Path 'src/example.rs' -Bytes (ConvertTo-Utf8Bytes -Text "// ignored`npub fn beta() {`n    beta_value();`n}`npub fn done() {}`npub fn alpha() {`n    alpha_value();`n}`n") -Message 'reversed markers'
        $reversedResult = Invoke-Guard -Fixture $reversedFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'reversed marker exit code' -Expected 2 -Actual $reversedResult.ExitCode
        Assert-Contains -Label 'reversed marker stderr' -Text $reversedResult.Stderr -Needle 'current signature alpha-rust: start marker does not precede end marker'

        $missingFixture = New-Fixture
        Commit-FixtureBytes -Fixture $missingFixture -Path 'src/example.rs' -Bytes (ConvertTo-Utf8Bytes -Text "// ignored`npub fn renamed_alpha() {`n    alpha_value();`n}`npub fn beta() {`n    beta_value();`n}`npub fn done() {}`n") -Message 'missing marker'
        $missingResult = Invoke-Guard -Fixture $missingFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'missing marker exit code' -Expected 2 -Actual $missingResult.ExitCode
        Assert-Contains -Label 'missing marker stderr' -Text $missingResult.Stderr -Needle 'current signature alpha-rust: start marker matched 0 lines'

        $duplicateEndFixture = New-Fixture
        Commit-FixtureBytes -Fixture $duplicateEndFixture -Path 'src/example.rs' -Bytes (ConvertTo-Utf8Bytes -Text "// ignored`n// EMPTY_START`n// EMPTY_END`npub fn alpha() {`n    alpha_value();`n}`npub fn beta() {`n    beta_value();`n}`npub fn done() {}`npub fn done() {}`n") -Message 'duplicate end marker'
        $duplicateEndResult = Invoke-Guard -Fixture $duplicateEndFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'duplicate end marker exit code' -Expected 2 -Actual $duplicateEndResult.ExitCode
        Assert-Contains -Label 'duplicate end marker stderr' -Text $duplicateEndResult.Stderr -Needle 'current signature beta-rust: end marker matched 2 lines'

        $missingEndFixture = New-Fixture
        Commit-FixtureBytes -Fixture $missingEndFixture -Path 'src/example.rs' -Bytes (ConvertTo-Utf8Bytes -Text "// ignored`n// EMPTY_START`n// EMPTY_END`npub fn alpha() {`n    alpha_value();`n}`npub fn beta() {`n    beta_value();`n}`npub fn renamed_done() {}`n") -Message 'missing end marker'
        $missingEndResult = Invoke-Guard -Fixture $missingEndFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'missing end marker exit code' -Expected 2 -Actual $missingEndResult.ExitCode
        Assert-Contains -Label 'missing end marker stderr' -Text $missingEndResult.Stderr -Needle 'current signature beta-rust: end marker matched 0 lines'
    }

    Invoke-TestCase -Name 'current descendant missing paths and non-descendants reject before signature success' -Action {
        $missingFixture = New-Fixture
        Remove-Item -LiteralPath (Join-Path $missingFixture.Root 'notes.txt')
        [void](Invoke-TestGit -Root $missingFixture.Root -Arguments @('add', '-u', '--', 'notes.txt'))
        [void](Invoke-TestGit -Root $missingFixture.Root -Arguments @('commit', '--quiet', '-m', 'missing current path'))
        $missingResult = Invoke-Guard -Fixture $missingFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'current missing path exit code' -Expected 2 -Actual $missingResult.ExitCode
        Assert-Contains -Label 'current missing path stderr' -Text $missingResult.Stderr -Needle 'current required path missing: notes.txt'

        $unrelatedFixture = New-Fixture
        $emptyTree = Invoke-TestGit -Root $unrelatedFixture.Root -Arguments @('mktree')
        $unrelated = Invoke-TestGit -Root $unrelatedFixture.Root -Arguments @('commit-tree', $emptyTree, '-m', 'unrelated')
        $branch = Invoke-TestGit -Root $unrelatedFixture.Root -Arguments @('symbolic-ref', '--short', 'HEAD')
        [void](Invoke-TestGit -Root $unrelatedFixture.Root -Arguments @('update-ref', "refs/heads/$branch", $unrelated))
        $unrelatedResult = Invoke-Guard -Fixture $unrelatedFixture -Arguments @('--allow-descendant')
        Assert-Equal -Label 'non-descendant exit code' -Expected 2 -Actual $unrelatedResult.ExitCode
        Assert-Contains -Label 'non-descendant stderr' -Text $unrelatedResult.Stderr -Needle "current revision $unrelated is not a descendant of pinned revision $($unrelatedFixture.BaselineRevision)"
    }

    Invoke-TestCase -Name 'strict manifest parsing rejects duplicate paths, commas, escapes, empty identity, and noncanonical orders' -Action {
        $duplicatePathFixture = New-Fixture
        Replace-ManifestText -Fixture $duplicatePathFixture -Find '    "notes.txt",' -Replace ('    "notes.txt",' + "`n" + '    "notes.txt",')
        $duplicatePath = Invoke-Guard -Fixture $duplicatePathFixture
        Assert-Equal -Label 'duplicate path exit code' -Expected 2 -Actual $duplicatePath.ExitCode
        Assert-Contains -Label 'duplicate path stderr' -Text $duplicatePath.Stderr -Needle 'manifest invalid'

        $commaFixture = New-Fixture
        Replace-ManifestText -Fixture $commaFixture -Find '    "notes.txt",' -Replace '    "notes.txt"'
        $comma = Invoke-Guard -Fixture $commaFixture
        Assert-Equal -Label 'missing comma exit code' -Expected 2 -Actual $comma.ExitCode
        Assert-Contains -Label 'missing comma stderr' -Text $comma.Stderr -Needle 'manifest invalid'

        $escapeFixture = New-Fixture
        Replace-ManifestText -Fixture $escapeFixture -Find 'start_marker = "begin text"' -Replace 'start_marker = "begin\btext"'
        $escape = Invoke-Guard -Fixture $escapeFixture
        Assert-Equal -Label 'unsupported escape exit code' -Expected 2 -Actual $escape.ExitCode
        Assert-Contains -Label 'unsupported escape stderr' -Text $escape.Stderr -Needle 'manifest invalid'

        foreach ($identity in @(@('version = "0.1.6"', 'version = ""'), @('tag = "v0.1.6"', 'tag = ""'), @('repository = "https://github.com/1jehuang/agentgrep.git"', 'repository = ""'))) {
            $identityFixture = New-Fixture
            Replace-ManifestText -Fixture $identityFixture -Find $identity[0] -Replace $identity[1]
            $identityResult = Invoke-Guard -Fixture $identityFixture
            Assert-Equal -Label "empty agentgrep $($identity[0].Split('=')[0].Trim()) exit code" -Expected 2 -Actual $identityResult.ExitCode
            Assert-Contains -Label 'empty agentgrep stderr' -Text $identityResult.Stderr -Needle 'manifest invalid'
        }

        foreach ($order in @('01', '+10')) {
            $orderFixture = New-Fixture
            Replace-ManifestText -Fixture $orderFixture -Find 'order = 10' -Replace "order = $order"
            $orderResult = Invoke-Guard -Fixture $orderFixture
            Assert-Equal -Label "noncanonical order $order exit code" -Expected 2 -Actual $orderResult.ExitCode
            Assert-Contains -Label 'noncanonical order stderr' -Text $orderResult.Stderr -Needle 'manifest invalid'
        }

        $duplicateKeyFixture = New-Fixture
        Replace-ManifestText -Fixture $duplicateKeyFixture -Find 'schema_version = 2' -Replace "schema_version = 2`nschema_version = 2"
        $duplicateKey = Invoke-Guard -Fixture $duplicateKeyFixture
        Assert-Equal -Label 'duplicate key exit code' -Expected 2 -Actual $duplicateKey.ExitCode
        Assert-Contains -Label 'duplicate key stderr' -Text $duplicateKey.Stderr -Needle 'manifest invalid'

        $caseTokenFixture = New-Fixture
        Replace-ManifestText -Fixture $caseTokenFixture -Find 'schema_version = 2' -Replace 'Schema_version = 2'
        $caseToken = Invoke-Guard -Fixture $caseTokenFixture
        Assert-Equal -Label 'case-sensitive token exit code' -Expected 2 -Actual $caseToken.ExitCode
        Assert-Contains -Label 'case-sensitive token stderr' -Text $caseToken.Stderr -Needle 'manifest invalid'

        $sectionFixture = New-Fixture
        Replace-ManifestText -Fixture $sectionFixture -Find '[agentgrep]' -Replace "[jcode]`n`n[agentgrep]"
        $duplicateSection = Invoke-Guard -Fixture $sectionFixture
        Assert-Equal -Label 'duplicate empty section exit code' -Expected 2 -Actual $duplicateSection.ExitCode
        Assert-Contains -Label 'duplicate empty section stderr' -Text $duplicateSection.Stderr -Needle 'manifest invalid'

        $revisionCaseFixture = New-Fixture
        Replace-ManifestText -Fixture $revisionCaseFixture -Find '[agentgrep]' -Replace ("Revision = `"$($revisionCaseFixture.BaselineRevision)`"`n`n[agentgrep]")
        $revisionCase = Invoke-Guard -Fixture $revisionCaseFixture
        Assert-Equal -Label 'case-variant jcode key exit code' -Expected 2 -Actual $revisionCase.ExitCode
        Assert-Contains -Label 'case-variant jcode key stderr' -Text $revisionCase.Stderr -Needle 'manifest invalid'
    }

    Invoke-TestCase -Name 'case-distinct required paths remain distinct and long or multiple arguments stay generic' -Action {
        $caseFixture = New-Fixture
        Replace-ManifestText -Fixture $caseFixture -Find '    "notes.txt",' -Replace ('    "notes.txt",' + "`n" + '    "CaseOnly.txt",' + "`n" + '    "caseonly.txt",')
        $caseResult = Invoke-Guard -Fixture $caseFixture
        Assert-Equal -Label 'case-distinct path exit code' -Expected 2 -Actual $caseResult.ExitCode
        Assert-Contains -Label 'uppercase path stderr' -Text $caseResult.Stderr -Needle 'pinned required path missing: CaseOnly.txt'
        Assert-Contains -Label 'lowercase path stderr' -Text $caseResult.Stderr -Needle 'pinned required path missing: caseonly.txt'

        $argumentFixture = New-Fixture
        $longArgument = 'x' * 8000
        $longResult = Invoke-Guard -Fixture $argumentFixture -Arguments @($longArgument)
        Assert-Equal -Label 'long argument exit code' -Expected 2 -Actual $longResult.ExitCode
        Assert-Equal -Label 'long argument stderr' -Expected "check_intel_baseline: usage: check_intel_baseline.ps1 [--allow-descendant]`n" -Actual $longResult.Stderr
        $multipleResult = Invoke-Guard -Fixture $argumentFixture -Arguments @('--allow-descendant', '--allow-descendant')
        Assert-Equal -Label 'multiple argument exit code' -Expected 2 -Actual $multipleResult.ExitCode
        Assert-Equal -Label 'multiple argument stderr' -Expected "check_intel_baseline: usage: check_intel_baseline.ps1 [--allow-descendant]`n" -Actual $multipleResult.Stderr
    }
} finally {
    $cleanupFailed = $false
    foreach ($root in $script:TempRoots) {
        Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction SilentlyContinue
        if (Test-Path -LiteralPath $root) {
            $cleanupFailed = $true
            [void]$script:Failures.Add("temporary fixture was not removed: $root")
        }
    }
    if (-not $cleanupFailed) {
        Write-Host "PASS: cleaned $($script:TempRoots.Count) temporary fixture(s)" -ForegroundColor Green
    }
}

if ($script:Failures.Count -gt 0) {
    $script:Failures | ForEach-Object { Write-Host "FAIL: $_" -ForegroundColor Red }
    exit 1
}

Write-Host "PASS: $($script:Passed) PowerShell baseline guard checks" -ForegroundColor Green
exit 0
