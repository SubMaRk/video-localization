# VID-IMPL-P00-002B2

Scope: real synthetic Matroska probe evidence for `VID-DEC-003@1.0` with enforced Windows containment and executable identity checks.

## Synthetic fixture evidence (rights-safe, local-only)

- Fixture path (not committed): `C:\Users\SubMark\AppData\Local\Temp\vid-impl-002b2-synthetic\fixture.mkv`
- Evidence identity file (committed only): `synthetic-matroska-fixture.json`
- One leased staged input is used as the live probe target: `fixture.mkv` under scope for this evidence run.
- File identity:
  - `size_bytes`: `218044`
  - `sha256`: `53a4c836ee5e20a470416069d3dfe43d898ff1ed5dceabe417bbf302b1f79782`
- Generator command (host-local, no remote fetch of media):

```powershell
$fixtureDir = "C:\Users\SubMark\AppData\Local\Temp\vid-impl-002b2-synthetic"
New-Item -ItemType Directory -Path $fixtureDir -Force | Out-Null
$fixture = Join-Path $fixtureDir 'fixture.mkv'
$subtitle = Join-Path $fixtureDir 'fixture.srt'
@"1
00:00:00,000 --> 00:00:01,000
Synthetic subtitle for containment proof
"@ | Set-Content -NoNewline $subtitle
& 'C:\ProgramData\chocolatey\lib\ffmpeg-full\tools\ffmpeg\bin\ffmpeg.exe' -y -loglevel error -f lavfi -i testsrc=size=64x64:rate=24:d=1 -c:v ffv1 -f lavfi -i sine=frequency=440:duration=1 -i $subtitle -c:v ffv1 -map 0:v -map 1:a -map 2:s:0 -c:a pcm_s16le -ar 48000 -ac 2 -c:s subrip -shortest $fixture
```

- Verified with fixture probe command:

```powershell
& 'C:\ProgramData\chocolatey\lib\ffmpeg-full\tools\ffmpeg\bin\ffprobe.exe' -v error -hide_banner -protocol_whitelist file -format_whitelist matroska -show_format -show_streams -show_chapters -of json $fixture
```

Observed fixture profile:

- `format_name_raw`: `matroska,webm` (raw ffprobe metadata)
- `format_name_qualified`: `matroska` (profile filter)
- `webm_qualified_or_supported`: `false`
- `duration`: `1.000000`
- `nb_streams`: `3`
- stream codecs: `ffv1`, `pcm_s16le`, `subrip`
- stream types: `video`, `audio`, `subtitle`

## 002A staging, pre/post identity, and fixed profile

- `ProbePrelaunchRequest` is prepared from 002A identity output and must carry:
  - `staged_copy_identity`
  - `staged_copy_sha256`
  - `staged_copy_length`
  - `observed_fence_token`
- Runtime re-check enforces post-launch source re-fingerprint before publication.
- Any prelaunch source identity mismatch after staging is treated as fail-closed (`SourceMutationDetected`).
- Publication identity is checked against context identity and request fence (`publication_fence_token`).

## Exact executable and arg policy

- Registered executable path: `C:\ProgramData\chocolatey\lib\ffmpeg-full\tools\ffmpeg\bin\ffprobe.exe`
- Registered executable SHA-256: `9df3b0b5275e830961df6d94e1f7a71121a7abd5ff708e9fec8a0b6084a55015`
- `ffprobe -version` first line:
  `ffprobe version 8.1.2-full_build-www.gyan.dev Copyright (c) 2007-2026 the FFmpeg developers`
- `ffprobe` configuration line captured exactly:
  `configuration: --enable-gpl --enable-version3 --enable-static --disable-w32threads --disable-autodetect --enable-cairo --enable-fontconfig --enable-iconv --enable-gnutls --enable-lcms2 --enable-libxml2 --enable-gmp --enable-bzlib --enable-lzma --enable-libsnappy --enable-zlib --enable-librist --enable-libsrt --enable-libssh --enable-libzmq --enable-avisynth --enable-libbluray --enable-libcaca --enable-libdvdnav --enable-libdvdread --enable-sdl2 --enable-libaribb24 --enable-libaribcaption --enable-libdav1d --enable-libdavs2 --enable-libopenjpeg --enable-libquirc --enable-libuavs3d --enable-libxevd --enable-libzvbi --enable-liboapv --enable-libqrencode --enable-librav1e --enable-libsvtav1 --enable-libvvenc --enable-libwebp --enable-libx264 --enable-libx265 --enable-libxavs2 --enable-libxeve --enable-libxvid --enable-libaom --enable-libjxl --enable-libsvtjpegxs --enable-libvpx --enable-mediafoundation --enable-libass --enable-frei0r --enable-libfreetype --enable-libfribidi --enable-libharfbuzz --enable-liblensfun --enable-libvidstab --enable-libvmaf --enable-libzimg --enable-amf --enable-cuda-llvm --enable-cuvid --enable-dxva2 --enable-d3d11va --enable-d3d12va --enable-ffnvcodec --enable-libvpl --enable-nvdec --enable-nvenc --enable-vaapi --enable-libshaderc --enable-vulkan --enable-libplacebo --enable-opencl --enable-libcdio --enable-openal --enable-libgme --enable-libmodplug --enable-libopenmpt --enable-libopencore-amrwb --enable-libmp3lame --enable-libshine --enable-libtheora --enable-libtwolame --enable-libvo-amrwbenc --enable-libcodec2 --enable-libilbc --enable-libgsm --enable-liblc3 --enable-libopencore-amrnb --enable-libopus --enable-libspeex --enable-libvorbis --enable-ladspa --enable-libbs2b --enable-libflite --enable-libmysofa --enable-librubberband --enable-libsoxr --enable-chromaprint --enable-whisper`

## Containment and bounded-output controls

- Network containment: live `DefaultNetworkDenialController` creates and verifies inbound/outbound `netsh` deny rules before launch.
- Job object containment: live `DefaultJobObjectController` creates limits and assigns the spawned ffprobe process.
- Watchdog containment: live monotonic wall-clock limit uses `CONTROL_LIMITS.wall_clock_seconds` (`60`) with cancellation checks.
- Fixed argv: exact `fixed_probe_argv` (`-v error -hide_banner -protocol_whitelist file -format_whitelist matroska -show_format -show_streams -show_chapters -of json <path>`).
- Bounded decode capture:
  - stdout: `1_048_576` bytes
  - stderr: `1_048_576` bytes
  - stream ceiling: `256`

## Negative paths under implementation

- executable mismatch
- launch-argv mismatch
- stale lease
- publication-fence mismatch
- source prelaunch mutation
- network containment unavailable
- job object unavailable
- watchdog unavailable
- process crash
- malformed/oversized stdout JSON
- parse stream-ceiling overflow
- parse required-field gaps
- timeout and cancellation path

## Live evidence files

- `synthetic-matroska-fixture.json`: fixture identity + ffprobe identity evidence
- `manifest.json`: artifact inventory and evidence scope
- `test-report.json`: command evidence, check table, and fail-closed outcomes
