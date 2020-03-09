/*
 * ArgumentHandler.cpp
 *
 *  Created on: Jan 24, 2016
 *      Author: michel
 */
#include <iostream>
#include "ArgumentHandler.hpp"

//TreeBuilder *treebuilder = new TreeBuilder();

ArgumentHandler::ArgumentHandler (char* argv[],int argc){
	this->abort = false;
	this->method.assign("default");
	this->njTmpDir = "";
	this->inType = alignment;
	this->outType = tree;
	this->alphType = null;
	this->corrType = not_assigned;
	this->outFile = stdout;
	this->threads = 0;
	this->SSE = true;
    this->printTime = false;

	if(argc == 1){
		printf("Use --help (or -h) to see possible arguments.\n");
		Exception::critical();
	}
	for(int i=1;i<argc;i++){
		std::string x,y;
		x.assign(argv[i]);
		if (i < argc-1) y.assign(argv[i+1]);
		if (!x.compare("--in") || !x.compare("-i")){
			if (i < argc-1){
				this->inFile.assign(argv[i+1]);
				i++;
			}else{
				fprintf(stderr,"No file specified.\n");
				Exception::critical();
			}
		}else if (!x.compare("--out") || !x.compare("-o")){
			if (i < argc-1){
				FILE* out = fopen(argv[i+1],"w+");
				i++;
				if (out == NULL){
					fprintf(stderr,"Error opening file");
					Exception::critical();
				}
				this->outFile = out;
			}else{
				fprintf(stderr,"No file specified.\n");
				Exception::critical();
			}
		}else if (!x.compare("--print-times")){
            this->printTime = true;
		}else if (!x.compare("--method") || !x.compare("-m")){
			if (i < argc-1){
				if(!y.compare("default")){
					//nothing happens, default is inmem
				}else if(!y.compare("extmem")){
					this->method.assign("extmem");
				}else if(!y.compare("inmem")){
					//nothing happens, default is inmem
				}else{
					fprintf(stderr,"Invalid method: available methods are  'default', 'inmem', and 'extmem'.");
					Exception::critical();
				}
				i++;
			}else{
				fprintf(stderr,"No method specified.\n");
				Exception::critical();
			}
		}else if (!x.compare("--in_type") || !x.compare("-it") ){
			if (i < argc-1){
				if(!y.compare("a")){
					this->inType = alignment;
				}else if(!y.compare("d")){
					this->inType = distance;
				}else{
					fprintf(stderr,"Invalid in_type:  try  'a' or 'd'.");
					Exception::critical();
				}
				i++;
			}else{
				fprintf(stderr,"No input type specified.\n");
				Exception::critical();
			}
		}else if (!x.compare("--out_type") || !x.compare("-ot") ){
			if (i < argc-1){
				if(!y.compare("d")){
					this->outType = dist;
				}else if(!y.compare("t")){
					this->outType = tree;
				}else{
					fprintf(stderr,"Invalid out_type:  try  'd' or 't'.");
					Exception::critical();
				}
				i++;
			}else{
				fprintf(stderr,"No output type specified.\n");
				Exception::critical();
			}
		}else if (!x.compare("--alph_type")){
			if (i < argc-1){
				if(!y.compare("a")){
					alphType = amino;
				}else if(!y.compare("d")){
					alphType = dna;
				}else{
					fprintf(stderr,"Invalid alph_type:  try  'a' or 'd'.");
					Exception::critical();
				}
				i++;
			}else{
				fprintf(stderr,"No alph_type specified.\n");
				Exception::critical();
			}
		}else if (!x.compare("--corr_type")){
			if (i < argc-1){
				if(!y.compare("n")){
					corrType = none;
				}else if(!y.compare("j")){
					corrType = JukesCantor;
				}else if(!y.compare("k")){
					corrType = Kimura2;
				}else if(!y.compare("s")){
					corrType = FastTree;
				}else{
					fprintf(stderr,"Invalid ut_type:  try  'n', 'j', 'k', or 's'.");
					Exception::critical();
				}
				i++;
			}else{
				fprintf(stderr,"No ut_type specified.\n");
				Exception::critical();
			}
/*		}else if (!x.compare("--verbose")){
			if (i < argc-1){
				int v = *argv[i+1] - '0'; //revisit
				//TreeBuilder.verbose = v; TODO:
				i++;
			}else{
				fprintf(stderr,"No verbose integer specified.\n");
				Exception::critical();
			}*/
		}else if (!x.compare("--quiet")){
				//TreeBuilder.verbose = 0;
		}else if (!x.compare("--tmp_dir") || !x.compare("-t")){
			if (i < argc-1){
				this->njTmpDir = argv[i+1];
				if (this->njTmpDir.empty()){
					fprintf(stderr,"No temporary directory specified.\n");
					Exception::critical();
				}
				i++;
			}else{
				fprintf(stderr,"No temporary directory specified.\n");
				Exception::critical();
			}
/*		}else if (!x.compare("--clust_size") || !x.compare("-s")){
			if (i < argc-1){
				int s = *argv[i+1] - '0'; //revisit
				//TreeBuilder.clustCnt = s; TODO:
				i++;
			}else{
				fprintf(stderr,"No clust_size integer specified.\n");
				Exception::critical();
			}
		}else if (!x.compare("--rebuild_step_ratio") || !x.compare("-r")){
			if (i < argc-1){
				int r = strtol(argv[i+1], NULL, 0);
				//TreeBuilder.rebuildStepRatio = r;
				i++;
			}else{
				fprintf(stderr,"No rebuild_step_ratio integer specified.\n");
				Exception::critical();
			}*/
		}else if(!x.compare("--threads") | !x.compare("-T")){
			int r = strtol(argv[i+1], NULL, 0);
			this->threads = r;
			i++;
		}else if(!x.compare("--NOSSE")){
			this->SSE = false;	
		}else if (!x.compare("--help") || !x.compare("-h")){
			printf("Arguments: \n");
			printf("--help (or -h) to display this help\n--in (or -i) filename\n--out (or -o) filename\n--method (or -m)  [inmem | extmem] (default inmem)\n");
			printf("--in_type type [a | d] (default a)\n--out_type type [t | d] (default t)\n--corr_type type [n | j | k | s]\n");
			printf("--threads (or -T) num_threads\nFor more information, check the README file.\n");
			this->abort = true;
		}else if (!x.compare("--version")){
			printf("Version 0.95\n");
			this->abort = true;
		}else{
			fprintf(stderr,"Invalid argument, ignored.\n");
		}
		//cand_heap_decay left out, not mentioned on Manual
	}
}
std::string ArgumentHandler::getMethod() {
	return this->method;
}
int ArgumentHandler::getNumThreads() {
	return this->threads;
}

std::string ArgumentHandler::getInFile() {
	return this->inFile;
}


std::string ArgumentHandler::getNJTmpDir() {
	return this->njTmpDir;
}

ArgumentHandler::InputType ArgumentHandler::getInputType () {
	return this->inType;
}

ArgumentHandler::OutputType ArgumentHandler::getOutputType () {
	return this->outType;
}
FILE* ArgumentHandler::getOutpuFile () {
	return this->outFile;
}

ArgumentHandler::AlphabetType ArgumentHandler::getAlphabetType () {
	return this->alphType;
}

ArgumentHandler::CorrectionType ArgumentHandler::getCorrectionType () {
	return this->corrType;
}

bool ArgumentHandler::argumentTest(){
	return true;
}

bool ArgumentHandler::useSSE(){
	return this->SSE;
}

bool ArgumentHandler::getPrintTime(){
	return this->printTime;
}
