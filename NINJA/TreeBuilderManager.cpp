/*
 * TreeBuilderManager.cpp
 *
 *  Created on: Feb 7, 2016
 *      Author: michel
 */

#include "TreeBuilderManager.hpp"

#define LINUX 1

#ifdef LINUX
#include <unistd.h>
#endif

//standard constructor
TreeBuilderManager::TreeBuilderManager(std::string method, std::string njTmpDir, std::string inFile, FILE* outFile, InputType inType, OutputType outType, AlphabetType alphType, CorrectionType corrType){
	this->method = method;
	this->njTmpDir = njTmpDir;
	this->inFile = inFile;
	this->outFile = outFile;
	this->inType = inType;
	this->outType = outType;
	this->alphType = alphType;
	this->corrType = corrType;
	this->names = NULL;
	this->chars = "abcdefghijklmonpqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
	this->newDistanceMethod = false;
}


std::string TreeBuilderManager::doJob(){
	// do all sorts of file and memory things using java libraries, I have to rewrite everything
	//it should be different depending on the OS, for system operations such as checking the memory available
	//I should consider creating a Logger object to handle all sorts of prints and expection throws
	int** distances = NULL;
	float** memD = NULL;
	float* R  = NULL;
	// all for external memory version
	int floatSize = 4;
	int pageBlockSize = 1024; //that many ints = 4096 bytes;
	FILE* diskD = NULL;

	int rowLength = 0;
	int firstMemCol = -1;

	int numCols = 0;


	//Runtime runtime = Runtime.getRuntime();
	long maxMemory = -1; //revisit, priority

	#ifdef LINUX //TODO: include support for multiple plataforms
	maxMemory = sysconf(_SC_PAGE_SIZE)*sysconf(_SC_AVPHYS_PAGES);
	#endif

	bool ok = true;
	TreeNode** nodes = NULL;
	std::string treeString = "";

	//NinjaLogWriter.printVersion();

	int K=0;

	#ifdef LINUX
	maxMemory = sysconf(_SC_PAGE_SIZE)*sysconf(_SC_AVPHYS_PAGES); //working, should I use all of this, or reduce it? How does the SO handles if I use it all and it does not have all of that anymore?
	#endif

	//TODO: check if I close all the files

	SequenceFileReader* seqReader = NULL;
	if (!this->method.compare("extmem")){ //The external memory if currently in debugging, therefore it is unabled.
/*		fprintf(stderr,"External memory not allowed yet.\n");
		Exception::critical();*/
		if (maxMemory < 1900000000) {
			fprintf(stderr,"Warning: using an external-memory variant of NINJA with less than 2GB allocated RAM.\n");
			fprintf(stderr,"The data structures of NINJA may not work well if given less than 2GB.\n");
		}
		fprintf(stderr,"Using External Memory...\n");
		FILE* tempFile;
		njTmpDir += "treeBuilderManager";


		tempFile = fopen(njTmpDir.c_str(), "w+");
		if(tempFile == NULL) Exception::criticalErrno(njTmpDir.c_str());

		fprintf(stderr,"created temporary directory for this run of NINJA : %s\n", njTmpDir.c_str());


		DistanceReaderExtMem* reader = NULL;

		if (inType == alignment) {
			seqReader = new SequenceFileReader(&(this->inFile),(SequenceFileReader::AlphabetType) this->alphType);
			std::string** seqs = seqReader->getSeqs();
			this->names = seqReader->getNames();
			this->alphType = (TreeBuilderManager::AlphabetType) seqReader->getAlphType();
			fprintf(stderr,"Calculating distances....\n");
			DistanceCalculator* distCalc = new DistanceCalculator(seqs,(DistanceCalculator::AlphabetType) alphType,(DistanceCalculator::CorrectionType)  corrType, seqReader->seqSize);
			K = seqReader->seqSize;
			reader = new DistanceReaderExtMem(distCalc, K);
		}	else {
			fprintf(stderr,"External memory with distances as input not allowed yet.\n"); //TODO: implement
			Exception::critical();
			//reader = new DistanceReaderExtMem(this->inFile);
			K = reader->K;
			this->names = new std::string*[K];
			for (int i = 0;i<K;i++)
				this->names[i] = new std::string();
		}



		R = new float[K]();
		rowLength = (K + K-2); //that's a K*K table for the initial values, plus another K*(K-2) columns for new nodes

		long maxSize; // max amount of D stored in memory
		if (TreeBuilderExtMem::useBinPairHeaps) {
			maxSize = maxMemory / 10;
		} else {
			maxSize = maxMemory / 3;
		}

		numCols = (int)(maxSize / (4 * K));
		int numBlocks = numCols/pageBlockSize; // chops off fractional part
		if (numBlocks == 0) numBlocks  = 1;  //for huge inputs, this could result in memD larger than 400MB
		numCols = numBlocks * pageBlockSize;


		if (numCols >= 2*K-2) {
			numCols = 2*K-2;
		} else {
			std::string newDir = njTmpDir + "ninja_diskD_tmp";
			FILE* tmpFile = fopen(newDir.c_str(),"w+");
			diskD = tmpFile;
		}

		memD = new float*[K];
		for(int i=0;i<K;i++){
			memD[i] = new float[numCols];
		}
		firstMemCol = reader->read( names, R, diskD, memD, numCols, rowLength, pageBlockSize);

	}else{
		DistanceReader* reader = NULL;

		if (this->inType == alignment) {
			seqReader = new SequenceFileReader(&(this->inFile),(SequenceFileReader::AlphabetType) this->alphType);
			std::string** seqs = seqReader->getSeqs();
			this->names = seqReader->getNames();
			this->alphType = (TreeBuilderManager::AlphabetType) seqReader->getAlphType();
			fprintf(stderr,"Calculating distances....\n");
			DistanceCalculator* distCalc = new DistanceCalculator(seqs,(DistanceCalculator::AlphabetType) alphType,(DistanceCalculator::CorrectionType)  corrType, seqReader->seqSize);
			K = seqReader->seqSize;
			reader = new DistanceReader(distCalc, K);
		}else {
			reader = new DistanceReader(this->inFile);
			K = reader->K;
			this->names = new std::string*[K];
			for (int i = 0;i<K;i++)
				this->names[i] = new std::string();
		}

		distances = new int*[K];
		for (int i=0; i<K; i++) {
			distances[i] = new int[K - i - 1];
		}

		reader->read( this->names, distances);

		if(this->outType == dist){
			reader->write(this->outFile,distances);
			return "";
		}

	}

	fprintf(stderr,"Generating tree....\n");
	int nodesSize = 0;
	TreeBuilderBinHeap* tb = NULL;
	TreeBuilderExtMem *tb_extmem = NULL;
	if (!this->method.compare("inmem") or !this->method.compare("default")) {
		tb = new TreeBuilderBinHeap(this->names, distances, K);
		nodes = tb->build();
		nodesSize = (tb->K*2)-1;

	} else if (!method.compare("extmem") ) {
		tb_extmem = new TreeBuilderExtMem(names, K,  R, njTmpDir, diskD, memD , numCols, firstMemCol, rowLength, maxMemory);
		nodes = tb_extmem->build();
		nodesSize = (tb_extmem->K*2)-1;
	}
	std::string *sb;
	if (ok && treeString.empty()) {
		if (nodes != NULL) {
			sb = new std::string();
			*sb = "";
			nodes[nodesSize-1]->buildTreeString(sb);
			treeString = *sb + ";\n";
			delete sb;
		}
	}
	if (tb != NULL)
		delete tb;

	if (tb_extmem != NULL)
		delete tb_extmem;

	if (seqReader != NULL)
		delete seqReader;

	delete[] distances;
	return (treeString);
}
