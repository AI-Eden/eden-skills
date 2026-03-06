$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoApiUrlDefault = "https://api.github.com/repos/AI-Eden/eden-skills/releases/latest"
$RepoReleaseBaseUrlDefault = "https://github.com/AI-Eden/eden-skills/releases/download"

function Write-Info {
    param([string]$Message)
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "warning: $Message"
}

function Fail {
    param([string]$Message)
    throw "error: $Message"
}

function Normalize-Version {
    param([string]$Version)

    if ($Version.StartsWith("v")) {
        return $Version.Substring(1)
    }

    return $Version
}

function Resolve-Version {
    if ($env:EDEN_SKILLS_VERSION) {
        return Normalize-Version $env:EDEN_SKILLS_VERSION
    }

    $apiUrl = if ($env:EDEN_SKILLS_RELEASE_API_URL) {
        $env:EDEN_SKILLS_RELEASE_API_URL
    } else {
        $RepoApiUrlDefault
    }

    $release = Invoke-RestMethod -Uri $apiUrl
    if (-not $release.tag_name) {
        Fail "Failed to resolve the latest release version from $apiUrl"
    }

    return Normalize-Version $release.tag_name
}

function Resolve-Target {
    $architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    if ($architecture -ne "X64") {
        Fail "Unsupported Windows architecture: $architecture"
    }

    return "x86_64-pc-windows-msvc"
}

function Get-InstallDir {
    if ($env:EDEN_SKILLS_INSTALL_DIR) {
        return $env:EDEN_SKILLS_INSTALL_DIR
    }

    if (-not $env:USERPROFILE) {
        Fail "USERPROFILE must be set before running this installer."
    }

    return (Join-Path $env:USERPROFILE ".eden-skills\bin")
}

function Read-Checksums {
    param([string]$ChecksumsPath)

    $checksums = @{}
    foreach ($line in Get-Content -Path $ChecksumsPath) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }

        $parts = $line -split "\s+", 2
        if ($parts.Length -eq 2) {
            $checksums[$parts[1].Trim()] = $parts[0].Trim()
        }
    }

    return $checksums
}

function Verify-Sha256 {
    param(
        [string]$ArchivePath,
        [string]$ChecksumsPath,
        [string]$ArchiveName
    )

    $checksums = Read-Checksums $ChecksumsPath
    if (-not $checksums.ContainsKey($ArchiveName)) {
        Fail "Checksum entry not found for $ArchiveName"
    }

    $actualHash = (Get-FileHash -Algorithm SHA256 -Path $ArchivePath).Hash.ToLowerInvariant()
    $expectedHash = $checksums[$ArchiveName].ToLowerInvariant()

    if ($actualHash -ne $expectedHash) {
        Fail "SHA-256 mismatch for $ArchiveName"
    }
}

function Ensure-PathEntry {
    param([string]$InstallDir)

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathEntries = @()
    if ($userPath) {
        $pathEntries = $userPath.Split(";") | Where-Object { $_ }
    }

    if ($pathEntries -contains $InstallDir) {
        return
    }

    $newPath = if ($userPath) { "$userPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")

    Write-Host ""
    Write-Host "Added $InstallDir to the user Path."
    Write-Host "Open a new terminal session to use eden-skills from PATH."
}

function Main {
    $target = Resolve-Target
    $version = Resolve-Version
    $releaseBaseUrl = if ($env:EDEN_SKILLS_RELEASE_BASE_URL) {
        $env:EDEN_SKILLS_RELEASE_BASE_URL
    } else {
        $RepoReleaseBaseUrlDefault
    }
    $installDir = Get-InstallDir
    $archiveName = "eden-skills-$version-$target.zip"
    $checksumsName = "eden-skills-$version-checksums.txt"
    $archiveUrl = "$releaseBaseUrl/v$version/$archiveName"
    $checksumsUrl = "$releaseBaseUrl/v$version/$checksumsName"
    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("eden-skills-" + [Guid]::NewGuid().ToString("N"))
    $archivePath = Join-Path $tempDir $archiveName
    $checksumsPath = Join-Path $tempDir $checksumsName
    $extractDir = Join-Path $tempDir "extract"
    $installedBinary = Join-Path $installDir "eden-skills.exe"

    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try {
        Write-Info "Detected target: $target"
        Write-Info "Resolved version: $version"
        Write-Info "Downloading $archiveName"
        Invoke-WebRequest -Uri $archiveUrl -OutFile $archivePath
        Write-Info "Downloading $checksumsName"
        Invoke-WebRequest -Uri $checksumsUrl -OutFile $checksumsPath
        Verify-Sha256 -ArchivePath $archivePath -ChecksumsPath $checksumsPath -ArchiveName $archiveName

        New-Item -ItemType Directory -Path $extractDir -Force | Out-Null
        Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force

        $binaryPath = Join-Path $extractDir "eden-skills.exe"
        if (-not (Test-Path -LiteralPath $binaryPath)) {
            Fail "Archive did not contain eden-skills.exe"
        }

        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        Copy-Item -Path $binaryPath -Destination $installedBinary -Force

        Write-Info "Installed eden-skills $version to $installDir"

        if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
            Write-Warn "Git was not found in PATH. Source sync commands require git."
        }

        $versionOutput = & $installedBinary --version
        Write-Info $versionOutput

        Ensure-PathEntry -InstallDir $installDir
    } finally {
        Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Main
