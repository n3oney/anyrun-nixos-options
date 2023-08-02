# anyrun-nixos-options

An anyrun plugin that lets you search NixOS options.

# how 2 build?

`nix build`  
... or `cargo build` optionally :)

# Configuration

This plugin requires a config in your anyrun config directory called `nixos_options.ron`.
The file looks like this:

```ron
Config(
  options_path: "/path/to/options.json", // You can obtain it using config.system.build.manual.optionsJSON
  prefix: ":nix", // Optional, default: ":nix"
  min_score: 50, // Optional, the minimum score of entries to show. The smaller the laggier. Default: 50
  nixpkgs_url: "https://github.com/NixOS/nixpkgs/blob/nixos-unstable" // Optional, URL to Nixpkgs tree. Set it to use the same branch as you're using. Defaults to the unstable url.
)
```

# Using this with NixOS?

1. Use the [home-manager module](https://github.com/NixOS/nixpkgs/blob/nixos-unstable)
2. Add the plugin to it:
   ```nix
     programs.anyrun.config.plugins = [ inputs.anyrun-nixos-options.packages.${pkgs.system}.default ];
   ```
3. Create a config file for it:
   ```nix
     programs.anyrun.extraConfigFiles."nixos-options.ron".text = ''
       Config(
         options_path: "${config.system.build.manual.optionsJSON}/share/doc/nixos/options.json"
       )
     '';
   ```
4. Done.
