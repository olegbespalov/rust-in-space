#!/usr/bin/env bash
set -euo pipefail

platform=$1
binary=$2
output=$3
tag=$4
work_dir=$(mktemp -d "${TMPDIR:-/tmp}/rust-in-space.XXXXXX")
trap 'rm -rf -- "$work_dir"' EXIT

case "$platform" in
    Linux)
        app_dir=$work_dir/RustInSpace.AppDir
        app_id=io.github.olegbespalov.RustInSpace
        mkdir -p "$app_dir/usr/bin" "$app_dir/usr/share/doc/rust-in-space"
        install -m 755 "$binary" "$app_dir/usr/bin/rust-in-space"
        install -m 755 packaging/AppRun "$app_dir/AppRun"
        install -m 644 "packaging/$app_id.desktop" "$app_dir/$app_id.desktop"
        install -m 644 packaging/app-icon.png "$app_dir/$app_id.png"
        cp -R assets "$app_dir/usr/bin/"
        cp LICENSE README.md "$app_dir/usr/share/doc/rust-in-space/"
        desktop-file-validate "$app_dir/$app_id.desktop"
        APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 VERSION="${tag#v}" \
            appimagetool --no-appstream --runtime-file "$APPIMAGETOOL_RUNTIME_FILE" \
                "$app_dir" "$output.AppImage"
        ;;
    macOS)
        version=${tag#v}
        version=${version%%[-+]*}
        [[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
        app="$work_dir/Rust in Space.app"
        contents="$app/Contents"
        resources="$contents/Resources"
        iconset="$work_dir/AppIcon.iconset"
        mkdir -p "$contents/MacOS" "$resources" "$iconset"
        install -m 755 "$binary" "$contents/MacOS/rust-in-space"
        cp -R assets LICENSE README.md "$resources/"
        cp packaging/Info.plist "$contents/Info.plist"
        plutil -insert CFBundleShortVersionString -string "$version" "$contents/Info.plist"
        plutil -insert CFBundleVersion -string "$version" "$contents/Info.plist"
        plutil -lint "$contents/Info.plist"

        for size in 16 32 128 256 512; do
            retina=$((size * 2))
            sips -z "$size" "$size" packaging/app-icon.png --out "$iconset/icon_${size}x${size}.png"
            sips -z "$retina" "$retina" packaging/app-icon.png --out "$iconset/icon_${size}x${size}@2x.png"
        done
        iconutil -c icns "$iconset" -o "$resources/AppIcon.icns"
        codesign --force --sign - "$app"
        codesign --verify --strict "$app"
        ditto -c -k --keepParent "$app" "$output.zip"
        ;;
    *)
        echo "Unsupported release platform: $platform" >&2
        exit 1
        ;;
esac
