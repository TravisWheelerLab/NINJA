/*
 * ClusterManager.cpp
 *
 *  Created on: Jul 12, 2019
 *      Author: robert
 */

#include "ClusterManager.hpp"

#define LINUX 1

#ifdef LINUX
#include <time.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#endif

//standard constructor
ClusterManager::ClusterManager(std::string method, std::string njTmpDir, std::string inFile, FILE* outFile,
InputType inType, OutputType outType, AlphabetType alphType, CorrectionType corrType, int threads, bool useSSE, bool
printTime, float clusterCutoff ){
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
	this->threads = threads;
        this->clusterCutoff = clusterCutoff;
	this->newDistanceMethod = useSSE;
	this->printTime = printTime;
}

/*
int64_t timespecDiff(struct timespec *timeA_p, struct timespec *timeB_p){
return ((timeA_p->tv_sec * 1000000000) + timeA_p->tv_nsec) -
        ((timeB_p->tv_sec * 1000000000) + timeB_p->tv_nsec);
}
*/

std::string ClusterManager::doJob(){
	double** distances = NULL;
	float** memD = NULL;
	float* R  = NULL;
	// all for external memory version
	int pageBlockSize = 1024; //that many ints = 4096 bytes;
	FILE* diskD = NULL;

	int rowLength = 0;
	int firstMemCol = -1;

	int numCols = 0;

	//Runtime runtime = Runtime.getRuntime();
	long maxMemory = -1;

	bool ok = true;
	std::string clusterString = "";

	//NinjaLogWriter.printVersion();

	int K=0;

	int* equalCluster;

    struct timespec start, afterRead, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

	SequenceFileReader* seqReader = NULL;
	if (!this->method.compare("extmem")){
		fprintf(stderr,"External memory clustering is not implemented.\n");
		Exception::critical();
	}else{
		DistanceReader* reader = NULL;

		if (this->inType == alignment) {
			seqReader = new SequenceFileReader(&(this->inFile),(SequenceFileReader::AlphabetType) this->alphType);
			std::string** seqs = seqReader->getSeqs();
			this->names = seqReader->getNames();

            clock_gettime(CLOCK_MONOTONIC, &afterRead); 

			this->alphType = (ClusterManager::AlphabetType) seqReader->getAlphType();
			fprintf(stderr,"Calculating distances....\n");
			DistanceCalculator* distCalc = new DistanceCalculator(seqs,(DistanceCalculator::AlphabetType) alphType,(DistanceCalculator::CorrectionType)  corrType, seqReader->numSeqs,newDistanceMethod);
			K = seqReader->numSeqs;
			reader = new DistanceReader(distCalc, K, this->threads);
		}else{
			reader = new DistanceReader(this->inFile);
			K = reader->K;
			this->names = new std::string*[K];
			for (int i = 0;i<K;i++)
				this->names[i] = new std::string();
		}

		distances = new double*[K];
		for (int i=0; i<K; i++) {
			distances[i] = new double[K - i - 1];
		}

		reader->readDoubles( this->names, distances);

                //
                // Nearest neighbor clustering
                //
                //   - Initialize starting cluster with sequence 0
                printf("Calculating clusters....\n");
                typedef std::vector<int> ClusterMembers;
                std::vector<ClusterMembers> clusters{ { 0 } };
                double dist;
                int cluster;
                int cseq;
                double scaledThresh = (double)clusterCutoff;
                //printf("scaledThresh = %.6lf\n", scaledThresh);
             
                for( int i = 1; i < K; i++ ) {
                  cluster = -1;
                  //printf("Where does %d go?\n", i);
                  for( int j = 0; j < clusters.size(); j++ ) {
                    for( int k = 0; k < clusters[j].size(); k++ ) {
                      cseq = clusters[j][k];
                      //printf("cseq = %d\n", cseq);
                      if ( i < cseq ){
                        //printf("Considering: seq %s vs cluster %d, seq %s distances[%d][%d]", this->names[i]->c_str(), j, this->names[cseq]->c_str(), i, cseq-i-1);
                        dist = distances[i][cseq-i-1];
                      }else {
                        //printf("Considering: seq %s vs cluster %d, seq %s distances[%d][%d]", this->names[i]->c_str(), j, this->names[cseq]->c_str(), cseq, i-cseq-1);
                        dist = distances[cseq][i-cseq-1];
                      }
                      //printf("=%.6lf\n", dist);
                      if ( dist < scaledThresh){
                        cluster = j;
                        break;
                      }
                    }
                    if ( cluster >= 0 )
                      break;
                  }
                  if ( cluster >= 0 ){
                    clusters[cluster].push_back(i);
                    //printf(" - adding to existing cluster %d\n", cluster);
                  }else {
                    clusters.push_back( std::vector<int>{ i } );
                    //printf(" - creating new cluster for this sequence\n");
                  }
                }
                if ( clusters.size() == 1 ) 
                  printf("There is 1 cluster\n");
                else
                  printf("There are %d clusters\n", clusters.size());
                for( int j = 0; j < clusters.size(); j++ ) {
                  for( int k = 0; k < clusters[j].size(); k++ ) {
                    fprintf(outFile,"%d\t%s\n", j, this->names[clusters[j][k]]->c_str());
                  }
                }
	}


	if (seqReader != NULL)
		delete seqReader;

	delete[] distances;
	return ( clusterString );
}
