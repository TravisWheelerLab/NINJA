/*
 * DistanceReader.cpp
 *
 *  Created on: Feb 13, 2016
 *      Author: michel
 */
#include "DistanceReader.hpp"
#include <iostream>


DistanceReader::DistanceReader(std::string fileName){ //include exception, not ready yet
	FILE* inFile = NULL;
	size_t size = 0;
	this->distCalc = NULL;
	this->K = 0;
	if(&fileName == NULL or fileName.empty()){
		fprintf(stderr,"Distances cannot be read from stdin.\n");
		Exception::critical();
	}else{
		inFile = fopen(&fileName.at(0),"r");
		if (inFile == NULL){
			Exception::criticalErrno(fileName.c_str());
		}
		fseek(inFile,0,SEEK_END);
		size = ftell(inFile);
		fseek(inFile,0,SEEK_SET);
		if (size==0) {
			fprintf(stderr,"The file is invalid or empty.\n");
			Exception::critical();
		}
	}

	//gets the number of rows
	char aux[20];
    fscanf(inFile,"%s\n",aux);
    this->K = strtol(aux,NULL,0);

/*
    if ( ! (line.equals(""+K) )) {
    	//LogWriter.stdErrLogln("Invalid format for input file. Please use Phylip distance matrix format. Qutting");
		throw new Exception("Invalid format for input file. Please use Phylip distance matrix format. Qutting");
    }*/

	this->r = inFile;
	this->fileSize = size;
	this->threads = 0;
}

DistanceReader::DistanceReader(DistanceCalculator* distCalc, int k, int threads){
	if (distCalc == NULL){
		fprintf(stderr,"Null distance calculator handed to external matrix reader. Quitting.");
		Exception::critical();
	}

	this->distCalc = distCalc;
	this->K = k;
	this->fileSize = 0;
	this->r = NULL;
	this->threads = threads;
}

void DistanceReader::read(std::string **names, int** distances){ //possibly wrong, the else part, perhaps the distance calculator class as well

    unsigned int begin = 0, end = 0, numBegin = 0, numEnd = 0;
    int count = 0;

    if (this->threads == 0){
        omp_set_num_threads(omp_get_max_threads());
    } else {
        omp_set_num_threads(this->threads);
    }
    if (this->distCalc != NULL) {//using distCalc on input alignment
		#pragma omp parallel for
    	for (int i=0; i<this->K; i++)
    		for (int j=i+1; j<this->K; j++)
    		{
    			//distances[i][j-i-1] = this->distCalc->testDifferenceCluster(i,j);
    			distances[i][j-i-1] = 100 * (int)(((100000000*this->distCalc->calc(i,j))+50)/100) ; // this gets the same rounding I have in the distance writer code

    		}
    } else { //TODO: optimize this, too slow

    	char *x = new char[this->fileSize];
    	fread(x,1,this->fileSize,this->r);
    	fclose(this->r);


    	while(end < this->fileSize && count < (signed)this->K){
    		//get name limits
    		begin = numEnd;
    		while(x[begin] == ' ' || x[begin] == '\t' || x[begin] == '\n' || x[begin] == '\r')
    			begin++;
    		end=begin+1;
    		while(x[end] != ' ' && x[end] != '\t' && x[end] != '\n' && x[end] != '\r')
    			end++;

    		numEnd = end;
    		int count2 = 0;
    		int offset = 0;
			names[count]->assign(x,(size_t)begin,(size_t)(end-begin));
    		while(x[numEnd] != '\n' && x[numEnd] != '\t' && count2 < this->K){
				numBegin = numEnd+1;
				while(x[numBegin] == ' ' || x[numBegin] == '\t' || x[numBegin] == '\n' || x[numBegin] == '\r')
					numBegin++;
				numEnd = numBegin+1;
				while(x[numEnd] != ' ' && x[numEnd] != '\t' && x[numEnd] != '\n' && x[numEnd] != '\r')
					numEnd++;
				if(offset < this->K-1-count)
					if(count2 > count)
						distances[count][offset++] = 100 * (int)(((100000000*(atoi(&x[numBegin],numEnd-numBegin-1))+50)/100));
				count2++;
    		}
    		count++;
    	}

    	assert(count  == (signed)this->K);

    	delete[] x;

    }
}
void DistanceReader::write(FILE* outFile,int** distances){
	for (int i=0; i<this->K; i++)
		for (int j=i+1; j<this->K; j++)
			fprintf(outFile,"%d\n",distances[i][j-i-1]);
	fprintf(outFile,"\\\n");
}
float DistanceReader::atoi (char* in, int end){
	float val = 0.0;
	int pos = end;
	float multiplier = 1.0;
	bool isNeg = false;
	while (pos>=0) {
		if (in[pos] == '-' ) {
			isNeg = true;
		} else if(in[pos] == '.'){
			val /= multiplier;
			multiplier = 0.1;
		} else if (in[pos]<'0' || in[pos]>'9') {
			fprintf(stderr,"Unable to convert integer from invalid char array. Position + %d, char = [%c]",pos,in[pos]);
			Exception::critical();
		}else{
			val += multiplier * (float)(in[pos]-'0');
		}
		multiplier *= 10.0;
		pos--;
	}
	if (isNeg)
		return 0 - val;
	else
		return val;
}
