#ifndef DISTANCECALCULATOR_HPP
#define DISTANCECALCULATOR_HPP

#include "DistanceCalculatorBase.hpp"
#include "ExceptionHandler.hpp"

#include <math.h>
#include <float.h>

#include <omp.h> /* openmp */

/**
 * SSE distance calculator implementation.
 */
class DistanceCalculator : DistanceCalculatorBase<__m128i>
{
public:
	DistanceCalculator(std::string **A /*alignment*/, AlphabetType alphType, CorrectionType corrType, int numberOfSequences, bool useSSE);
	~DistanceCalculator();

	// Inherited from DistanceCalculatorBase
	double calc(int a, int b);

	// Inherited from DistanceCalculatorBase
	double testDifferenceCluster(int a, int b);

protected:
	void countBitWidth(
		register __m128i &seq1,
		register __m128i &seq2,
		register __m128i &gap1,
		register __m128i &gap2,
		register __m128i &tmp,
		register __m128i &tmp2,
		register __m128i &tmp3,
		register __m128i &count_transversions,
		register __m128i &count_transitions,
		register __m128i &count_gaps);

private:
	int *inv_alph;

	int protein_dict[256];
	int protein_dict_original[256];
	int additionalGaps; //inverse of the number of gaps added at the end of the sequence in sse calculation

	inline void count128P(register __m128i &seq1, register __m128i &seq2, register __m128i &gap1, register __m128i &gap2, register __m128i &VALUES_0, register __m128i &VALUES_1, register __m128i &VALUES_2, register __m128i &VALUES_3, register __m128i &VALUES_4, register __m128i &VALUES_5, register __m128i &VALUES_6, register __m128i &VALUES_7, register __m128i &sum, register __m128i &gap_count, register __m128i &tmp1, register __m128i &tmp2, int a, int b);

	double newCalcDNA(int a, int b);

	double newCalcProtein(int a, int b);

	void convertAllDNA();
	void convertAllProtein();

	void getBitsDNA(char *seq, int *size, unsigned int *seqOut, unsigned int *gapOut);

	void generateProteinClusterDict(int *protein_dictionary);
	void getBitsProteinClustered(char *seq, int *size, unsigned int *seqOut, unsigned int *gapOut);
	void generateProteinOriginalDict(int *protein_dictionary);

	unsigned int *getProteinDic(std::string alph, int length);
};

#endif
