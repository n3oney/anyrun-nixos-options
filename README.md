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
  options_paths: ["/path/to/options.json"], // You can obtain it using config.system.build.manual.optionsJSON
  prefix: ":nix", // Optional, default: ":nix"
  min_score: 0, // Optional, the minimum score of entries to show. Set it to a larger value on slow machines. Default: 0
  nixpkgs_url: "https://github.com/NixOS/nixpkgs/blob/nixos-unstable" // Optional, URL to Nixpkgs tree. Set it to use the same branch as you're using. Defaults to the unstable url.
)
```

**Important: Make sure to set a `max_entries` in your anyrun config. Without that, the plugin will be VERY slow, since there exist over 16 thousand options to search through at the time of writing this.**
**I have set mine to 10, since even that value is enough for it to go off-screen, so you shouldn't lose any data.**

# Using this with NixOS?

Under flakes, the following instructions will apply:

1. Use the [anyrun home-manager module](https://github.com/Kirottu/anyrun/blob/master/nix/hm-module.nix)
2. Add the plugin to your anyrun plugins list

```nix
programs.anyrun.config = {
    # ...
    plugins = [
        inputs.anyrun-nixos-options.packages.${pkgs.system}.default
        # other plugins that you might have
    ];
    # ...
};
```

3. Create a config file for it:

```nix
#                  ↓ make sure osConfig is in the argument set
{inputs, pkgs, osConfig,  ...}: {
    programs.anyrun.extraConfigFiles."nixos-options.ron".text = let
        #               ↓ home-manager refers to the nixos configuration as osConfig
        nixos-options = osConfig.system.build.manual.optionsJSON + "/share/doc/nixos/options.json";
        # get the docs-json package from the home-manager flake
        hm-options = inputs.home-manager.packages.${pkgs.system}.docs-json + "/share/doc/home-manager/options.json";
        # merge your options
        options = builtins.toJSON ["${nixos-options}"];
        # or alternatively if you wish to read any other documentation options, such as home-manager
        # options = builtins.toJSON ["${nixos-options}" "${nixos-options}" "${some-other-option}" /* ... */];

    in ''
        Config(
            # add your option paths
            options_path: "${options}",
         )
    '';
}

```

4. You are done. Rebuild your system and run anyrun as usual. `:nix` should bring up
   your NixOS (and any other configured) options

Without flakes, inputs... generally should be changed to <channel> or <source> depending on your usage.
