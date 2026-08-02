param([switch]$NoWriteEvidence)

$ErrorActionPreference='Stop'
$workspace=(Resolve-Path(Join-Path $PSScriptRoot '../../../..')).Path
$specPath=Join-Path $workspace 'Video Localization/specifications/VID-SPEC-014-design-and-localization.md'
$registerPath=Join-Path $workspace '_shared/Plan/sections/09_SPECIFICATION_AND_DECISION_REGISTER.md'
$tokenPath=Join-Path $workspace 'Video Localization/design/tokens/vid.tokens.json'
$componentPath=Join-Path $workspace 'Video Localization/design/component-matrix.json'
$enPath=Join-Path $workspace 'Video Localization/localization/en.json'
$thPath=Join-Path $workspace 'Video Localization/localization/th.json'
$termPath=Join-Path $workspace 'Video Localization/localization/terminology.en-th.json'
$fixturePath=Join-Path $PSScriptRoot 'fixtures/design-localization-cases.json'
$evidencePath=Join-Path $workspace 'Video Localization/specifications/evidence/VID-SPEC-014/VID-EVID-014-reference-result.json'
$spec=Get-Content $specPath -Raw;$register=Get-Content $registerPath -Raw
$tokens=Get-Content $tokenPath -Raw|ConvertFrom-Json;$components=Get-Content $componentPath -Raw|ConvertFrom-Json
$en=Get-Content $enPath -Raw|ConvertFrom-Json;$th=Get-Content $thPath -Raw|ConvertFrom-Json;$terms=Get-Content $termPath -Raw|ConvertFrom-Json;$fixture=Get-Content $fixturePath -Raw|ConvertFrom-Json

function Linear([int]$v){$s=$v/255.0;if($s-le0.04045){$s/12.92}else{[Math]::Pow(($s+0.055)/1.055,2.4)}}
function Luminance([string]$hex){$h=$hex.TrimStart('#');0.2126*(Linear([Convert]::ToInt32($h.Substring(0,2),16)))+0.7152*(Linear([Convert]::ToInt32($h.Substring(2,2),16)))+0.0722*(Linear([Convert]::ToInt32($h.Substring(4,2),16)))}
function Contrast([string]$a,[string]$b){$x=Luminance $a;$y=Luminance $b;if($x-lt$y){$t=$x;$x=$y;$y=$t};($x+0.05)/($y+0.05)}
function Placeholders([string]$s){@([regex]::Matches($s,'\{[A-Za-z][A-Za-z0-9_]*\}')|ForEach-Object Value|Sort-Object -Unique)}

$contrast=[ordered]@{}
foreach($theme in @('light','dark')){
 $c=$tokens.primitive.color.$theme
 $contrast["$theme.text"]=(Contrast $c.text $c.canvas)-ge4.5
 $contrast["$theme.muted"]=(Contrast $c.muted $c.canvas)-ge4.5
 $contrast["$theme.action"]=(Contrast $c.action $c.canvas)-ge4.5
 $contrast["$theme.onAction"]=(Contrast $c.actionText $c.action)-ge4.5
 $contrast["$theme.boundary"]=(Contrast $c.border $c.surface)-ge3.0
 $contrast["$theme.focus"]=(Contrast $c.focus $c.canvas)-ge3.0
 foreach($state in @('success','warning','error','info')){$contrast["$theme.$state"]=(Contrast $c.$state $c.canvas)-ge4.5}
}
$enKeys=@($en.messages.psobject.Properties.Name|Sort-Object);$thKeys=@($th.messages.psobject.Properties.Name|Sort-Object)
$catalogParity=($enKeys -join "`n")-eq($thKeys -join "`n")
$placeholderParity=$true
foreach($key in $enKeys){if(((Placeholders $en.messages.$key.text)-join ',') -ne ((Placeholders $th.messages.$key.text)-join ',')){$placeholderParity=$false;break}}
$catalogComplete=@($enKeys|Where-Object{[string]::IsNullOrWhiteSpace($en.messages.$_.text)-or[string]::IsNullOrWhiteSpace($en.messages.$_.context)-or[string]::IsNullOrWhiteSpace($th.messages.$_.text)-or[string]::IsNullOrWhiteSpace($th.messages.$_.context)}).Count-eq0
$links=[regex]::Matches($spec,'\[[^\]]+\]\(([^)]+)\)')|ForEach-Object{$_.Groups[1].Value}|Where-Object{$_-notmatch'^https?://'};$missing=@($links|Where-Object{-not(Test-Path(Join-Path(Split-Path $specPath)$_))})
$gates=1..12|ForEach-Object{'VID-DESIGN-{0:D3}'-f$_};$families=@('theme','locale','placeholder','thai','bidi','accessibility','ime','layout');$ids=@($fixture.cases|ForEach-Object id)
$componentJson=$tokens.component|ConvertTo-Json -Depth 12
$checks=[ordered]@{
 one_h1=([regex]::Matches($spec,'(?m)^# ')).Count-eq1
 candidate_status=$spec-match'\| Status \| In Review \|'-and$spec-match'\| Revision \| `1\.0-review\.1` \|'
 dependencies_ready=$register-match'(?m)^\| SUI-SPEC-007 .*\| Frozen \|\r?$'-and$register-match'(?m)^\| VID-DEC-004 .*\| Accepted \|\r?$'
 local_links_resolve=$missing.Count-eq0
 no_placeholders=$spec-notmatch'TODO|TBD|FIXME'
 three_layer_tokens=$tokens.schema-eq'submark.design-tokens/1.0'-and$tokens.direction-eq'Cinematic Ledger'-and$componentJson-notmatch'primitive\.'
 theme_contract=$tokens.systemTheme-eq'live-os-light-or-dark'-and$tokens.primitive.color.mediaStage-eq'#0B0B0B'-and$tokens.highContrast-eq'system-color-mapping-required'
 contrast_pairs=@($contrast.GetEnumerator()|Where-Object{-not$_.Value}).Count-eq0
 component_matrix=@($components.components).Count-eq18-and@($components.required_states).Count-eq12-and@($components.components|Where-Object{-not$_.keyboard-or-not$_.accessible-or-not$_.bounded}).Count-eq0
 workspace_matrix=@('edit','timing','translation','review','delivery'|Where-Object{$_-notin$components.workspaces}).Count-eq0-and@($components.components|Where-Object{$_.id-in@('cue-list','timeline','waveform','finding-list','job-queue')-and-not$_.virtualized}).Count-eq0
 catalog_contract=$en.locale-eq'en'-and$null-eq$en.fallback-and$th.locale-eq'th'-and$th.fallback-eq'en'-and$catalogParity-and$placeholderParity-and$catalogComplete
 terminology=@($terms.entries).Count-eq15-and@($terms.entries.id|Sort-Object -Unique).Count-eq15-and@($terms.entries|Where-Object{[string]::IsNullOrWhiteSpace($_.en)-or[string]::IsNullOrWhiteSpace($_.th)-or[string]::IsNullOrWhiteSpace($_.context)}).Count-eq0
 complex_script=$spec-match'Thai, CJK, RTL, mixed script'-and$spec-match'active IME composition MUST NOT invoke editor commands'-and$spec-match'MUST NOT assume whitespace word boundaries'
 gate_inventory=@($gates|Where-Object{$spec-notmatch[regex]::Escape($_)}).Count-eq0
 fixture_coverage=$fixture.classification-eq'synthetic'-and$fixture.redistributable-eq$true-and@($fixture.cases).Count-eq32-and@($families|Where-Object{$_-notin$fixture.families}).Count-eq0
 fixture_integrity=$ids.Count-eq@($ids|Sort-Object -Unique).Count-and@($fixture.cases|Where-Object{[string]::IsNullOrWhiteSpace($_.expected)}).Count-eq0
}
$failed=@($checks.GetEnumerator()|Where-Object{-not$_.Value}|ForEach-Object Name)
$result=[ordered]@{evidence_id='VID-EVID-014';revision='1.0.0-review.1';status=$(if($failed.Count){'Fail'}else{'Pass'});specification=[ordered]@{id='VID-SPEC-014';revision='1.0-review.1';sha256=(Get-FileHash $specPath -Algorithm SHA256).Hash};assets=[ordered]@{tokens=(Get-FileHash $tokenPath -Algorithm SHA256).Hash;components=(Get-FileHash $componentPath -Algorithm SHA256).Hash;english=(Get-FileHash $enPath -Algorithm SHA256).Hash;thai=(Get-FileHash $thPath -Algorithm SHA256).Hash;terminology=(Get-FileHash $termPath -Algorithm SHA256).Hash;fixtures=(Get-FileHash $fixturePath -Algorithm SHA256).Hash};counts=[ordered]@{components=@($components.components).Count;messages=$enKeys.Count;terms=@($terms.entries).Count;fixture_families=@($fixture.families).Count;fixture_cases=@($fixture.cases).Count;contrast_pairs=$contrast.Count};contrast=$contrast;checks=$checks;summary=[ordered]@{passed=$checks.Count-$failed.Count;total=$checks.Count;failures=$failed};limitations=@('no-native-control-evidence','no-rendered-screenshot-evidence','no-font-payload-frozen','no-native-ime-evidence','no-assistive-technology-runtime-evidence','no-ui-performance-evidence')}
$json=$result|ConvertTo-Json -Depth 12;if(-not$NoWriteEvidence){New-Item -ItemType Directory -Force -Path(Split-Path $evidencePath)|Out-Null;[IO.File]::WriteAllText($evidencePath,$json+"`n",[Text.UTF8Encoding]::new($false))};$json;if($failed.Count){exit 1}
