# Piper TTS Binaries

This directory contains platform-specific Piper TTS binaries and their required shared libraries.

## Contents

### Linux (x86_64)
- `piper-linux-x86_64` - Main Piper executable
- `libpiper_phonemize.so*` - Phonemization library (required)
- `libonnxruntime.so*` - ONNX Runtime library (required)
- `libespeak-ng.so*` - eSpeak NG library (required)

### macOS
- `piper-macos-x86_64` - macOS Intel binary
- `piper-macos-arm64` - macOS Apple Silicon binary

### Windows
- `piper-windows-x86_64.exe` - Windows executable

## Library Dependencies

The Linux binary requires the `.so` (shared object) files to be in the same directory or accessible via `LD_LIBRARY_PATH`. The application automatically sets this environment variable when spawning the Piper subprocess.

## Source

Binaries obtained from: https://github.com/rhasspy/piper/releases

## License

Piper TTS is licensed under the MIT License.
