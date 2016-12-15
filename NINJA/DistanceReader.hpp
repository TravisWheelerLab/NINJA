/*
 * DistanceReader.hpp
 *
 *  Created on: Feb 13, 2016
 *      Author: michel
 */
#include "ExceptionHandler.hpp"
#include "DistanceCalculator.hpp"

class DistanceReader{
	public:
		static const int numPages = 10;
		DistanceReader(std::string fileName);
		DistanceReader(DistanceCalculator* distCalc, int K, int threads);
		~DistanceReader();

		void read (std::string **names, int** distances);
		void write(FILE* outFile, int** distances);

		int threads;
		int K;
		FILE* r;
		size_t fileSize;
		DistanceCalculator *distCalc;
	private:
		float atoi(char* in, int end);
};


