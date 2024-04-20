#!/usr/bin/env just --justfile

PROJECT_NAME := 'cpxsln_rpt_prj_2024_rk6_75b_nikitinvl_nirs'
OUTDIR := join(justfile_directory(), 'out')
SRCDIR := join(justfile_directory(), 'src/')
DOCDIR := join(justfile_directory(), 'doc/')

export BIBINPUTS := SRCDIR
export BSTINPUTS := SRCDIR

default:
    just --choose

clean:
    rm -rf {{OUTDIR}}

[private]
mk_folder:
    mkdir -p {{OUTDIR}}
    mkdir -p {{OUTDIR}}/chapters

[private]
build_pdf:
    cd {{ SRCDIR }} && latexmk -quiet -outdir={{ OUTDIR }} -pdf {{ PROJECT_NAME }}

[private]
build_gls:
    cd {{ OUTDIR }} && makeglossaries-lite {{ PROJECT_NAME }}

[private]
build_bib:
    cd {{ OUTDIR }} && bibtex {{ PROJECT_NAME }}

build: mk_folder build_pdf build_gls build_bib && build_pdf

rebuild: clean build

dist: rebuild
    cp {{ OUTDIR }}/{{ PROJECT_NAME }}.pdf {{ DOCDIR }}
