{
  description = "Ancorix development shell: slangc, Vulkan loader, validation layers";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        # mkShellNoCC, not mkShell: mkShell puts the nix cc wrapper on PATH,
        # rustc links through it, and the resulting binary carries a nix
        # RUNPATH and interpreter - so it runs inside the shell and fails
        # outside it with NoWaylandLib. Linking stays with the host toolchain.
        default = pkgs.mkShellNoCC {
          # No Rust toolchain here on purpose: ancorix is built with whatever
          # rustup the host has, and pulling a second one in would only make
          # `cargo` ambiguous. The shell exists for the two things that are
          # awkward to get from a distro - slangc and the validation layers.
          packages = [ pkgs.shader-slang ];

          # Everything below is dlopen'd at runtime, never linked at build
          # time: ash uses the `loaded` feature, winit dlopens X11 and
          # Wayland. So these belong in LD_LIBRARY_PATH, not in buildInputs.
          shellHook =
            let
              runtimeLibs = pkgs.lib.makeLibraryPath [
                # `slangc -target exe` links against slang's own runtime, and
                # the host cc bakes no RUNPATH for it
                pkgs.shader-slang
                pkgs.vulkan-loader
                pkgs.wayland
                pkgs.libxkbcommon
                pkgs.libx11
                pkgs.libxcursor
                pkgs.libxrandr
                pkgs.libxi
              ];
            in
            ''
              # VK_ADD_LAYER_PATH adds to the search path instead of replacing
              # it, so layers the host already has keep working.
              export VK_ADD_LAYER_PATH="${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d''${VK_ADD_LAYER_PATH:+:$VK_ADD_LAYER_PATH}"
              export LD_LIBRARY_PATH="${runtimeLibs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

              # Drivers come from the host: the loader finds them through
              # XDG_DATA_DIRS (/usr/share/vulkan/icd.d), and a cargo-built
              # binary resolves their libdrm/libLLVM through the host
              # ld.so.cache. Set VK_DRIVER_FILES to override the choice.
              #
              # This is also why vulkan-tools is not in `packages`: a nix
              # binary runs under the nix ELF interpreter, which ignores
              # /etc/ld.so.cache, so `vulkaninfo` from the store reports
              # "Found no drivers!" against host ICDs. Use the distro's.

              echo "ancorix devshell: slangc $(slangc -v 2>&1 | head -1)"
              echo "validation layers on VK_ADD_LAYER_PATH; debug builds enable them by themselves"
            '';
        };
      });
    };
}
