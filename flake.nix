{
	description = "Glotta - My toy language";
	inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

	outputs = { self, nixpkgs }:
		let
			supportedSystems = [ "x86_64-linux" ];
			forAllSupportedSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
				pkgs = import nixpkgs { inherit system; };
			});
		in
			{
			devShells = forAllSupportedSystems ({ pkgs }: {
				default = pkgs.mkShell {
						packages = with pkgs; [
							zig
							zls
							lldb
						];
					};
			});
		};
}
