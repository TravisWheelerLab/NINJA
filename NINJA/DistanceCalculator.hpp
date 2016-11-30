/*
 * DistanceCalculator.hpp
 *
 *  Created on: Feb 13, 2016
 *      Author: michel
 */

#ifndef DISTANCECALCULATOR_HPP
#define DISTANCECALCULATOR_HPP

#include "ExceptionHandler.hpp"
#include <math.h>

#include <xmmintrin.h>		/* SSE  */
#include <emmintrin.h>		/* SSE2 */
#include <tmmintrin.h>      /* SSE3 */
#include <stdint.h>

#include <omp.h> /* openmp */


class DistanceCalculator{
		const float bl45[20][20];
	public:


		enum CorrectionType {not_assigned, none, JukesCantor/*DNA*/, Kimura2/*DNA*/, FastTree /*amino*/};
		enum AlphabetType {dna, amino, null};

		DistanceCalculator (std::string** A /*alignment*/, AlphabetType alphType, CorrectionType corrType, int numberOfSequences);
		~DistanceCalculator();

		AlphabetType   alph_type;
		CorrectionType corr_type;

		std::string dna_chars;
		std::string aa_chars;
		std::string** A;

		int numberOfSequences;
		int lengthOfSequences;

		//TODO: set this as true only if it has support for SSE3
		const bool newCalculation = true;


		double calc (int a, int b);
		int* getInverseAlphabet (std::string alph, int length);



		unsigned int** convertedSequences;
		unsigned int** gapInTheSequences;


		__m128i x128;
		__m128i zero;
		__m128i GAPS_COUNT_MASK;

		__m128i DECOMPRESSED_GAPS;
		__m128i COUNTS_MASK;
		__m128i TRANSITIONS_MASK;
		__m128i TRANSVERSIONS_MASK;

	private:
		int *inv_alph;

		int protein_dict[128];

		inline void count128(register __m128i &seq1, register __m128i &seq2, register __m128i &gap1, register __m128i &gap2, register __m128i &tmp, register __m128i &tmp2, register __m128i &tmp3, register __m128i &count_transversions, register __m128i &count_transitions, register __m128i &count_gaps);
		inline void count128P(register __m128i &seq1, register __m128i &seq2, register __m128i &gap1, register __m128i &gap2, register __m128i &tmp, register __m128i &tmp2, register __m128i &tmp3, register __m128i &count_equal);


		double newCalcDNA(int a, int b);

		double newCalcProtein(int a, int b);

		void convertAllDNA();
		void convertAllProtein();

		void getBitsDNA(char* seq, int* size, unsigned int *seqOut, unsigned int *gapOut);

		void generateProteinClusterDict(int* protein_dictionary);
		void getBitsProteinClustered(char* seq, int* size, unsigned int *seqOut, unsigned int *gapOut);

		unsigned int* getProteinDic(std::string alph, int length);

};

#endif

