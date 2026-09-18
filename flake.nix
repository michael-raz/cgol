{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
	};

	outputs = { self, nixpkgs }@inputs:
		let
			system = "x86_64-linux";
			pkgs = import nixpkgs { inherit system; };
		in
		{
			devShells.${system}.default = pkgs.mkShell {
				shellHook = ''
					PS1="\[\033[36m\][flake]\[\033[0m\] $PS1"
					alias build="wasm-pack build --target web"
				'';
				packages = with pkgs; [
					cargo
					lld
					wasm-pack
				];
			};
		};
}
