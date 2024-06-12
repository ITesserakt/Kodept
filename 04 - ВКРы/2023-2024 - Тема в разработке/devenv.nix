{ pkgs, lib, config, inputs, ... }:

{
    packages = with pkgs; [ 
    	texliveFull
    	just
    	watchexec
    	inkscape
    ];

    scripts.plantuml.exec = ''
    	cd $DEVENV_ROOT
    	${pkgs.jre8}/bin/java -jar "thirdparty/plantuml.jar" -o "$DEVENV_ROOT/src/figures/.generated/" $@
    '';

    env.GRAPHVIZ_DOT = "${pkgs.graphviz}/bin/dot";
}
