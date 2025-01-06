{
  lib,
  hydra-check,
  rustPlatform,
  versionCheckHook,
  flake ? { },
}@args:

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

        # append git revision to the version string, if available
        revSuffix =
          if (flake ? dirtyShortRev) then
            "-g${flake.dirtyShortRev}"
          else if (flake ? shortRev) then
            "-g${flake.shortRev}-unstable"
          else
            "-unstable";

        # append last modified date to the version string, if available
        inherit (builtins) substring;
        epoch = "19700101";
        date = substring 0 8 (flake.lastModifiedDate or epoch);
        dateSuffix = lib.optionalString (date != epoch) (
          let
            inherit (flake) lastModifiedDate;
            year = substring 0 4 lastModifiedDate;
            month = substring 4 2 lastModifiedDate;
            day = substring 6 2 lastModifiedDate;
          in
          "-${year}-${month}-${day}"
        );
      in
      "${packageVersion}${revSuffix}${dateSuffix}";

    # `builtins.path` works well with lazy trees
    src =
      args.flake or (builtins.path {
        name = "hydra-check-source";
        path = ./.;
      });

    cargoDeps = rustPlatform.importCargoLock {
      lockFile = builtins.path {
        name = "hydra-check-Cargo.lock";
        path = ./Cargo.lock;
      };
      outputHashes = {
        "trycmd-0.15.9" = "sha256-LskZORhkNstrVvI7N1LHExwxlEOmyaHVvhqHKTZlcsM=";
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
