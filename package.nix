{
  lib,
  hydra-check,
  rustPlatform,
  versionCheckHook,
  shortRev ? null,
}:

hydra-check.overrideAttrs (
  {
    meta ? { },
    nativeInstallCheckInputs ? [ ],
    ...
  }:
  {
    version =
      let
        packageVersion = with builtins; (fromTOML (readFile ./Cargo.toml)).package.version;
        versionSuffix = if shortRev == null then "" else "-g${shortRev}";
      in
      "${packageVersion}${versionSuffix}";

    # `builtins.path` works well with lazy trees
    src = builtins.path {
      name = "hydra-check-source";
      path = ./.;
    };

    cargoDeps = rustPlatform.importCargoLock {
      lockFile = builtins.path {
        name = "hydra-check-Cargo.lock";
        path = ./Cargo.lock;
      };
    };

    nativeInstallCheckInputs = nativeInstallCheckInputs ++ [
      versionCheckHook
    ];

    doInstallCheck = true;

    meta = meta // {
      maintainers = with lib.maintainers; [
        makefu
        artturin
        bryango
      ];
    };
  }
)
