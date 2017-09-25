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
    } else {

	fscanf(this->r, "%d\n", &this->K);

	std::string lineString = "";

	int pos, newPos;

	size_t lineSize;

	char* line = new char[this->K*10 + 100];

	for(int i=0; i< this->K; i++){
		getline(&line, &lineSize, this->r);
//		printf(line);
		lineString = line;
		pos = lineString.find_first_of(" ");
		*names[i] = lineString.substr(0,pos);
//		printf(names[i]->c_str());
//		printf("-----------");
		pos++;
		newPos = lineString.find_first_of(" ", pos); //first space after the first number
		int length = newPos - pos; //length of the number
		for (int j=i+1; j<this->K; j++){
			distances[i][j-i-1] = 100 * (int)((((100000000*std::atof(lineString.substr(pos, length).c_str())))+50)/100);
//			printf(" %d",distances[i][j]);
			pos += length + 1;
		}
//		printf("\n");
	}

    	fclose(this->r);

    }
}
void DistanceReader::readAndWrite(std::string **names, FILE* outFile){ //possibly wrong, the else part, perhaps the distance calculator class as well

    unsigned int begin = 0, end = 0, numBegin = 0, numEnd = 0;
    int count = 0;
    double** distances = new double*[this->K];
    for(int i = 0; i < this->K; ++i)
    	distances[i] = new double[this->K];

    if (this->threads == 0){
        omp_set_num_threads(omp_get_max_threads());
    } else {
        omp_set_num_threads(this->threads);
    }
    	#pragma omp parallel for
    	for (int i=0; i<this->K; i++){
    		for (int j=i+1; j<this->K; j++){
    			//distances[i][j-i-1] = this->distCalc->testDifferenceCluster(i,j);
    			distances[i][j-i-1] = this->distCalc->calc(i,j) ; // this gets the same rounding I have in the distance writer code
    		}
	}
    this->write(outFile, distances, names);    
}

void DistanceReader::write(FILE* outFile,double** distances,std::string** names){
	fprintf(outFile, "%d\n", this->K); //number of sequences
	for (int i=0; i<this->K; i++){
		fprintf(outFile, "%s", names[i]->c_str());
		for (int j=0; j<i; j++){
			fprintf(outFile," %.6lf",distances[i][j]);
		}
		fprintf(outFile," 0.000000");
		for (int j=i+1; j<this->K; j++){
			fprintf(outFile," %.6lf",distances[j][i]);
		}
		fprintf(outFile,"\n");
	}
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
		} else if ((in[pos] != ' ') && (in[pos]<'0' || in[pos]>'9')) {
			fprintf(stderr,"Unable to convert integer from invalid char array. Position + %d, char = '%c', numberString = '%s'",pos,in[pos], in);
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
