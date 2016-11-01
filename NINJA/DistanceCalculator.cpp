/*
 * DistanceCalculator.cpp
 *
 *  Created on: Feb 13, 2016
 *      Author: michel
 */
#include "DistanceCalculator.hpp"
#include <iostream>

DistanceCalculator::DistanceCalculator (std::string** A /*alignment*/, AlphabetType alphType, CorrectionType corrType, int numberOfSequences) :
bl45{
			      {0, 1.31097856, 1.06573001, 1.26827829, 0.90471293, 1.05855446, 1.05232790, 0.76957444, 1.27579668, 0.96460409, 0.98717819, 1.05007594, 1.05464162, 1.19859874, 0.96740447, 0.70049019, 0.88006018, 1.09748548, 1.28141710, 0.80003850},
			      {1.31097856, 0, 0.80108902, 0.95334071, 1.36011107, 0.63154377, 0.79101490, 1.15694899, 0.76115257, 1.45014917, 1.17792001, 0.39466107, 0.99880755, 1.13514340, 1.15432562, 1.05309036, 1.05010474, 1.03938321, 0.96321690, 1.20274751},
			      {1.06573001, 0.80108902, 0, 0.48821721, 1.10567116, 0.81497020, 0.81017644, 0.74648741, 0.61876156, 1.17886558, 1.52003670, 0.80844267, 1.28890258, 1.16264109, 1.18228799, 0.67947568, 0.85365861, 1.68988558, 1.24297493, 1.55207513},
			      {1.26827829, 0.95334071, 0.48821721, 0, 1.31581050, 0.76977847, 0.48207762, 0.88836175, 0.73636084, 1.76756333, 1.43574761, 0.76361291, 1.53386612, 1.74323672, 0.88634740, 0.80861404, 1.01590147, 1.59617804, 1.17404948, 1.46600946},
			      {0.90471293, 1.36011107, 1.10567116, 1.31581050, 0, 1.38367893, 1.37553994, 1.26740695, 1.32361065, 1.26087264, 1.02417540, 1.37259631, 1.09416720, 0.98698208, 1.59321190, 0.91563878, 0.91304285, 1.80744143, 1.32944171, 0.83002214},
			      {1.05855446, 0.63154377, 0.81497020, 0.76977847, 1.38367893, 0, 0.50694279, 1.17699648, 0.61459544, 1.17092829, 1.19833088, 0.63734107, 0.80649084, 1.83315144, 0.93206447, 0.85032169, 1.06830084, 1.05739353, 0.97990742, 1.54162503},
			      {1.05232790, 0.79101490, 0.81017644, 0.48207762, 1.37553994, 0.50694279, 0, 1.17007322, 0.76978695, 1.46659942, 1.19128214, 0.63359215, 1.27269395, 1.44641491, 0.73542857, 0.84531998, 1.06201695, 1.32439599, 1.22734387, 1.53255698},
			      {0.76957444, 1.15694899, 0.74648741, 0.88836175, 1.26740695, 1.17699648, 1.17007322, 0, 1.12590070, 1.70254155, 1.38293205, 1.16756929, 1.17264582, 1.33271035, 1.07564768, 0.77886828, 1.23287107, 0.96853965, 1.42479529, 1.41208067},
			      {1.27579668, 0.76115257, 0.61876156, 0.73636084, 1.32361065, 0.61459544, 0.76978695, 1.12590070, 0, 1.41123246, 1.14630894, 0.96779528, 0.77147945, 1.10468029, 1.12334774, 1.02482926, 1.28754326, 1.27439749, 0.46868384, 1.47469999},
			      {0.96460409, 1.45014917, 1.17886558, 1.76756333, 1.26087264, 1.17092829, 1.46659942, 1.70254155, 1.41123246, 0, 0.43335051, 1.46346092, 0.46296554, 0.66291968, 1.07010201, 1.23000200, 0.97348545, 0.96354620, 0.70872476, 0.35120011},
			      {0.98717819, 1.17792001, 1.52003670, 1.43574761, 1.02417540, 1.19833088, 1.19128214, 1.38293205, 1.14630894, 0.43335051, 0, 1.49770950, 0.47380007, 0.53847312, 1.37979627, 1.58597231, 0.99626739, 0.98609554, 0.72531066, 0.57054219},
			      {1.05007594, 0.39466107, 0.80844267, 0.76361291, 1.37259631, 0.63734107, 0.63359215, 1.16756929, 0.96779528, 1.46346092, 1.49770950, 0, 1.00797618, 1.44331961, 0.92459908, 1.06275728, 1.05974425, 1.04892430, 0.97205882, 1.21378822},
			      {1.05464162, 0.99880755, 1.28890258, 1.53386612, 1.09416720, 0.80649084, 1.27269395, 1.17264582, 0.77147945, 0.46296554, 0.47380007, 1.00797618, 0, 0.72479754, 1.16998686, 1.34481214, 1.06435197, 1.05348497, 0.77487815, 0.60953285},
			      {1.19859874, 1.13514340, 1.16264109, 1.74323672, 0.98698208, 1.83315144, 1.44641491, 1.33271035, 1.10468029, 0.66291968, 0.53847312, 1.44331961, 0.72479754, 0, 1.32968844, 1.21307373, 0.96008757, 0.47514255, 0.34948536, 0.69273324},
			      {0.96740447, 1.15432562, 1.18228799, 0.88634740, 1.59321190, 0.93206447, 0.73542857, 1.07564768, 1.12334774, 1.07010201, 1.37979627, 0.92459908, 1.16998686, 1.32968844, 0, 0.97908742, 0.97631161, 1.21751652, 1.42156458, 1.40887880},
			      {0.70049019, 1.05309036, 0.67947568, 0.80861404, 0.91563878, 0.85032169, 0.84531998, 0.77886828, 1.02482926, 1.23000200, 1.58597231, 1.06275728, 1.34481214, 1.21307373, 0.97908742, 0, 0.56109848, 1.76318885, 1.29689226, 1.02015839},
			      {0.88006018, 1.05010474, 0.85365861, 1.01590147, 0.91304285, 1.06830084, 1.06201695, 1.23287107, 1.28754326, 0.97348545, 0.99626739, 1.05974425, 1.06435197, 0.96008757, 0.97631161, 0.56109848, 0, 1.39547634, 1.02642577, 0.80740466},
			      {1.09748548, 1.03938321, 1.68988558, 1.59617804, 1.80744143, 1.05739353, 1.32439599, 0.96853965, 1.27439749, 0.96354620, 0.98609554, 1.04892430, 1.05348497, 0.47514255, 1.21751652, 1.76318885, 1.39547634, 0, 0.32000293, 1.26858915},
			      {1.28141710, 0.96321690, 1.24297493, 1.17404948, 1.32944171, 0.97990742, 1.22734387, 1.42479529, 0.46868384, 0.70872476, 0.72531066, 0.97205882, 0.77487815, 0.34948536, 1.42156458, 1.29689226, 1.02642577, 0.32000293, 0, 0.93309543},
			      {0.80003850, 1.20274751, 1.55207513, 1.46600946, 0.83002214, 1.54162503, 1.53255698, 1.41208067, 1.47469999, 0.35120011, 0.57054219, 1.21378822, 0.60953285, 0.69273324, 1.40887880, 1.02015839, 0.80740466, 1.26858915, 0.93309543, 0}
				}
{
	this->A  = A;
	this->corr_type = corrType;
	this->alph_type = alphType;
	this->dna_chars = "AGCT";
	this->aa_chars = "ARNDCQEGHILKMFPSTWYV";
	this->numberOfSequences = numberOfSequences;
	this->lengthOfSequences = A[0]->size();

	if(this->newCalculation && this->alph_type == this->dna)
		convertAllDNA();
	else if(this->newCalculation && this->alph_type == this->amino)
		convertAllProtein();

	if (this->corr_type == not_assigned) {
		if (this->alph_type == amino) {
			this->corr_type = FastTree;
		} else {
			this->corr_type = Kimura2;
		}
	}

	this->inv_alph = DistanceCalculator::getInverseAlphabet( this->alph_type==dna ? this->dna_chars : this->aa_chars, this->alph_type==dna ? 4 : 20 );
}


int* DistanceCalculator::getInverseAlphabet (std::string alph, int length) {
	int* inv_alph = new int[256];
	for (int i=0; i<256; i++)
		inv_alph[i]=-1;
	for (int i=0; i<length; i++)
		inv_alph[(int)alph[i]] = i;
	return inv_alph;
}
inline void DistanceCalculator::count128(register __m128i &seq1, register __m128i &seq2, register __m128i &gap1, register __m128i &gap2, register __m128i &tmp, register __m128i &tmp2, register __m128i &tmp3, register __m128i &count_transversions, register __m128i &count_transitions, register __m128i &count_gaps){
	/*
	 * Maps and their description:
	 *
	 * GAPS_COUNT_MASK (count for gaps, basically how many 0`s there are)
	 * 	0000	4
	 * 	0001	3
	 * 	0010	3
	 * 	0011	2
	 * 	0100	3
	 * 	0101	2
	 * 	0110	2
	 * 	0111	1
	 * 	1000	3
	 * 	1001	2
	 * 	1010	2
	 * 	1011	1
	 * 	1100	2
	 * 	1101	1
	 * 	1110	1
	 * 	1111	0
	 *
	 * 	DECOMPRESSED_GAPS (transform 1-bit representation to 2-bits)
	 *
	 * 	0000	00 00 00 00
	 * 	0001	00 00 00 11
	 * 	0010	00 00 11 00
	 * 	0011	00 00 11 11
	 * 	0100	00 11 00 00
	 * 	0101	00 11 00 11
	 * 	0110	00 11 11 00
	 * 	0111	00 11 11 11
	 * 	1000	11 00 00 00
	 * 	1001	11 00 00 11
	 * 	1010	11 00 11 00
	 * 	1011	11 00 11 11
	 * 	1100	11 11 00 00
	 * 	1101	11 11 00 11
	 * 	1110	11 11 11 00
	 * 	1111	11 11 11 11
	 *
	 * 	COUNTS_MASK (clear upper 4 bits)
	 *
	 * 	0000 1111
	 *
	 * 	TRANSITIONS_MASK (count 10`s)
	 *
	 * 	0000	0
	 * 	0001	0
	 * 	0010	1
	 * 	0011	0
	 * 	0100	0
	 * 	0101	0
	 * 	0110	1
	 * 	0111	0
	 * 	1000	1
	 * 	1001	1
	 * 	1010	2
	 * 	1011	1
	 * 	1100	0
	 * 	1101	0
	 * 	1110	1
	 * 	1111	0
	 *
	 * 	TRANSVERSIONS_MASK (count 01`s and 11`s)
	 *
	 * 	0000	0
	 * 	0001	1
	 * 	0010	0
	 * 	0011	1
	 * 	0100	1
	 * 	0101	2
	 * 	0110	1
	 * 	0111	2
	 * 	1000	0
	 * 	1001	1
	 * 	1010	0
	 * 	1011	1
	 * 	1100	1
	 * 	1101	2
	 * 	1110	1
	 * 	1111	2
	 */

	tmp = _mm_xor_si128(seq1, seq2);
	tmp2  = _mm_and_si128(gap1,gap2);

	tmp3 = _mm_shuffle_epi8(GAPS_COUNT_MASK, tmp2);

	count_gaps = _mm_add_epi8(count_gaps, tmp3);

	tmp3 = _mm_shuffle_epi8(DECOMPRESSED_GAPS, tmp2);



	tmp  = _mm_and_si128(tmp,tmp3);


	tmp2  = _mm_and_si128(tmp,COUNTS_MASK);

	tmp3 = _mm_srli_epi64(tmp,4);

	tmp3 = _mm_and_si128(tmp3,COUNTS_MASK);


	tmp = _mm_shuffle_epi8(TRANSITIONS_MASK, tmp2);

	count_transitions = _mm_add_epi8(count_transitions, tmp);

	tmp = _mm_shuffle_epi8(TRANSITIONS_MASK, tmp3);

	count_transitions = _mm_add_epi8(count_transitions, tmp);



	tmp = _mm_shuffle_epi8(TRANSVERSIONS_MASK, tmp2);

	count_transversions = _mm_add_epi8(count_transversions, tmp);

	tmp = _mm_shuffle_epi8(TRANSVERSIONS_MASK, tmp3);

	count_transversions = _mm_add_epi8(count_transversions, tmp);


}
double DistanceCalculator::newCalcDNA(int a, int b){
	/*
	 * A = 00
	 * C = 01
	 * G = 10
	 * T = 11
	 *
	 *
	 * Explanation of what`s coded below:
	 *
	 *
	 *
	 * TMP = XOR (Seq1, Seq2)
	 *
	 * After the XOR we can see the following pattern
	 *
	 * Transition = 10
	 * Transversion = 11,01
	 *
	 * However, there might be some misleading counts because of gaps (currently encoded the same way as A, which is 00), which forces us to use
	 * a mask which accounts for place where there are gaps and places where there are not. To account for that we do the following:
	 *
	 * TMP2 = AND(Gap1, Gap2)
	 *
	 * We can now use a count process based on SHUFFLE to figure out how many gaps there are:
	 *
	 * TMP3 = SHUFFLE(GAPS_COUNT_MASK, TMP2)
	 *
	 * COUNTS_GAP = ADD(COUNTS_GAP, TMP3)
	 *
	 * Now we have added the 8-bit count bins to COUNTS_GAP. We can then proceed to calculate the transitions and transversions:
	 *
	 * TMP3 = SHUFFLE(DECOMPRESSED_GAPS, TPM2) //we use the 0000 - 1 bit representation per gap, and map it to a 2 bit representation of the gaps, with 11 as gap, and 00 as not-gap
	 *
	 * Now that we have the right mask we AND the mask and the result of the XOR:
	 *
	 * TMP = AND(TMP, TMP3)
	 *
	 * With the right vector in hands we will calculate transitions using the SHUFFLE count approach:
	 *
	 * TMP2 = AND(TMP, COUNTS_MASK) //the mask is 0000 1111 , and it basically removes the higher 4 bits
	 *
	 * TMP3 = SHIFT_RIGHT(TMP, 4)
	 *
	 * TMP3 = AND(TMP3, COUNTS_MASK) // same here, but then we have the higher 4 bits, originally, in the lower 4 bits now
	 *
	 * Finally get the transitions count:
	 *
	 * TMP = SHUFFLE(TRANSITIONS_MASK, TMP2) //get the counts
	 *
	 * TRANSITIONS_COUNT = ADD(TRANSITIONS_COUNT, TMP) //add to the transitions vector
	 *
	 * TMP = SHUFFLE(TRANSITIONS_MASK, TMP3) //get the counts
	 *
	 * TRANSITIONS_COUNT = ADD(TRANSITIONS_COUNT, TMP) //add to the transitions vector
	 *
	 *
	 * And then the same happens to transversions:
	 *
	 * TMP = SHUFFLE(TRANSVERSIONS_MASK, TMP2) //get the counts
	 *
	 * TRANSITIONS_COUNT = ADD(TRANSITIONS_COUNT, TMP) //add to the transitions vector
	 *
	 * TMP = SHUFFLE(TRANSVERSIONS_MASK, TMP3) //get the counts
	 *
	 * TRANSITIONS_COUNT = ADD(TRANSITIONS_COUNT, TMP) //add to the transitions vector
	 *
	 *
	 * And that`s it. We can calculate 64 characters in 17 operations. There is also a gather/extract part after all vectors which is 5 instructions
	 * for each count, which equals 15.
	 *
	 */


	register __m128i seq1;
	register __m128i seq2;
	register __m128i gap1;
	register __m128i gap2;
	register __m128i tmp;
	register __m128i tmp2;
	register __m128i tmp3;
	register __m128i counts_transversions;
	register __m128i counts_gaps;
	register __m128i counts_transitions;


	int numOfInts = ceil((float)this->lengthOfSequences/16.0);
	if(numOfInts % 4 != 0)
		numOfInts += 4 - (numOfInts % 4);

	int length = this->lengthOfSequences;

	const unsigned int* Achar = this->convertedSequences[a];
	const unsigned int* Bchar = this->convertedSequences[b];

	const unsigned int* Agap = this->gapInTheSequences[a];
	const unsigned int* Bgap = this->gapInTheSequences[b];

	int num_transversions = 0;

	int num_transitions = 0;

	int gaps = 0;

	int i = 0;

	while(i < numOfInts){
		counts_transversions = x128;
		counts_gaps= x128;
		counts_transitions = x128;


		//TODO: review number of max iterations
		for(int j = 0;i<numOfInts && j < 31;i += 4){ //a maximum of 32 vectors allowed not to overflow things

				seq1 = *(__m128i*)&Achar[i];
				seq2 = *(__m128i*)&Bchar[i];

				gap1 = *(__m128i*)&Agap[i];
				gap2 = *(__m128i*)&Bgap[i];

				count128(seq1,seq2,gap1, gap2, tmp,tmp2,tmp3,counts_transversions,counts_transitions, counts_gaps);


				j+=4;
		}

		/*gather transversion counts*/

		counts_transversions = _mm_xor_si128(counts_transversions, x128);

		counts_transversions = _mm_sad_epu8 (counts_transversions, zero);
		tmp = _mm_shuffle_epi32(counts_transversions, _MM_SHUFFLE(1, 1, 1, 2));
		counts_transversions = _mm_add_epi16(counts_transversions, tmp);

		num_transversions +=  _mm_extract_epi16(counts_transversions, 0);


		/*gather transition counts*/

		counts_transitions = _mm_xor_si128(counts_transitions, x128);

		counts_transitions = _mm_sad_epu8 (counts_transitions, zero);
		tmp = _mm_shuffle_epi32(counts_transitions, _MM_SHUFFLE(1, 1, 1, 2));
		counts_transitions = _mm_add_epi16(counts_transitions, tmp);

		num_transitions += _mm_extract_epi16(counts_transitions, 0);

		/*gather gaps counts*/

		counts_gaps = _mm_xor_si128(counts_gaps, x128);

		counts_gaps = _mm_sad_epu8 (counts_gaps, zero);
		tmp = _mm_shuffle_epi32(counts_gaps, _MM_SHUFFLE(1, 1, 1, 2));
		counts_gaps = _mm_add_epi16(counts_gaps, tmp);

		gaps += _mm_extract_epi16(counts_gaps, 0);
	}

	length -= gaps;


	float dist = 0.0f;
	float maxscore =  (this->corr_type == none ? 1.0f : 3.0f);

	if(length == 0){
		dist = maxscore;
	}else{
		float p_f = (float)((float)num_transitions / (float)length);
		float q_f = (float)((float)num_transversions / (float)length);

		if ( p_f+q_f == 0)
			dist = 0;
		else if (this->corr_type == JukesCantor)
			dist = (float)(-(0.75)*log((double)(1.0-(4.0/3.0)*(p_f+q_f))));
		else if (this->corr_type == Kimura2){
			dist = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));
		}else if (this->corr_type == none)
			dist = p_f + q_f;
	}

	double dist_d = (dist < maxscore ? dist : maxscore);

    return dist_d;

}
inline void DistanceCalculator::count128P(register __m128i &seq1, register __m128i &seq2, register __m128i &gap1, register __m128i &gap2, register __m128i &tmp, register __m128i &tmp2, register __m128i &tmp3, register __m128i &count_equal){
	//TODO: define MASK and right COUNT_MASK
	//TODO: review calculations


	tmp = _mm_xor_si128(seq1, seq2);
	tmp  = _mm_and_si128(tmp, gap1);
	tmp  = _mm_and_si128(tmp, gap2);
	tmp2 = _mm_srli_epi64(tmp, 4);

	//tmp = _mm_and_si128(tmp, MASK);
	//tmp2 = _mm_and_si128(tmp2, MASK);

	//tmp = _mm_shuffle_epi8(COUNTS_MASK, tmp);
	//tmp2 = _mm_shuffle_epi8(COUNTS_MASK, tmp2);

	count_equal = _mm_add_epi8(count_equal, tmp);
	count_equal = _mm_add_epi8(count_equal, tmp2);

}
double DistanceCalculator::newCalcProtein(int a, int b){

	register __m128i seq1;
	register __m128i seq2;
	register __m128i gap1;
	register __m128i gap2;
	register __m128i tmp;
	register __m128i tmp2;
	register __m128i tmp3;
	register __m128i counts_equal;
	register __m128i counts_gaps;


	int numOfInts = ceil((float)this->lengthOfSequences/8.0);
	if(numOfInts % 4 != 0)
		numOfInts += 4 - (numOfInts % 4);

	int length = this->lengthOfSequences;

	const unsigned int* Achar = this->convertedSequences[a];
	const unsigned int* Bchar = this->convertedSequences[b];

	const unsigned int* Agap = this->gapInTheSequences[a];
	const unsigned int* Bgap = this->gapInTheSequences[b];

	int equal = 0;

	int i = 0;

	//TODO: review number of max iterations
	while(i < numOfInts){ //a maximum of 32 vectors allowed not to overflow things
		counts_equal = x128;
		counts_gaps= x128;

		for(int j = 0;i<numOfInts && j < 31;i += 4){

				seq1 = *(__m128i*)&Achar[i];
				seq2 = *(__m128i*)&Bchar[i];

				gap1 = *(__m128i*)&Agap[i];
				gap2 = *(__m128i*)&Bgap[i];

				//count128P(seq1,seq2,gap1, gap2, tmp, tmp2, tmp3, counts_equal, counts_gaps);

				j+=4;
		}

		/*gather equal counts*/

		counts_equal = _mm_xor_si128(counts_equal, x128);

		counts_equal = _mm_sad_epu8 (counts_equal, zero);
		tmp = _mm_shuffle_epi32(counts_equal, _MM_SHUFFLE(1, 1, 1, 2));
		counts_equal = _mm_add_epi16(counts_equal, tmp);

		equal +=  _mm_extract_epi16(counts_equal, 0);
	}

    return (length-equal);
}

double DistanceCalculator::calc (int a, int b){
	if(this->newCalculation && this->alph_type == this->dna){
		return newCalcDNA(a,b);
	}else if(this->newCalculation && this->alph_type == this->amino){
		return newCalcProtein(a,b);
	}

	float dist = 0.0f;
	float maxscore =  (this->corr_type == none ? 1.0f : 3.0f);



	if (this->alph_type == amino) {
		if (this->corr_type != none && this->corr_type != FastTree){
			fprintf(stderr, "illegal choice of correction method; must be 'n' or 's'");
			Exception::critical();
		}

		int count = 0;
		int length = (int)this->A[0]->length();
		const char* Achar = this->A[a]->c_str();
		const char* Bchar = this->A[b]->c_str();

		for (int i=0; i<length; i++){
			if (this->inv_alph[(int)Achar[i]] >= 0 && this->inv_alph[(int)Bchar[i]] >= 0) { // both are characters in the core alphabet
				dist += this->bl45[ this->inv_alph[(int)Achar[i]]  ]  [ this->inv_alph[(int)Bchar[i]]  ] ;
				count++;
			}
		}

		if (count==0) {
			dist = maxscore;
		} else {
			dist /= count;
			if (this->corr_type == FastTree)
				dist = dist < 0.91 ? (float)(-1.3*log((double)(1.0 - dist))) : maxscore;
		}
	} else {
		if (this->corr_type == FastTree){
			fprintf(stderr, "illegal choice of correction method; must be 'n', 'j', or 'k'");
			Exception::critical();
		}


		int p=0; //transitions
		int q=0; //transversion
		int count = 0;
		int length = (int)this->A[0]->length();
		const char* Achar = this->A[a]->c_str();
		const char* Bchar = this->A[b]->c_str();
		for (int i=0; i<length; i++) {
			if (this->inv_alph[(int)Achar[i]] >= 0 && this->inv_alph[(int)Bchar[i]] >= 0) { // both are characters in the core alphabet
				count++;
				if (this->inv_alph[(int)Achar[i]] != this->inv_alph[(int)Bchar[i]]) {
					//count transitions (A-G, C-T) and transversions (others)
					if ( (this->inv_alph[(int)Achar[i]] <2 && this->inv_alph[(int)Bchar[i]] < 2) || // both A and G, and not equal
							(this->inv_alph[(int)Achar[i]] >1 && this->inv_alph[(int)Bchar[i]] > 1) ) // both C and T, not equal
						p++;
					else
						q++;
				}
			}
		}

		if (count == 0) {
				dist = maxscore;
		} else {
			float p_f = (float)p / count;
			float q_f = (float)q / count;

			if ( p_f+q_f == 0)
				dist = 0;
			else if (this->corr_type == JukesCantor)
				dist = (float)(-(0.75)*log((double)(1.0-(4.0/3.0)*(p_f+q_f))));
			else if (this->corr_type == Kimura2){
				dist = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));
			}else if (this->corr_type == none)
				dist = p_f + q_f;
		}

	}

	double dist_d = (dist < maxscore ? dist : maxscore);
	return dist_d;
}

DistanceCalculator::~DistanceCalculator(){
	delete[](this->inv_alph);
}
void DistanceCalculator::getBitsDNA(char* seq, int* size, unsigned int *seqOut, unsigned int *gapOut){
	/*
	 * transform the sequence of files into 2-bit-packed characters
	 *
	 * A = 00
	 * C = 01
	 * G = 10
	 * T = 11
	 *
	 * A is ignored in the loop, because it would add 0 to the integer.
	 * If the size left is less than 16, the other bits are all set to 0 by default,
	 * therefore it will not be counted as transitions or transversions. It will not
	 * affect the length of the sequence as well because we assume all sequences have
	 * the same length and this value is stored before the conversion to bits.
	 *
	 * The gaps (represented in ASCII as '-' ) are stored in gapOut, in the following manner:
	 *
	 * In each byte, the higher four bits are 0, and the lower 4 bits are either 1 or 0 if there is a gap or not, respectively.
	 *
	 */

	*seqOut = 0x0;
	*gapOut = 0xF0F0F0F; // initialize higher 4 bits as 0, and lower 4 bits as one

	if(size<=0){//TODO: perhaps remove later?
		return;
	}

	const static int numCharPerElement = 16;

	const static int whereToGo[] = {27,19,11,3}; //lookup table to figure where should one add the gap value

	const unsigned int powersOfTwo[] = {1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,
			65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432,
			67108864, 134217728, 268435456, 536870912, 1073741824, 2147483648};

	int i;

	for(i=0;i<*size && i<numCharPerElement;i++){ //goes until the sequence ends or 16 characters are converted
		if(seq[i] == 'C'){ //01
			*seqOut += powersOfTwo[((numCharPerElement-i)*2)-2];
		}else if(seq[i] == 'G'){ //10
			*seqOut += powersOfTwo[((numCharPerElement-i)*2)-1];
		}else if(seq[i] == 'T'){ //11
			*seqOut += (powersOfTwo[((numCharPerElement-i)*2)-2] + powersOfTwo[((numCharPerElement-i)*2)-1]);
		}else if(seq[i] == '-'){ //gap, 1 in one of the 4 lower bits of a byte
			*gapOut -= powersOfTwo[whereToGo[i/4]-(i%4)]; //get the right place to add and and the powers of 2 correspondent
		}
	}
	*size -= i;

}
void DistanceCalculator::generateProteinClusterDict(int* protein_dictionary){
	/*
	 * Clusters: {A} {R, N} {D} {C} {Q, E} {G} {H} {I, L, K} {M} {F} {S} {T} {W} {Y} {V}
	 */
	char proteins[20] = {'A', 'R', 'N', 'D', 'C', 'Q', 'E', 'G', 'H', 'I', 'L', 'K', 'M', 'F', 'S', 'T', 'W', 'Y', 'V'};
	protein_dictionary[(int)proteins[0]] = 0;
	protein_dictionary[(int)proteins[1]] = 1;
	protein_dictionary[(int)proteins[2]] = 1;
	protein_dictionary[(int)proteins[3]] = 2;
	protein_dictionary[(int)proteins[4]] = 3;
	protein_dictionary[(int)proteins[5]] = 4;
	protein_dictionary[(int)proteins[6]] = 4;
	protein_dictionary[(int)proteins[7]] = 5;
	protein_dictionary[(int)proteins[8]] = 6;
	protein_dictionary[(int)proteins[9]] = 7;
	protein_dictionary[(int)proteins[10]] = 8;
	protein_dictionary[(int)proteins[11]] = 8;
	protein_dictionary[(int)proteins[12]] = 8;
	protein_dictionary[(int)proteins[13]] = 9;
	protein_dictionary[(int)proteins[14]] = 10;
	protein_dictionary[(int)proteins[15]] = 11;
	protein_dictionary[(int)proteins[16]] = 12;
	protein_dictionary[(int)proteins[17]] = 13;
	protein_dictionary[(int)proteins[18]] = 14;
	protein_dictionary[(int)proteins[19]] = 15;
}
void DistanceCalculator::getBitsProteinClustered(char* seq, int* size, unsigned int *seqOut, unsigned int *gapOut){
	/*
	 * For now use a vector for the gaps, just to count them and make the whole calculation easier.
	 *
	 * I use inverse_alphabet array to set the values of each protein. See DistanceCalculator::getInverseAlphabet.
	 */

	*seqOut = 0x0;
	*gapOut = 0x0; // initialize higher 4 bits as 0, and lower 4 bits as one

	if(size<=0){//TODO: perhaps remove later?
		return;
	}

	const static int numCharPerElement = 8;

	const unsigned int gapValues[] = {15, 240, 3840, 61440, 983040, 15728640, 251658240, 4026531840};

	int i;

	for(i=0;i<*size && i<numCharPerElement;i++){
		if(seq[i] == '-'){
			//*seqOut += (0xFF << ((3-i)*8)); This byte will be all zeros
			*gapOut += gapValues[i]; //put up the gap in the right place
		}else{
			*seqOut += (this->protein_dict[(int)seq[i]]) << (i*4);
		}
	}
	*size -= i;
}
unsigned int* DistanceCalculator::getProteinDic (std::string alph, int length) {
	unsigned int* inv_alph = new unsigned int[256];
	for (int i=0; i<256; i++)
		inv_alph[i]= 0xFFFFFFFF;
	for (int i=0; i<length; i++)
		inv_alph[(int)alph.at(i)] = i;
	return inv_alph;
}

void DistanceCalculator::convertAllProtein(){
	/*
	 * TODO: at first use all 20 proteins, and represent them in 1 byte, although we only need 5 bits. We will also represent gaps with 1 byte in the same array, all 1`s,
	 *  but later on we can pack at least 3 gaps in one byte (higher 3 bits).
	 */

	this->x128 = _mm_set1_epi8((int8_t) -128);
	this->zero = _mm_set1_epi8((int8_t) 0x00);
	this->COUNTS_MASK = _mm_set1_epi8((int8_t) 0xF);

	generateProteinClusterDict(this->protein_dict);


	this->GAPS_COUNT_MASK = _mm_set_epi8(0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4);

	this->DECOMPRESSED_GAPS = _mm_set_epi8(255, 252, 243, 240, 207, 204, 195, 192, 63, 60, 51, 48, 15, 12, 3, 0);


	//TODO: make sure all of these are aligned, so I can load them faster
	this->convertedSequences = new unsigned int*[this->numberOfSequences];
	this->gapInTheSequences = new unsigned int*[this->numberOfSequences];

	int allocSize = ceil((float)this->lengthOfSequences/8.0);
	if(allocSize % 4 != 0) //min size of 128bits
		allocSize += 4 - (allocSize % 4);
	int sizeLeft;
	for(int i=0;i<this->numberOfSequences;i++){
		this->convertedSequences[i] = new unsigned int[allocSize]; //min of 128bits, no need to change
		sizeLeft = this->lengthOfSequences;
		for(int j=0;j<allocSize;j++){
			getBitsProteinClustered((char*)(this->A[i]->c_str()+(this->lengthOfSequences-sizeLeft)),&sizeLeft, &(this->convertedSequences[i][j]), &(this->gapInTheSequences[i][j]));
		}

	}
	//TODO: free all the unnecessary memory

}
void DistanceCalculator::convertAllDNA(){

	this->x128 = _mm_set1_epi8((int8_t) -128);
	this->zero = _mm_set1_epi8((int8_t) 0x00);
	this->COUNTS_MASK = _mm_set1_epi8((int8_t) 0xF);

	/* GAPS MASK:
	 *
	 *  0000	4
	 * 	0001	3
	 * 	0010	3
	 * 	0011	2
	 * 	0100	3
	 * 	0101	2
	 * 	0110	2
	 * 	0111	1
	 * 	1000	3
	 * 	1001	2
	 * 	1010	2
	 * 	1011	1
	 * 	1100	2
	 * 	1101	1
	 * 	1110	1
	 * 	1111	0
	 */


	this->GAPS_COUNT_MASK = _mm_set_epi8(0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4);

	this->DECOMPRESSED_GAPS = _mm_set_epi8(255, 252, 243, 240, 207, 204, 195, 192, 63, 60, 51, 48, 15, 12, 3, 0);

	this->TRANSITIONS_MASK = _mm_set_epi8(0, 1, 0, 0, 1, 2, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0);

	this->TRANSVERSIONS_MASK = _mm_set_epi8(2, 1, 2, 1, 1, 0, 1, 0, 2, 1, 2, 1, 1, 0, 1, 0);


	//TODO: make sure all of these are aligned, so I can load them faster
	this->convertedSequences = new unsigned int*[this->numberOfSequences];
	this->gapInTheSequences = new unsigned int*[this->numberOfSequences];

	int allocSize = ceil((float)this->lengthOfSequences/16.0);
	if(allocSize % 4 != 0) //min size of 128bits
		allocSize += 4 - (allocSize % 4);
	int sizeLeft;
	for(int i=0;i<this->numberOfSequences;i++){
		this->convertedSequences[i] = new unsigned int[allocSize]; //min of 128bits, TODO: change later
		this->gapInTheSequences[i] = new unsigned int[allocSize]; //min of 128bits, TODO: change later

		sizeLeft = this->lengthOfSequences;
		for(int j=0;j<allocSize;j++){
			getBitsDNA((char*)(this->A[i]->c_str()+(this->lengthOfSequences-sizeLeft)),&sizeLeft, &(this->convertedSequences[i][j]), &(this->gapInTheSequences[i][j]));
		}

	}
	//TODO: free all the unnecessary memory
/*	for(int i=0;i<this->numberOfSequences;i++)
		delete(this->A[i]);
	delete[](this->A);*/
}
