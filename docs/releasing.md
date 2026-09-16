# Releasing

## Source publication

Publishing the source repository on GitHub does not require Developer ID signing
or notarization. Users can build locally and run the source installer. The
`--release` signing requirement below applies to the prebuilt binary distribution
workflow, not to publishing source code.

## Prebuilt binaries

1. Install cargo-audit 0.22.2 and run sh scripts/release-check.sh.
2. Run hardware tests on the macOS versions and hardware you intend to claim.
   Record skipped and unverified cases in compatibility.md.
3. Update Cargo.toml and native/Info.plist together; regenerate the command docs.
4. Run sh scripts/package.sh --local on Apple Silicon for a local test archive.

The default package is ad-hoc signed for local use. It is not a notarized public
release. To create a Developer ID signed and notarized package, provide your own
existing signing identity and notarytool keychain profile:

    MAC_CLI_SIGN_IDENTITY="Developer ID Application: ..." \
    MAC_CLI_NOTARY_PROFILE="your-existing-profile" \
    sh scripts/package.sh --release

No signing credentials are stored in the repository. The script checks Apple's
returned status rather than treating submission as successful notarization.
A bare CLI executable cannot carry a stapled ticket; Gatekeeper can retrieve its
notarization ticket online. Use a signed installer package if offline stapling
becomes a distribution requirement.

Release mode reruns source checks and refuses development or ad-hoc signatures.
An Apple Development certificate is insufficient; use Developer ID Application.
Configure private vulnerability reporting on the hosting repository before release.

The archive includes mac, README, license, documentation, and
dependency/Rust standard-library license notices.
The adjacent .sha256 file records the archive checksum.

## Homebrew

After uploading the verified archive to the actual repository's release page,
generate a formula with its real HTTPS URL and homepage:

    python3 scripts/homebrew.py \
      --archive dist/mac-cli-0.1.0-aarch64-apple-darwin.tar.gz \
      --url YOUR_HTTPS_RELEASE_ARCHIVE_URL \
      --homepage YOUR_HTTPS_REPOSITORY_URL \
      --version 0.1.0

The generator validates inputs, calculates the checksum, and refuses to
overwrite an existing formula. No repository or release URL is invented.
Install the resulting formula through your own tap. Do not use brew link
--overwrite if another mac command is installed.

The source installer also refuses an existing mac command. Packaging and
formula generation do not publish anything, create a tap, or upload a release.
