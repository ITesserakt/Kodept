{ pkgs, lib, config, inputs, ... }:

{
    packages = with pkgs; [ texliveFull just mermaid-cli ];
}
