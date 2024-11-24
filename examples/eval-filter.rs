mod utils;

fn main() -> anyhow::Result<()> {
    hydra_check!("--eval rust-analyzer", do_assert = false, @r#"
    [1;37minfo:[0m querying the latest evaluation of --jobset 'nixpkgs/trunk'

    Evaluations of jobset [1mnixpkgs/trunk[0m [2m@ https://hydra.nixos.org/jobset/nixpkgs/trunk/evals[0m
    [33m⧖[0m nixpkgs → d440628  1h ago      [32m✔[0m 240310  [31m✖[0m 4303  [1m[33m⧖[0m[1m 9490[0m  Δ ~     [2mhttps://hydra.nixos.org/eval/1810108[0m
    [33m⧖[0m nixpkgs → 72f6884  9h ago      [32m✔[0m 240771  [31m✖[0m 4435  [1m[33m⧖[0m[1m 8886[0m  Δ ~     [2mhttps://hydra.nixos.org/eval/1810105[0m
    [33m⧖[0m nixpkgs → 67fd9cc  18h ago     [32m✔[0m 243629  [31m✖[0m 6493  [1m[33m⧖[0m[1m 3884[0m  Δ ~     [2mhttps://hydra.nixos.org/eval/1810097[0m
    [33m⧖[0m nixpkgs → 8edf06b  1d ago      [32m✔[0m 246613  [31m✖[0m 5446  [1m[33m⧖[0m[1m 2191[0m  Δ ~     [2mhttps://hydra.nixos.org/eval/1810089[0m
    [33m⧖[0m nixpkgs → eb2872e  1d ago      [32m✔[0m 246648  [31m✖[0m 5477  [1m[33m⧖[0m[1m 2051[0m  Δ ~     [2mhttps://hydra.nixos.org/eval/1810083[0m
    [33m⧖[0m nixpkgs → 96ae446  1d ago      [32m✔[0m 246832  [31m✖[0m 5645  [1m[33m⧖[0m[1m 1639[0m  Δ ~     [2mhttps://hydra.nixos.org/eval/1810079[0m
    [33m⧖[0m nixpkgs → e34e6cb  2d ago      [32m✔[0m 246883  [31m✖[0m 5642  [1m[33m⧖[0m[1m 1575[0m  Δ ~     [2mhttps://hydra.nixos.org/eval/1810075[0m
    [33m⧖[0m nixpkgs → d774cc7  2d ago      [32m✔[0m 247034  [31m✖[0m 5652  [1m[33m⧖[0m[1m 1209[0m  Δ ~     [2mhttps://hydra.nixos.org/eval/1810064[0m
    [33m⧖[0m nixpkgs → 4f9c49e  2d ago      [32m✔[0m 247989  [31m✖[0m 5770  [1m[33m⧖[0m[1m 132[0m   Δ [32m+6[0m    [2mhttps://hydra.nixos.org/eval/1810058[0m
    [33m⧖[0m nixpkgs → 45573a3  3d ago      [32m✔[0m 247983  [31m✖[0m 5808  [1m[33m⧖[0m[1m 70[0m    Δ [32m+67[0m   [2mhttps://hydra.nixos.org/eval/1810050[0m
    [32m✔[0m nixpkgs → f8b656d  3d ago      [32m✔[0m 247916  [31m✖[0m 5952  [33m⧖[0m 0     Δ [31m-20[0m   [2mhttps://hydra.nixos.org/eval/1810041[0m
    [32m✔[0m nixpkgs → f818c4a  4d ago      [32m✔[0m 247936  [31m✖[0m 5888  [33m⧖[0m 0     Δ [31m-268[0m  [2mhttps://hydra.nixos.org/eval/1810036[0m
    [32m✔[0m nixpkgs → 462a897  4d ago      [32m✔[0m 248204  [31m✖[0m 5609  [33m⧖[0m 0     Δ [31m-116[0m  [2mhttps://hydra.nixos.org/eval/1810027[0m
    [32m✔[0m nixpkgs → 10343b0  4d ago      [32m✔[0m 248320  [31m✖[0m 5287  [33m⧖[0m 0     Δ [31m-141[0m  [2mhttps://hydra.nixos.org/eval/1810017[0m
    [32m✔[0m nixpkgs → 5083ec8  5d ago      [32m✔[0m 248461  [31m✖[0m 5138  [33m⧖[0m 0     Δ [32m+272[0m  [2mhttps://hydra.nixos.org/eval/1810006[0m
    [32m✔[0m nixpkgs → 0a14706  6d ago      [32m✔[0m 248189  [31m✖[0m 5286  [33m⧖[0m 0     Δ [32m+7[0m    [2mhttps://hydra.nixos.org/eval/1809995[0m
    [32m✔[0m nixpkgs → f9aa610  6d ago      [32m✔[0m 248182  [31m✖[0m 5289  [33m⧖[0m 0     Δ [32m+89[0m   [2mhttps://hydra.nixos.org/eval/1809990[0m
    [32m✔[0m nixpkgs → c69a9bf  6d ago      [32m✔[0m 248093  [31m✖[0m 5354  [33m⧖[0m 0     Δ [32m+60[0m   [2mhttps://hydra.nixos.org/eval/1809987[0m
    [32m✔[0m nixpkgs → 34a6264  2024-11-16  [32m✔[0m 248033  [31m✖[0m 5358  [33m⧖[0m 0     Δ [32m+177[0m  [2mhttps://hydra.nixos.org/eval/1809982[0m
    [32m✔[0m nixpkgs → 9eea90d  2024-11-15  [32m✔[0m 247856  [31m✖[0m 5512  [33m⧖[0m 0     Δ [31m-331[0m  [2mhttps://hydra.nixos.org/eval/1809976[0m

    Evaluation [1m1810108[0m filtered by '[1mrust-analyzer[0m' [2m@ https://hydra.nixos.org/eval/1810108?filter=rust-analyzer[0m

    [1minput[0m: nixpkgs
    [1mtype[0m: Git checkout
    [1mvalue[0m: https://github.com/nixos/nixpkgs.git
    [1mrevision[0m: d440628dda319389a2c9a104a06e50db4f8c19fa
    [1mstore_path[0m: /nix/store/4v4m3x57np3ggq52c29vj6wgffchrx5y-source

    [1minput[0m: officialRelease
    [1mtype[0m: Boolean
    [1mvalue[0m: false

    [1mchanged_input[0m: nixpkgs
    [1mchanges[0m: 72f688496625 to d440628dda31
    [1murl[0m: https://hydra.nixos.org/api/scmdiff?uri=https%3A%2F%2Fgithub.com%2Fnixos%2Fnixpkgs.git&rev2=d440628dda319389a2c9a104a06e50db4f8c19fa&rev1=72f68849662579c8d4e5d13bd4d400723a1d8edd&type=git&branch=
    [1mrevs[0m: 72f68849662579c8d4e5d13bd4d400723a1d8edd -> d440628dda319389a2c9a104a06e50db4f8c19fa

    [1mStill Succeeding:[0m
    [32m✔[0m  rust-analyzer-unwrapped.aarch64-darwin                    rust-analyzer-unwrapped-2024-11-11       2024-11-18  [2mhttps://hydra.nixos.org/build/279419139[0m
    [32m✔[0m  rust-analyzer-unwrapped.aarch64-linux                     rust-analyzer-unwrapped-2024-11-11       2024-11-18  [2mhttps://hydra.nixos.org/build/279418697[0m
    [32m✔[0m  rust-analyzer-unwrapped.x86_64-darwin                     rust-analyzer-unwrapped-2024-11-11       2024-11-18  [2mhttps://hydra.nixos.org/build/279418823[0m
    [32m✔[0m  rust-analyzer-unwrapped.x86_64-linux                      rust-analyzer-unwrapped-2024-11-11       2024-11-18  [2mhttps://hydra.nixos.org/build/279420296[0m
    [32m✔[0m  rust-analyzer.aarch64-darwin                              rust-analyzer-2024-11-11                 2024-11-18  [2mhttps://hydra.nixos.org/build/279421335[0m
    [32m✔[0m  rust-analyzer.aarch64-linux                               rust-analyzer-2024-11-11                 2024-11-18  [2mhttps://hydra.nixos.org/build/279420774[0m
    [32m✔[0m  rust-analyzer.x86_64-darwin                               rust-analyzer-2024-11-11                 2024-11-18  [2mhttps://hydra.nixos.org/build/279419129[0m
    [32m✔[0m  rust-analyzer.x86_64-linux                                rust-analyzer-2024-11-11                 2024-11-18  [2mhttps://hydra.nixos.org/build/279420627[0m
    [32m✔[0m  vimPlugins.coc-rust-analyzer.aarch64-darwin               vimplugin-coc-rust-analyzer-0.77.5       2024-11-15  [2mhttps://hydra.nixos.org/build/278450218[0m
    [32m✔[0m  vimPlugins.coc-rust-analyzer.aarch64-linux                vimplugin-coc-rust-analyzer-0.77.5       2024-11-15  [2mhttps://hydra.nixos.org/build/278528095[0m
    [32m✔[0m  vimPlugins.coc-rust-analyzer.x86_64-darwin                vimplugin-coc-rust-analyzer-0.77.5       2024-11-15  [2mhttps://hydra.nixos.org/build/278414948[0m
    [32m✔[0m  vimPlugins.coc-rust-analyzer.x86_64-linux                 vimplugin-coc-rust-analyzer-0.77.5       2024-11-15  [2mhttps://hydra.nixos.org/build/278581527[0m
    [32m✔[0m  vscode-extensions.rust-lang.rust-analyzer.aarch64-darwin  vscode-extension-rust-analyzer-0.3.2029  2024-11-18  [2mhttps://hydra.nixos.org/build/279420149[0m
    [32m✔[0m  vscode-extensions.rust-lang.rust-analyzer.aarch64-linux   vscode-extension-rust-analyzer-0.3.2029  2024-11-18  [2mhttps://hydra.nixos.org/build/279419467[0m
    [32m✔[0m  vscode-extensions.rust-lang.rust-analyzer.x86_64-darwin   vscode-extension-rust-analyzer-0.3.2029  2024-11-18  [2mhttps://hydra.nixos.org/build/279419245[0m
    [32m✔[0m  vscode-extensions.rust-lang.rust-analyzer.x86_64-linux    vscode-extension-rust-analyzer-0.3.2029  2024-11-18  [2mhttps://hydra.nixos.org/build/279421452[0m
    "#);
    Ok(())
}
