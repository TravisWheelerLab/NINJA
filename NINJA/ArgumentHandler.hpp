#ifndef ARGUMENTHANDLER_HPP
#define ARGUMENTHANDLER_HPP

#include <cstdio>
#include <cstdlib>

#include "ExceptionHandler.hpp"
#include "TreeBuilder.hpp"

class ArgumentHandler {
public:
    /// The input type can be either a set of alignments in FASTA format,
    /// or a pre-computed distance matrix in Phylip format.
    enum InputType {
        alignment, distance
    };

    /// The output type can be either a distance matrix or a tree.
    enum OutputType {
        dist, tree
    };

    /// The alphabet to use; either the DNA nucleotides or amino acids.
    enum AlphabetType {
        dna, amino, unknown
    };

    /// The correction algorithm to employ. The JukesCantor and Kimura2 methods
    /// are appropriate for DNA while FastTree is for amino acid alphabets.
    enum CorrectionType {
        not_assigned, none, JukesCantor, Kimura2, FastTree
    };

    ArgumentHandler(char *argv[], int argc);

    bool getAbort() const;

    std::string getMethod() const;

    std::string getInputFile() const;

    /// Get the temporary directory used for the neighbor-joining algorithm.
    std::string getNJTmpDir() const;

    InputType getInputType() const;

    OutputType getOutputType() const;

    FILE *getOutputFile() const;

    int getNumThreads() const;

    AlphabetType getAlphabetType() const;

    CorrectionType getCorrectionType() const;

    static bool argumentTest();

    bool getUseSSE() const;

private:
    // TODO: use an enum if there are only a few values
    std::string method; // default should be "bin" for small, "cand" for large.
    std::string njTmpDir;
    std::string inputFile; //works?
    InputType inputType;
    OutputType outputType;
    AlphabetType alphabetType;
    CorrectionType correctionType;
    FILE *outputFile;
    int numThreads;
    bool useSSE;
    bool abort;
};

#endif
