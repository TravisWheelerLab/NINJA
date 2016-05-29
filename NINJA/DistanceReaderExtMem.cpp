/*
 * DistanceReaderExtMem.cpp
 *
 *  Created on: May 2, 2016
 *      Author: michel
 */

#include "DistanceReaderExtMem.hpp"

//TODO: Implement this
DistanceReaderExtMem::DistanceReaderExtMem ( DistanceCalculator* distCalc, int K ){

	if (distCalc == NULL){
    	fprintf(stderr,"Null distance calculator handed to external matrix reader. Qutting.\n");
    	Exception::critical();
	}

	this->distCalc = distCalc;
	this->K = K;
}
int DistanceReaderExtMem::read (std::string** names, float* R, FILE* diskD, float** memD, int memDRowsize, int rowLength,  int pageBlockSize){
    int row = 0;


    //I want to write a hunk of the data to memD instead of disk.
    //Find a point where memD is partly full, then scale back a bit,
    //so what would otherwise be an incomplete block on disk isn't written
    // (note: numColsToDisk must be <= K)
    int numColsToDisk = 0;
    if (memDRowsize < 2*K-2) {
    	numColsToDisk = 2*K - 2 - (memDRowsize/2)  ;
	    int numBlocksToDisk = numColsToDisk/pageBlockSize;
	    numColsToDisk = numBlocksToDisk*pageBlockSize;
	    if (numColsToDisk > K) numColsToDisk = K;
    }


    int currCol = 0;

    int buffSize = numPages*pageBlockSize;


    int buffPtr=0;

	float d;
	long diskPos;

	float* fBuff = new float[numColsToDisk];
	int floatsInBuff = 0;

	//TODO: implement threads here, more complicated that in the in memory function, but still feasible
	if(this->distCalc != NULL){
    	while (row != K && buffPtr<buffSize) {

			for (int col=0; col<K; col++) {

				d = ((float)ceil((float)(10000000 * distCalc->calc(row, col))))/10000000;

				R[row] += d;

				if (currCol<numColsToDisk)  {

					fBuff[floatsInBuff++] = d;
			    	if (floatsInBuff == numColsToDisk ) {

				    	diskPos = (long)floatSize * rowLength * row ;
				    	fseek(diskD,diskPos,SEEK_SET);
				    	fwrite(fBuff,sizeof(float),numColsToDisk,diskD);

				    	floatsInBuff = 0;

			    	}
				} else {
					memD[row][currCol-numColsToDisk] = d;
				}


				currCol++;

			}

			row++;
	        currCol=0;
	    }
	}else{
    	fprintf(stderr,"Not allowed yet.\n");
    	Exception::critical();
	}
	return numColsToDisk;
}

float DistanceReaderExtMem::atoi (char* in, int end){
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

