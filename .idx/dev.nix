# To learn more about how to use Nix to configure your environment
# see: https://firebase.google.com/docs/studio/customize-workspace
{ pkgs, ... }: {
  # Which nixpkgs channel to use.
  channel = "stable-24.05"; # or "unstable"

  # Use https://search.nixos.org/packages to find packages
  packages = [
    pkgs.bazel_7 # Bazel build tool
    pkgs.go
    pkgs.rustc
    pkgs.cargo
    pkgs.rust-analyzer # For Rust language server
    pkgs.jdk # For Java Development Kit (defaults to a recent stable version)
    pkgs.sbt # Scala Build Tool (even if using Bazel, might be useful for Scala tooling)
    pkgs.scala # Scala language
    pkgs.nodejs_20 # Node.js (LTS version)
    pkgs.bun # Bun JavaScript runtime
  ];

  # Sets environment variables in the workspace
  env = {
    # JAVA_HOME can be useful for Java projects
    # Nixpkgs typically sets this up, but explicitly can't hurt if needed by specific tools
    # JAVA_HOME = "${pkgs.jdk}/lib/openjdk"; # Example, adjust if needed based on pkgs.jdk structure
  };

  idx = {
    # Search for the extensions you want on https://open-vsx.org/ and use "publisher.id"
    extensions = [
      "rust-lang.rust-analyzer", # Rust Language Server
      "vscjava.vscode-java-pack", # Java extension pack
      "scala-lang.scala", # Scala language support (Metals)
      "dbaeumer.vscode-eslint", # ESLint for Node.js/Bun projects
      "oven.bun-vscode" # Bun support
      # "vscodevim.vim"
    ];

    # Enable previews
    previews = {
      enable = true;
      previews = {
        # web = {
        #   # Example: run "npm run dev" with PORT set to IDX's defined port for previews,
        #   # and show it in IDX's web preview panel
        #   command = ["npm" "run" "dev"];
        #   manager = "web";
        #   env = {
        #     # Environment variables to set for your server
        #     PORT = "$PORT";
        #   };
        # };
      };
    };

    # Workspace lifecycle hooks
    workspace = {
      # Runs when a workspace is first created
      onCreate = {
        # Example: install JS dependencies from NPM
        # npm-install = "npm install";
        # Bun install example (if you have a bun.lockb and package.json)
        # bun-install = "bun install";
      };
      # Runs when the workspace is (re)started
      onStart = {
        # Example: start a background task to watch and re-build backend code
        # watch-backend = "npm run watch-backend";
      };
    };
  };
}
