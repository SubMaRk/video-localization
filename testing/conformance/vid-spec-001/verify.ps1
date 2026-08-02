param(
    [string]$OutputPath = "Video Localization/specifications/evidence/VID-SPEC-001/VID-EVID-001-traceability-result.json"
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$specPath = Join-Path $repoRoot "Video Localization\specifications\VID-SPEC-001-foundation-prd.md"
$registerPath = Join-Path $repoRoot "_shared\Plan\sections\09_SPECIFICATION_AND_DECISION_REGISTER.md"
$dependencyPath = Join-Path $repoRoot "_shared\decisions\SUI-DEC-008-planning-baseline-acceptance.md"
$resolvedOutput = [IO.Path]::GetFullPath((Join-Path $repoRoot $OutputPath))

$spec = Get-Content -Raw -LiteralPath $specPath
$register = Get-Content -Raw -LiteralPath $registerPath
$dependency = Get-Content -Raw -LiteralPath $dependencyPath
$checks = [Collections.Generic.List[object]]::new()

function Add-Check {
    param(
        [string]$Id,
        [bool]$Passed,
        [string]$Evidence
    )

    $checks.Add([ordered]@{
        id = $Id
        status = if ($Passed) { "Pass" } else { "Fail" }
        evidence = $Evidence
    })
}

$h1Matches = [regex]::Matches($spec, '(?m)^# ')
Add-Check "structure.one-h1" ($h1Matches.Count -eq 1) "Found $($h1Matches.Count) H1 heading(s); expected 1."

$statusMatch = [regex]::Match($spec, '(?m)^\| Status \| (?<value>[^|]+) \|$')
$revisionMatch = [regex]::Match($spec, '(?m)^\| Revision \| `(?<value>[^`]+)` \|$')
Add-Check "metadata.status" ($statusMatch.Success -and $statusMatch.Groups['value'].Value.Trim() -eq 'Drafting') "Expected Drafting metadata status."
Add-Check "metadata.revision" ($revisionMatch.Success -and $revisionMatch.Groups['value'].Value -eq '0.1-draft.1') "Expected revision 0.1-draft.1."

$requirementMatches = [regex]::Matches($spec, '(?m)^\| `(?<id>VID-PRD-F[01]-\d{3})` \| (?<requirement>.*?) \| (?<refs>.*?) \|$')
$requirementIds = @($requirementMatches | ForEach-Object { $_.Groups['id'].Value })
$uniqueRequirementIds = @($requirementIds | Sort-Object -Unique)
$expectedF0 = @(1..20 | ForEach-Object { 'VID-PRD-F0-{0:D3}' -f $_ })
$expectedF1 = @(1..15 | ForEach-Object { 'VID-PRD-F1-{0:D3}' -f $_ })
$expectedRequirements = @($expectedF0 + $expectedF1)
$missingRequirements = @($expectedRequirements | Where-Object { $_ -notin $uniqueRequirementIds })
$unexpectedRequirements = @($uniqueRequirementIds | Where-Object { $_ -notin $expectedRequirements })
Add-Check "requirements.count" ($requirementIds.Count -eq 35) "Found $($requirementIds.Count) requirement rows; expected 35."
Add-Check "requirements.unique" ($requirementIds.Count -eq $uniqueRequirementIds.Count) "Found $($uniqueRequirementIds.Count) unique IDs from $($requirementIds.Count) rows."
Add-Check "requirements.sequence" ($missingRequirements.Count -eq 0 -and $unexpectedRequirements.Count -eq 0) "Missing: $($missingRequirements -join ', '); unexpected: $($unexpectedRequirements -join ', ')."

$registeredIds = @([regex]::Matches($register, '(?m)^\| (?<id>(?:VID|SUI)-(?:SPEC|DEC)-\d{3}(?:-[A-Z])?) \|') | ForEach-Object { $_.Groups['id'].Value } | Sort-Object -Unique)
$referencedIds = @($requirementMatches | ForEach-Object {
    [regex]::Matches($_.Groups['refs'].Value, '(?:VID|SUI)-(?:SPEC|DEC)-\d{3}(?:-[A-Z])?') | ForEach-Object { $_.Value }
} | Sort-Object -Unique)
$unregisteredReferences = @($referencedIds | Where-Object { $_ -notin $registeredIds })
Add-Check "requirements.registered-references" ($unregisteredReferences.Count -eq 0) "Unregistered downstream references: $($unregisteredReferences -join ', ')."

$laterPhaseRefs = @('VID-SPEC-007', 'VID-SPEC-008', 'VID-SPEC-009', 'VID-SPEC-010', 'VID-SPEC-011', 'VID-SPEC-012', 'VID-SPEC-013')
$leakedLaterRefs = @($referencedIds | Where-Object { $_ -in $laterPhaseRefs })
Add-Check "scope.no-later-phase-reference" ($leakedLaterRefs.Count -eq 0) "Later-phase references in Phase 00/01 requirement mappings: $($leakedLaterRefs -join ', ')."

$gateMatches = [regex]::Matches($spec, '(?m)^\| `(?<id>VID-PRD-GATE-\d{3})` \|')
$gateIds = @($gateMatches | ForEach-Object { $_.Groups['id'].Value })
$expectedGates = @(1..7 | ForEach-Object { 'VID-PRD-GATE-{0:D3}' -f $_ })
$missingGates = @($expectedGates | Where-Object { $_ -notin $gateIds })
Add-Check "gates.complete" ($gateIds.Count -eq 7 -and $missingGates.Count -eq 0) "Found $($gateIds.Count) gates; missing: $($missingGates -join ', ')."

$registerDrafting = $register -match '(?m)^\| VID-SPEC-001 \|.*\| Drafting \|$'
Add-Check "register.status" $registerDrafting "VID-SPEC-001 register row must be Drafting for this candidate."

$dependencyAccepted = $dependency -match '(?m)^Status: Accepted\s*$' -and $dependency -match '(?m)^Revision: 1\.0\s*$'
Add-Check "dependency.sui-dec-008" $dependencyAccepted "SUI-DEC-008 must be Accepted revision 1.0."

$linkMatches = [regex]::Matches($spec, '\[[^\]]+\]\((?<target>[^)]+)\)')
$missingLinks = [Collections.Generic.List[string]]::new()
$specDirectory = Split-Path -Parent $specPath
foreach ($match in $linkMatches) {
    $target = $match.Groups['target'].Value.Trim().Trim('<', '>')
    if ($target -match '^[a-z]+://' -or $target.StartsWith('#')) {
        continue
    }
    $pathPart = $target.Split('#')[0]
    $candidatePath = [IO.Path]::GetFullPath((Join-Path $specDirectory $pathPart))
    if (-not (Test-Path -LiteralPath $candidatePath)) {
        $missingLinks.Add($target)
    }
}
Add-Check "references.local-links" ($missingLinks.Count -eq 0) "Missing local links: $($missingLinks -join ', ')."

$issueLinked = $spec -match 'SubMaRk/video-localization#19'
Add-Check "traceability.github-issue" $issueLinked "Expected GitHub work item SubMaRk/video-localization#19."

$failed = @($checks | Where-Object { $_.status -eq 'Fail' })
$candidateHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $specPath).Hash
$result = [ordered]@{
    evidence_id = "VID-EVID-001"
    evidence_revision = "0.1.0-draft.1"
    status = if ($failed.Count -eq 0) { "Pass" } else { "Fail" }
    candidate = [ordered]@{
        specification_id = "VID-SPEC-001"
        revision = "0.1-draft.1"
        status = "Drafting"
        path = "Video Localization/specifications/VID-SPEC-001-foundation-prd.md"
        sha256 = $candidateHash
    }
    summary = [ordered]@{
        checks = $checks.Count
        passed = $checks.Count - $failed.Count
        failed = $failed.Count
        phase_00_requirements = $expectedF0.Count
        phase_01_requirements = $expectedF1.Count
        acceptance_gates = $expectedGates.Count
        downstream_references = $referencedIds.Count
        local_links = $linkMatches.Count
    }
    checks = $checks
    approval_boundary = "A machine Pass proves structural traceability only. It does not approve, freeze, implement, or verify VID-SPEC-001."
}

$outputDirectory = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Force $outputDirectory | Out-Null
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resolvedOutput -Encoding utf8

Write-Output ("VID-EVID-001 {0}: {1}/{2} checks passed; candidate SHA-256 {3}" -f $result.status, $result.summary.passed, $result.summary.checks, $candidateHash)
if ($failed.Count -gt 0) {
    exit 1
}
