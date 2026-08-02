param([switch]$NoWriteEvidence)

$ErrorActionPreference='Stop'
$workspace=(Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$specPath=Join-Path $workspace 'Video Localization/specifications/VID-SPEC-004-media-ingest-threat-model.md'
$registerPath=Join-Path $workspace '_shared/Plan/sections/09_SPECIFICATION_AND_DECISION_REGISTER.md'
$fixturePath=Join-Path $PSScriptRoot 'fixtures/hostile-ingest-cases.json'
$evidencePath=Join-Path $workspace 'Video Localization/specifications/evidence/VID-SPEC-004/VID-EVID-004-reference-result.json'
$spec=Get-Content $specPath -Raw
$register=Get-Content $registerPath -Raw
$fixture=Get-Content $fixturePath -Raw|ConvertFrom-Json
$links=[regex]::Matches($spec,'\[[^\]]+\]\(([^)]+)\)')|ForEach-Object{$_.Groups[1].Value}|Where-Object{$_ -notmatch '^https?://'}
$missing=@($links|Where-Object{-not(Test-Path(Join-Path(Split-Path $specPath) $_))})
$threatRows=@($spec -split "`r?`n"|Where-Object{$_ -match '^\| VID-MEDIA-THR-\d{3} \|'})
$gateIds=1..12|ForEach-Object{'VID-MEDIA-SEC-{0:D3}' -f $_}
$failureCodes=@('VID-INGEST-UNSUPPORTED','VID-INGEST-MALFORMED','VID-INGEST-LIMIT','VID-INGEST-PROTECTED','VID-INGEST-QUARANTINED','VID-INGEST-CANCELLED','VID-INGEST-WORKER-FAILED','VID-INGEST-IDENTITY-MISMATCH','VID-INGEST-REVIEW')
$families=@('container','timestamp','protocol','path','archive','subtitle','font','optical','worker','publication')
$ids=@($fixture.cases|ForEach-Object id)
$expected=@('reject','limit','unsupported','review','safe-structured','quarantine','data-only','protected','terminate')
$checks=[ordered]@{
 one_h1=([regex]::Matches($spec,'(?m)^# ')).Count -eq 1
 candidate_status=$spec -match '\| Status \| In Review \|' -and $spec -match '\| Revision \| `1\.0-review\.1` \|'
 dependency_frozen=$register -match '(?m)^\| SUI-SPEC-008 .*\| Frozen \|\r?$'
 local_links_resolve=$missing.Count -eq 0
 no_placeholders=$spec -notmatch 'TODO|TBD|FIXME'
 trust_boundaries=(1..6|Where-Object{$spec -notmatch ('(?m)^'+$_+'\. ')}).Count -eq 0
 security_limits=@('2 TiB','1 MiB','32,768','768 kHz','1,000 hours','10 GiB','2 s cooperative / 5 s total'|Where-Object{$spec -notmatch [regex]::Escape($_)}).Count -eq 0
 threat_inventory=$threatRows.Count -eq 12
 threat_mapping=@($threatRows|Where-Object{($_.Trim('|').Split('|')).Count -lt 6}).Count -eq 0
 failure_contract=@($failureCodes|Where-Object{$spec -notmatch [regex]::Escape($_)}).Count -eq 0
 gate_inventory=@($gateIds|Where-Object{$spec -notmatch [regex]::Escape($_)}).Count -eq 0
 unsafe_defaults=$spec -match 'Shell invocation and concatenated command strings are prohibited' -and $spec -match 'DRM, encryption, region control, copy protection, access control, and signature protection are never bypassed'
 timebase_safety=$spec -match 'MUST NOT derive durable time from nominal FPS' -and $spec -match 'checked arithmetic before allocation'
 fixture_coverage=$fixture.classification -eq 'synthetic' -and $fixture.redistributable -eq $true -and @($fixture.cases).Count -eq 40 -and @($families|Where-Object{$_ -notin $fixture.families}).Count -eq 0
 fixture_integrity=$ids.Count -eq @($ids|Sort-Object -Unique).Count -and @($fixture.cases|Where-Object{$_.expected -notin $expected}).Count -eq 0
}
$failed=@($checks.GetEnumerator()|Where-Object{-not $_.Value}|ForEach-Object Name)
$result=[ordered]@{evidence_id='VID-EVID-004';revision='1.0.0-review.1';status=$(if($failed.Count){'Fail'}else{'Pass'});specification=[ordered]@{id='VID-SPEC-004';revision='1.0-review.1';sha256=(Get-FileHash $specPath -Algorithm SHA256).Hash};dependency=[ordered]@{id='SUI-SPEC-008';revision='1.0';status='Frozen'};fixture_manifest=[ordered]@{id=$fixture.manifest_id;sha256=(Get-FileHash $fixturePath -Algorithm SHA256).Hash;families=@($fixture.families).Count;cases=@($fixture.cases).Count};checks=$checks;summary=[ordered]@{passed=$checks.Count-$failed.Count;total=$checks.Count;failures=$failed};limitations=@('planning-control-conformance-only','synthetic-fixtures-only','no-parser-or-decoder-runtime','no-sandbox-enforcement-evidence','no-format-support-claim')}
$json=$result|ConvertTo-Json -Depth 12
if(-not $NoWriteEvidence){New-Item -ItemType Directory -Force -Path(Split-Path $evidencePath)|Out-Null;[IO.File]::WriteAllText($evidencePath,$json+"`n",[Text.UTF8Encoding]::new($false))}
$json
if($failed.Count){exit 1}
