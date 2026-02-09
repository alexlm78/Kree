# CI/CD con GitHub Actions

Este documento describe la configuración necesaria para implementar un pipeline de integración y despliegue continuo (CI/CD) utilizando GitHub Actions para el proyecto Kree.

El pipeline automatizado incluye:
1.  **Build**: Compilación del proyecto en Linux, macOS y Windows (incluyendo soporte para ARM64).
2.  **Test**: Ejecución de pruebas unitarias y de integración.
3.  **Linting**: Análisis estático con `clippy` y verificación de formato con `rustfmt`.
4.  **Release Automatizado**: Creación de releases en GitHub y subida de binarios compilados cuando se pushea un tag (v*).

## Archivo de Workflow

El archivo de configuración `.github/workflows/ci.yml` define los siguientes trabajos:

```yaml
name: CI/CD

on:
  push:
    branches: [ "main" ]
    tags: [ "v*" ]
  pull_request:
    branches: [ "main" ]

env:
  CARGO_TERM_COLOR: always

jobs:
  # Job de verificación y pruebas
  check-and-test:
    name: Check and Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache Cargo dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Check Formatting
        run: cargo fmt --all -- --check

      - name: Clippy Linting
        run: cargo clippy -- -D warnings

      - name: Run Tests
        run: cargo test --verbose

  # Job de construcción de binarios para release
  build-release:
    name: Build Release Binary
    needs: check-and-test
    if: startsWith(github.ref, 'refs/tags/v')
    strategy:
      matrix:
        include:
          # Linux x64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: kree
            asset_name: kree-linux-amd64
          # macOS x64 (Intel)
          - os: macos-13
            target: x86_64-apple-darwin
            artifact_name: kree
            asset_name: kree-macos-amd64
          # macOS ARM64 (Apple Silicon)
          - os: macos-14
            target: aarch64-apple-darwin
            artifact_name: kree
            asset_name: kree-macos-arm64
          # Windows x64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: kree.exe
            asset_name: kree-windows-amd64.exe
          # Windows ARM64
          - os: windows-latest
            target: aarch64-pc-windows-msvc
            artifact_name: kree.exe
            asset_name: kree-windows-arm64.exe

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build Binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.asset_name }}
          path: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}

  # Job de publicación del Release en GitHub
  create-release:
    name: Create GitHub Release
    needs: build-release
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    permissions:
      contents: write
    
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Display structure of downloaded files
        run: ls -R artifacts

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          draft: false
          prerelease: false
          generate_release_notes: true
```

## Instrucciones de uso

1.  El archivo de workflow ya se encuentra en `.github/workflows/ci.yml`.
2.  Cada vez que se haga push a `main`, se ejecutarán los tests.
3.  Para generar un release con binarios para todas las plataformas (incluyendo ARM64), crea un tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```
