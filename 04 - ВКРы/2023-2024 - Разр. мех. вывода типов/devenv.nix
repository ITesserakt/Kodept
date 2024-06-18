{ pkgs, lib, config, inputs, ... }:

{
    packages = with pkgs; [ 
    	texliveFull
    	just
    	watchexec
    	inkscape
    ];

    scripts.plantuml.exec = ''
    	${pkgs.jre8}/bin/java -jar "thirdparty/plantuml.jar" -o "$DEVENV_ROOT/src/figures/.generated/" $@
    '';

    env.GRAPHVIZ_DOT = "${pkgs.graphviz}/bin/dot";

    processes = {
    	assemble-pdf.exec = ''
    		just clean
    		just build_pdf
    		just build_gls
    		just build_bib
    		watchexec -d 2000 -i "*.puml" -w src -r -- just build_pdf
    	'';
    	
    	compile_figs.exec = ''
    		watchexec -d 2000 -w src/plantuml -w src/inkscape -r -- just compile
    	'';
    	
    	open-viewer.exec = ''${pkgs.evince}/bin/evince out/rndhpc_prj_2024_rk6_75b_nikitinvl_vkr.pdf'';
    	open-viewer.process-compose = {
    		is_foreground = true;
    	};

		assemble-pdf_pres.exec = ''
			cd presentation
			just clean
			just build_pdf
			just build_gls
			just build_bib
			watchexec -d 2000 -i "*.puml" -w src -r -- just build_pdf
		'';

    	open-viewer_pres.exec = ''${pkgs.evince}/bin/evince presentation/out/rndhpc_prj_2024_rk6_85b_nikitinvl_vkr_presentation.pdf'';
    	open-viewer_pres.process-compose.is_foreground = true;
    };
}
