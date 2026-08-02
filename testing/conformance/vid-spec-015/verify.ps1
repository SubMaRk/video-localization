param([switch]$NoWriteEvidence)

$ErrorActionPreference='Stop'
$workspace=(Resolve-Path(Join-Path $PSScriptRoot '../../../..')).Path
$specPath=Join-Path $workspace 'Video Localization/specifications/VID-SPEC-015-foundation-verification.md'
$registerPath=Join-Path $workspace '_shared/Plan/sections/09_SPECIFICATION_AND_DECISION_REGISTER.md'
$environmentPath=Join-Path $PSScriptRoot 'profiles/environment-matrix.json'
$budgetPath=Join-Path $PSScriptRoot 'profiles/foundation-budgets.json'
$corpusPath=Join-Path $PSScriptRoot 'fixtures/foundation-corpus.json'
$faultPath=Join-Path $PSScriptRoot 'fixtures/fault-matrix.json'
$evidencePath=Join-Path $workspace 'Video Localization/specifications/evidence/VID-SPEC-015/VID-EVID-015-entry-result.json'
$spec=Get-Content $specPath -Raw;$register=Get-Content $registerPath -Raw
$environment=Get-Content $environmentPath -Raw|ConvertFrom-Json;$budgets=Get-Content $budgetPath -Raw|ConvertFrom-Json;$corpus=Get-Content $corpusPath -Raw|ConvertFrom-Json;$faults=Get-Content $faultPath -Raw|ConvertFrom-Json
$links=[regex]::Matches($spec,'\[[^\]]+\]\(([^)]+)\)')|ForEach-Object{$_.Groups[1].Value}|Where-Object{$_-notmatch'^https?://'};$missing=@($links|Where-Object{-not(Test-Path(Join-Path(Split-Path $specPath)$_))})
$deps=@('VID-SPEC-001','VID-SPEC-002','VID-SPEC-003','VID-SPEC-004','VID-SPEC-014')
$gates=1..14|ForEach-Object{'VID-VERIFY-{0:D3}'-f$_}
$workloads=@('VID-WL-PROJECT-SMALL','VID-WL-PROJECT-FOUNDATION','VID-WL-MEDIA-CFR','VID-WL-MEDIA-VFR','VID-WL-WAVEFORM','VID-WL-LOCALE','VID-WL-RECOVERY')
$requiredFaults=@('worker-crash','worker-hang','cancellation','malformed-result','force-close','power-loss-surrogate','disk-full','permission-loss','partial-write','damaged-cache','migration-failure','missing-media','relink-mismatch','invalid-ipc','memory-exhaustion','unsafe-protocol','stale-worker','antivirus-interference')
$envIds=@($environment.profiles|ForEach-Object id);$budgetIds=@($budgets.budgets|ForEach-Object id);$corpusIds=@($corpus.fixtures|ForEach-Object id);$faultIds=@($faults.cases|ForEach-Object id)
$base=@($environment.profiles|Where-Object id -eq 'WIN11-X64-BASE')[0];$low=@($environment.profiles|Where-Object id -eq 'WIN11-X64-LOW')[0]
$rpo=@($budgets.budgets|Where-Object id -eq 'VID-PERF-015')[0]
$corpusFieldsOk=@($corpus.fixtures|Where-Object{[string]::IsNullOrWhiteSpace($_.id)-or[string]::IsNullOrWhiteSpace($_.family)-or[string]::IsNullOrWhiteSpace($_.generator_or_source)-or[string]::IsNullOrWhiteSpace($_.rights_basis)-or$null-eq$_.redistributable-or[string]::IsNullOrWhiteSpace($_.privacy)-or[string]::IsNullOrWhiteSpace($_.materialization)-or[string]::IsNullOrWhiteSpace($_.expected)-or[string]::IsNullOrWhiteSpace($_.gate)}).Count-eq0
$checks=[ordered]@{
 one_h1=([regex]::Matches($spec,'(?m)^# ')).Count-eq1
 candidate_status=$spec-match'\| Status \| In Review \|'-and$spec-match'\| Revision \| `1\.0-review\.1` \|'
 dependencies_frozen=@($deps|Where-Object{$register-notmatch('(?m)^\| '+[regex]::Escape($_)+' .*\| Frozen \|\r?$')}).Count-eq0
 local_links_resolve=$missing.Count-eq0
 no_placeholders=$spec-notmatch'TODO|TBD|FIXME'
 evidence_levels=@('Entry','Implemented','Verified','Exit'|Where-Object{$spec-notmatch('\| `'+$_+'` \|')}).Count-eq0-and$spec-match'Synthetic planning checks MUST NOT be labeled runtime verification'
 release_allocation=$spec-match'v0\.1\.0-alpha'-and$spec-match'one later-promoted baseline media profile'-and$spec-match'Professional editor completeness, broad formats, proxy/burn-in, AI, Teams, plugins, visual text, public distribution'
 environment_matrix=@($environment.profiles).Count-eq3-and$envIds.Count-eq@($envIds|Sort-Object -Unique).Count-and$base.architecture-eq'x64'-and$base.ram_gib-eq16-and-not$base.network_required-and$low.required-eq'safe-failure-only'
 environment_axes=@('en','th'|Where-Object{$_-notin$environment.axes.locales}).Count-eq0-and@('System','Light','Dark','High Contrast'|Where-Object{$_-notin$environment.axes.themes}).Count-eq0-and@(100,150,200|Where-Object{$_-notin$environment.axes.scale_percent}).Count-eq0
 budget_contract=@($budgets.budgets).Count-eq15-and$budgetIds.Count-eq@($budgetIds|Sort-Object -Unique).Count-and$budgets.method.warm_iterations-eq30-and$budgets.method.cold_start_iterations-eq10-and$rpo.threshold-eq1-and$rpo.unit-eq'commands'
 workload_inventory=@($workloads|Where-Object{$spec-notmatch[regex]::Escape($_)}).Count-eq0
 corpus_governance=@($corpus.fixtures).Count-eq16-and$corpusIds.Count-eq@($corpusIds|Sort-Object -Unique).Count-and$corpusFieldsOk-and@($corpus.fixtures|Where-Object{$_.privacy-ne'public-test'-or-not$_.redistributable-or$_.materialization-notin@('planned','present')}).Count-eq0
 private_asset_deny=$spec-match'Private customer media, copyrighted commercial media, restricted subtitle/font/model payloads, credentials, and personal data MUST NOT enter the repository corpus'
 fault_matrix=@($faults.cases).Count-eq18-and$faultIds.Count-eq@($faultIds|Sort-Object -Unique).Count-and@($requiredFaults|Where-Object{$_-notin$faults.cases.class}).Count-eq0
 migration_recovery=$spec-match'failed migration with the original remaining readable'-and$spec-match'rejection of unsupported future schema without mutation'-and$spec-match'Acknowledged user commands survive force close'
 interoperability_boundary=$spec-match'UTF-8 SRT output'-and$spec-match'No player/editor interoperability claim exists until the exact third-party application/version'
 exit_evidence=(1..12|Where-Object{$spec-notmatch('(?m)^'+$_+'\. ')}).Count-eq0
 gate_inventory=@($gates|Where-Object{$spec-notmatch[regex]::Escape($_)}).Count-eq0
 entry_exit_separation=$spec-match'Freezing this contract fixes profiles, thresholds, corpus governance, procedures, and required evidence; it does not satisfy `Exit:V0`'-and$spec-match'Exit approval remains pending until implementation evidence exists'
}
$failed=@($checks.GetEnumerator()|Where-Object{-not$_.Value}|ForEach-Object Name)
$result=[ordered]@{evidence_id='VID-EVID-015-ENTRY';revision='1.0.0-review.1';status=$(if($failed.Count){'Fail'}else{'Pass'});classification='entry-contract-conformance-only';specification=[ordered]@{id='VID-SPEC-015';revision='1.0-review.1';sha256=(Get-FileHash $specPath -Algorithm SHA256).Hash};assets=[ordered]@{environment=[ordered]@{sha256=(Get-FileHash $environmentPath -Algorithm SHA256).Hash;profiles=@($environment.profiles).Count};budgets=[ordered]@{sha256=(Get-FileHash $budgetPath -Algorithm SHA256).Hash;metrics=@($budgets.budgets).Count};corpus=[ordered]@{sha256=(Get-FileHash $corpusPath -Algorithm SHA256).Hash;records=@($corpus.fixtures).Count;status=$corpus.status};faults=[ordered]@{sha256=(Get-FileHash $faultPath -Algorithm SHA256).Hash;cases=@($faults.cases).Count}};checks=$checks;summary=[ordered]@{passed=$checks.Count-$failed.Count;total=$checks.Count;failures=$failed};exit_status='Pending implementation evidence';limitations=@('planned-generators-are-not-materialized-fixtures','no-runtime-performance-evidence','no-runtime-recovery-evidence','no-runtime-interoperability-evidence','no-release-support-claim')}
$json=$result|ConvertTo-Json -Depth 12;if(-not$NoWriteEvidence){New-Item -ItemType Directory -Force -Path(Split-Path $evidencePath)|Out-Null;[IO.File]::WriteAllText($evidencePath,$json+"`n",[Text.UTF8Encoding]::new($false))};$json;if($failed.Count){exit 1}
