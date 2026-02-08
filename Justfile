set shell := ["powershell.exe", "-NoProfile", "-Command"]

APP := "lfs_stt"

VERSION := `
    cargo metadata --no-deps --format-version 1 |
    ConvertFrom-Json |
    Select-Object -ExpandProperty packages |
    Select-Object -First 1 -ExpandProperty version
`

DIST := "dist/" + APP + "-" + VERSION

release:
    cargo build --release

    if (Test-Path dist) { Remove-Item -Recurse -Force dist }
    New-Item -ItemType Directory -Force {{DIST}}/models | Out-Null

    Copy-Item target\release\{{APP}}.exe {{DIST}}
    Copy-Item LICENSE {{DIST}}
    Copy-Item config.example.toml {{DIST}}/config.toml -Recurse
    Copy-Item models/small.en.bin {{DIST}}/models/small.en.bin -Recurse

    Compress-Archive -Path {{DIST}} -DestinationPath dist\{{APP}}-{{VERSION}}-small-en.zip -Force

    Remove-Item -Recurse -Force {{DIST}}/models/small.en.bin
    Compress-Archive -Path {{DIST}} -DestinationPath dist\{{APP}}-{{VERSION}}.zip -Force
