param([switch]$NoWriteEvidence)
$ErrorActionPreference='Stop'
$workspace=(Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$specPath=Join-Path $workspace 'Video Localization/specifications/VID-SPEC-002-domain-schema.md'
$registerPath=Join-Path $workspace '_shared/Plan/sections/09_SPECIFICATION_AND_DECISION_REGISTER.md'
$fixturePath=Join-Path $PSScriptRoot 'fixtures/domain-schema-cases.json'
$evidencePath=Join-Path $workspace 'Video Localization/specifications/evidence/VID-SPEC-002/VID-EVID-002-reference-result.json'
$spec=Get-Content $specPath -Raw;$register=Get-Content $registerPath -Raw;$fixture=Get-Content $fixturePath -Raw|ConvertFrom-Json
$records=@('Video Project Record','Collection Record','Media Asset and Source Authority','Track Record','Cue Record','Word Record','Speaker Record','Annotation Record','Observation and Proposal Record','Video Revision and Lineage','Artifact and Delivery References')
$gates=1..12|ForEach-Object{'VID-DOM-{0:D3}' -f $_};$families=@('identity','references','media-authority','timed-text','lineage','observation-approval','migration-recovery','security-boundary')
$links=[regex]::Matches($spec,'\[[^\]]+\]\(([^)]+)\)')|ForEach-Object{$_.Groups[1].Value}|Where-Object{$_ -notmatch '^https?://'};$missing=@($links|Where-Object{-not(Test-Path (Join-Path (Split-Path $specPath) $_))});$ids=@($fixture.cases|ForEach-Object id)
$checks=[ordered]@{
 one_h1=(([regex]::Matches($spec,'(?m)^# ')).Count -eq 1)
 candidate_status=($spec -match '\| Status \| In Review \|' -and $spec -match '\| Revision \| `1\.0-review\.1` \|')
 dependencies_frozen=($register -match '(?m)^\| SUI-SPEC-001 .*\| Frozen \|$' -and $register -match '(?m)^\| VID-SPEC-001 .*\| Frozen \|$')
 namespace_bound=($spec -match 'submark\.video-localization')
 record_inventory=(@($records|Where-Object{$spec -notmatch [regex]::Escape("## $_")}).Count -eq 0)
 exact_gate_inventory=(@($gates|Where-Object{$spec -notmatch [regex]::Escape($_)}).Count -eq 0)
 normative_coverage=(([regex]::Matches($spec,'\bMUST(?: NOT)?\b')).Count -ge 70)
 local_links_resolve=($missing.Count -eq 0)
 no_placeholders=($spec -notmatch 'TODO|TBD|FIXME')
 fixture_coverage=($fixture.classification -eq 'synthetic' -and $fixture.redistributable -eq $true -and @($fixture.cases).Count -ge 32 -and @($families|Where-Object{$_ -notin $fixture.families}).Count -eq 0)
 fixture_integrity=($ids.Count -eq @($ids|Sort-Object -Unique).Count)
 authority_boundary=($spec -match 'Shared Core MUST NOT infer cue timing' -and $spec -match 'worker.*MUST NOT directly mutate authoritative Video state')
 observation_boundary=($spec -match 'Successful execution MUST NOT imply human approval' -and $spec -match 'Authority class fixed as non-authoritative')
 timing_boundary=($spec -match 'MUST NOT be derived from `frame_index / nominal_fps`' -and $spec -match 'belongs to `VID-SPEC-003`')
 product_isolation=($spec -match 'Video MUST NOT import Manga or Document domain schemas')
}
$failed=@($checks.GetEnumerator()|Where-Object{-not $_.Value}|ForEach-Object Name);$result=[ordered]@{evidence_id='VID-EVID-002';revision='1.0.0-review.1';status=$(if($failed.Count){'Fail'}else{'Pass'});specification=[ordered]@{id='VID-SPEC-002';revision='1.0-review.1';sha256=(Get-FileHash $specPath -Algorithm SHA256).Hash};dependencies=@([ordered]@{id='SUI-SPEC-001';revision='1.0';status='Frozen'},[ordered]@{id='VID-SPEC-001';revision='1.0';status='Frozen'});fixture_manifest=[ordered]@{id=$fixture.manifest_id;sha256=(Get-FileHash $fixturePath -Algorithm SHA256).Hash;families=@($fixture.families).Count;cases=@($fixture.cases).Count};checks=$checks;summary=[ordered]@{passed=$checks.Count-$failed.Count;total=$checks.Count;failures=$failed};limitations=@('no-native-database','no-media-library','no-timebase-math','no-hostile-ingest-runtime','no-crash-migration-run','no-product-adapter-run')}
$json=$result|ConvertTo-Json -Depth 12;if(-not $NoWriteEvidence){New-Item -ItemType Directory -Force -Path (Split-Path $evidencePath)|Out-Null;[IO.File]::WriteAllText($evidencePath,$json+"`n",[Text.UTF8Encoding]::new($false))};$json;if($failed.Count){exit 1}
