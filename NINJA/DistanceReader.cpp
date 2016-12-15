/*
 * DistanceReader.cpp
 *
 *  Created on: Feb 13, 2016
 *      Author: michel
 */
#include "DistanceReader.hpp"
#include <iostream>
/*
void stringParsing(std::string* s, std::string* delimiter){
	size_t pos = 0;
	std::string token;
	while ((pos = s->find(*delimiter)) != std::string::npos) {
	    token += s->substr(0, pos);
	    s->erase(0, pos + delimiter->length());
	}
}
*/
/*
 * Example of a matrix
 *
   14
Mouse
Bovine      1.7043
Lemur       2.0235  1.1901
Tarsier     2.1378  1.3287  1.2905
Squir Monk  1.5232  1.2423  1.3199  1.7878
 */

DistanceReader::DistanceReader(std::string fileName){ //include exception, not ready yet
	FILE* inFile;
	size_t size;
	this->distCalc = NULL;
	this->K = 0;
	if(&fileName == NULL or fileName.empty()){
		inFile = stdin;
		fseek(inFile,0,SEEK_END); //TODO: review that, fseek not supposed to work in stdin
		size = ftell(inFile);
		fseek(inFile,0,SEEK_SET);
		if (size==0) {
			fprintf(stderr,"Error reading from STDIN.\n");
			Exception::critical();
		}
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
/*    	if (this->fileSize != numRead){
    		printf("File size expected: %d file size read: %d\n",(int)this->fileSize,(int)numRead);
    		fprintf(stderr,"Reading error. Quitting");
    		Exception::critical();
    	}*/
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
			//std::cout<<*names[count]<<std::endl;
    		//get nums limit
    		while(x[numEnd] != '\n' && x[numEnd] != '\t' && count2 < this->K){
				numBegin = numEnd+1;
				while(x[numBegin] == ' ' || x[numBegin] == '\t' || x[numBegin] == '\n' || x[numBegin] == '\r')
					numBegin++;
				numEnd = numBegin+1;
				while(x[numEnd] != ' ' && x[numEnd] != '\t' && x[numEnd] != '\n' && x[numEnd] != '\r')
					numEnd++;
/*
 * 					for (int i=0; i<inD.length-1; i++)
						for (int j=0; j<inD.length-1-i; j++)
							distances[i][j] = (int)(inD[i][j+i+1] * 10000000);
 */
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
	//val = roundf(val*1000000)/1000000;
	if (isNeg)
		return 0 - val;
	else
		return val;
}
