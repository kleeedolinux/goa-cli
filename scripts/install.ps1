# GOA CLI Windows Installer
# This script downloads and installs the GOA CLI from the mirror service

Write-Host "GOA CLI Installer for Windows" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan
Write-Host

# Define installation directory (in user's AppData\Roaming)
$installDir = Join-Path $env:APPDATA "GoaCliTool"
$binPath = Join-Path $installDir "goa.exe"

# Check if GOA CLI is already installed
$isUpdate = $false
if (Test-Path $binPath) {
    $isUpdate = $true
    $confirmUpdate = Read-Host "GOA CLI is already installed. Do you want to update it? (y/n)"
    if ($confirmUpdate -ne "y" -and $confirmUpdate -ne "Y") {
        Write-Host "Update cancelled. Exiting..." -ForegroundColor Yellow
        exit 0
    }
    Write-Host "Updating GOA CLI..." -ForegroundColor Yellow
}

# Create installation directory if it doesn't exist
if (-not (Test-Path $installDir)) {
    Write-Host "Creating installation directory..." -ForegroundColor Yellow
    New-Item -Path $installDir -ItemType Directory -Force | Out-Null
}

# Function to follow redirects and get final URL
function Get-RedirectedUrl {
    param (
        [Parameter(Mandatory=$true)]
        [string]$URL
    )

    try {
        $request = [System.Net.WebRequest]::Create($URL)
        $request.AllowAutoRedirect = $false
        $request.Method = "HEAD"
        
        $response = $request.GetResponse()
        
        if ($response.StatusCode -eq "Found" -or $response.StatusCode -eq "Moved" -or $response.StatusCode -eq "Redirect" -or $response.StatusCode -eq "TemporaryRedirect" -or $response.StatusCode -eq "PermanentRedirect") {
            $newUrl = $response.GetResponseHeader("Location")
            $response.Close()
            return $newUrl
        }
        else {
            $response.Close()
            return $URL
        }
    }
    catch {
        Write-Host "Warning: Could not resolve redirect. Using original URL." -ForegroundColor Yellow
        return $URL
    }
}

# Download the binary
Write-Host "Downloading GOA CLI from mirror service..." -ForegroundColor Yellow
$tempFile = Join-Path $env:TEMP "goa-windows-temp.exe"

# First try with HTTPS (some services automatically redirect HTTP to HTTPS)
$downloadUrl = "https://re.juliaklee.wtf/windows"

# If there's a redirection, follow it manually
try {
    Write-Host "Resolving download URL..." -ForegroundColor Yellow
    $finalUrl = Get-RedirectedUrl -URL $downloadUrl
    
    if ($finalUrl -ne $downloadUrl) {
        Write-Host "Following redirect to: $finalUrl" -ForegroundColor Yellow
        $downloadUrl = $finalUrl
    }
    
    Write-Host "Downloading from: $downloadUrl" -ForegroundColor Yellow
    
    # Use .NET WebClient for better reliability with redirects
    $webClient = New-Object System.Net.WebClient
    $webClient.Headers.Add("User-Agent", "GOA-CLI-Installer")
    $webClient.DownloadFile($downloadUrl, $tempFile)
}
catch {
    # Try direct download from dist directory as fallback
    try {
        Write-Host "Primary download failed, trying fallback method..." -ForegroundColor Yellow
        $fallbackUrl = "https://raw.githubusercontent.com/goonairplanes/goa-cli/main/dist/goa-windows.exe"
        
        $webClient = New-Object System.Net.WebClient
        $webClient.Headers.Add("User-Agent", "GOA-CLI-Installer")
        $webClient.DownloadFile($fallbackUrl, $tempFile)
    }
    catch {
        Write-Host "Error: All download attempts failed." -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
        Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
        exit 1
    }
}

# Verify file was downloaded successfully
if (-not (Test-Path $tempFile) -or (Get-Item $tempFile).Length -eq 0) {
    Write-Host "Error: Download appears to have failed or produced an empty file." -ForegroundColor Red
    Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
    exit 1
}

# Move to installation directory
Write-Host "Installing GOA CLI to $binPath..." -ForegroundColor Yellow
try {
    Move-Item -Path $tempFile -Destination $binPath -Force
}
catch {
    Write-Host "Error: Failed to install the GOA CLI binary." -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
    exit 1
}

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding GOA CLI to your PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    $env:Path = "$env:Path;$installDir"
    Write-Host "PATH updated! You may need to restart your terminal for the changes to take effect." -ForegroundColor Yellow
}

Write-Host
if ($isUpdate) {
    Write-Host "GOA CLI has been successfully updated!" -ForegroundColor Green
} else {
    Write-Host "GOA CLI has been successfully installed!" -ForegroundColor Green
}
Write-Host "You can now use the 'goa' command from your terminal." -ForegroundColor Green
Write-Host "For help, run: goa --help" -ForegroundColor Green
Write-Host
Write-Host "Thank you for using Go on Airplanes!" -ForegroundColor Cyan 