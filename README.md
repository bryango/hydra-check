# hydra-check

![Test](https://github.com/nix-community/hydra-check/workflows/Test/badge.svg)

Check hydra for the build status of a package in a given channel.

# Disclaimer
Keep in mind that hydra is the NixOS build-farm orchestrator and has more important tasks to do than answering your puny requests. Response time may be in the seconds for each request.

# Usage

```console
$ nix-shell

$ hydra-check -h
Check hydra.nixos.org for build status of packages

Usage: hydra-check [OPTIONS] [PACKAGES]...

Arguments:
  [PACKAGES]...  
...

$ hydra-check --help
Check hydra.nixos.org for build status of packages

Channels can be:
  - unstable      - alias for nixos/trunk-combined (for NixOS) or nixpkgs/trunk
  - master        - alias for nixpkgs/trunk (Default for other architectures)
  - staging-next  - alias for nixpkgs/staging-next
  - 24.11         - alias for nixos/release-24.11
...

$ hydra-check
Evaluations of jobset nixpkgs/trunk @ https://hydra.nixos.org/jobset/nixpkgs/trunk/evals
⧖ nixpkgs → 9e09ce2  4h ago   ✔ 236021  ✖ 11574  ⧖ 7658  Δ ~      https://hydra.nixos.org/eval/1810735
⧖ nixpkgs → 33ec844  13h ago  ✔ 237618  ✖ 11623  ⧖ 6009  Δ ~      https://hydra.nixos.org/eval/1810729
⧖ nixpkgs → 581fd65  22h ago  ✔ 242549  ✖ 12400  ⧖ 296   Δ ~      https://hydra.nixos.org/eval/1810721
✔ nixpkgs → 7cc0bff  1d ago   ✔ 242782  ✖ 12467  ⧖ 0     Δ +129   https://hydra.nixos.org/eval/1810715
✔ nixpkgs → efd9668  1d ago   ✔ 242653  ✖ 12600  ⧖ 0     Δ +174   https://hydra.nixos.org/eval/1810710
...

$ hydra-check hello
Build Status for hello.x86_64-linux on jobset nixpkgs/trunk
https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-linux
✔  hello-2.12.1  2024-12-23  https://hydra.nixos.org/build/282851413
✔  hello-2.12.1  2024-12-03  https://hydra.nixos.org/build/280858326
...

$ hydra-check hello --arch x86_64-darwin
Build Status for hello.x86_64-darwin on jobset nixpkgs/trunk
https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-darwin
✔  hello-2.12.1  2024-12-23  https://hydra.nixos.org/build/282846204
✔  hello-2.12.1  2024-12-03  https://hydra.nixos.org/build/281028527
...

$ hydra-check hello python3 --channel 23.05
Build Status for nixpkgs.hello.x86_64-linux on jobset nixos/release-23.05
https://hydra.nixos.org/job/nixos/release-23.05/nixpkgs.hello.x86_64-linux
✔  hello-2.12.1  2023-12-31  https://hydra.nixos.org/build/245248169
...

Build Status for nixpkgs.python3.x86_64-linux on jobset nixos/release-23.05
https://hydra.nixos.org/job/nixos/release-23.05/nixpkgs.python3.x86_64-linux
✔  python3-3.10.13  2023-12-31  https://hydra.nixos.org/build/245266443
...

$ hydra-check nixos.tests.installer.simpleUefiGrub --channel 23.05 --arch aarch64-linux
Build Status for nixos.tests.installer.simpleUefiGrub.aarch64-linux on jobset nixos/release-23.05
https://hydra.nixos.org/job/nixos/release-23.05/nixos.tests.installer.simpleUefiGrub.aarch64-linux
✔              vm-test-run-installer-simpleUefiGrub  2024-01-05  https://hydra.nixos.org/build/245743824
...

$ hydra-check ugarit --channel 23.05 --short
Build Status for nixpkgs.ugarit.x86_64-linux on jobset nixos/release-23.05
✔  chicken-ugarit-2.0  2023-12-31  https://hydra.nixos.org/build/245243604

$ hydra-check nixos.containerTarball hello --channel 23.05 --json
{
  "nixos.containerTarball.x86_64-linux": [
    {
      "icon": "✔",
      "success": true,
      "status": "Succeeded",
      "timestamp": "2024-01-06T21:43:41Z",
      "build_id": "245747935",
      "build_url": "https://hydra.nixos.org/build/245747935",
      "name": "tarball",
      "arch": "x86_64-linux",
      "evals": true
    },
...
  ],
  "nixpkgs.hello.x86_64-linux": [
    {
      "icon": "✔",
      "success": true,
      "status": "Succeeded",
      "timestamp": "2023-12-31T21:34:50Z",
      "build_id": "245248169",
      "build_url": "https://hydra.nixos.org/build/245248169",
      "name": "hello-2.12.1",
      "arch": "x86_64-linux",
      "evals": true
    },
...
  ]
}

$ hydra-check --channel=staging-next --eval
info: no package filter has been specified, so the default filter '/nixVersions.stable' is used for better performance
info: specify another filter with --eval '<id>/<filter>', or force an empty filter with a trailing slash '/'

info: querying the latest evaluation of --jobset 'nixpkgs/staging-next'

Evaluations of jobset nixpkgs/staging-next @ https://hydra.nixos.org/jobset/nixpkgs/staging-next/evals
✔ nixpkgs → 94e324b  5d ago      ✔ 240490  ✖ 14688   ⧖ 0  Δ +379     https://hydra.nixos.org/eval/1810647
✔ nixpkgs → 93d9b27  6d ago      ✔ 240111  ✖ 14948   ⧖ 0  Δ +189     https://hydra.nixos.org/eval/1810617
...

Evaluation 1810647 filtered by 'nixVersions.stable' @ https://hydra.nixos.org/eval/1810647?filter=nixVersions.stable

input: nixpkgs
type: Git checkout
value: https://github.com/NixOS/nixpkgs.git
revision: 94e324b3937f6203b2271a61d8786646a3fb7ff8
store_path: /nix/store/ppc41xk8ii2hsfm1acarbvsbqbcsyl6n-source

input: officialRelease
type: Boolean
value: false

input: supportedSystems
type: Nix expression
value: [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ]

changed_input: nixpkgs
changes: 93d9b277f418 to 94e324b3937f
url: https://hydra.nixos.org/api/scmdiff?rev1=93d9b277f4182eec56c90b7e16e8457d9eb303e4&type=git&uri=https%3A%2F%2Fgithub.com%2FNixOS%2Fnixpkgs.git&rev2=94e324b3937f6203b2271a61d8786646a3fb7ff8&branch=
revs: 93d9b277f4182eec56c90b7e16e8457d9eb303e4 -> 94e324b3937f6203b2271a61d8786646a3fb7ff8

Still Succeeding:
✔  nixVersions.stable.aarch64-darwin  nix-2.24.11  2024-12-18  https://hydra.nixos.org/build/282268230
✔  nixVersions.stable.aarch64-linux   nix-2.24.11  2024-12-17  https://hydra.nixos.org/build/282264935
✔  nixVersions.stable.x86_64-darwin   nix-2.24.11  2024-12-21  https://hydra.nixos.org/build/282407544
✔  nixVersions.stable.x86_64-linux    nix-2.24.11  2024-12-20  https://hydra.nixos.org/build/282408286

```

# Changelog

## 2.0.0 Breaking changes
- Rewritten in Rust
- Always prints long outputs with all recent builds unless `--short` is explicitly specified
- `--arch` defaults to the target architecture (instead of `x86_64-linux` all the time)
- `--jobset` explicitly conflicts with `--channel` to avoid confusion, as channels are just aliases for jobsets
- The `staging` channel / alias is removed as `nixos/staging` is no longer active; instead we add `staging-next` as an alias for `nixpkgs/staging-next`
- The default `unstable` channel points to `nixpkgs/trunk` on non-NixOS systems

### Features
- Print recent evaluations of the jobset if no package is specified
- Add an `--eval` flag for information about a specific evaluation
- Infer the current stable Nixpkgs release (e.g. `24.05`) with a hack
- Support standard channel names (e.g. `nixos-unstable`)
- Generate shell completions with `--shell-completion SHELL`
- Print nicely formatted, colored and aligned tables
