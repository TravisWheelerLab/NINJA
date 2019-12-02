#ifndef ARGUMENTHANDLER_HPP
#define ARGUMENTHANDLER_HPP

#include "ExceptionHandler.hpp"
#include "TreeBuilder.hpp"
#include <stdio.h>
#include <stdlib.h>

class ArgumentHandler {
  public:
    enum InputType { alignment, distance };
    enum OutputType { dist, tree };
    enum AlphabetType { dna, amino, null };
    enum CorrectionType {
        not_assigned,
        none,
        JukesCantor /*DNA*/,
        Kimura2 /*DNA*/,
        FastTree /*amino*/
    };

    std::string method; // default should be "bin" for small, "cand" for large.
    std::string njTmpDir;
    std::string inFile; // works?
    InputType inType;
    OutputType outType;
    AlphabetType alphType;
    CorrectionType corrType;
    FILE *outFile;

    int threads;

    bool SSE;

    bool abort;

    ArgumentHandler(char *argv[], int argc);
    std::string getMethod();
    std::string getInFile();
    std::string getNJTmpDir();
    InputType getInputType();
    OutputType getOutputType();
    FILE *getOutpuFile();
    int getNumThreads();

    AlphabetType getAlphabetType();

    CorrectionType getCorrectionType();

    bool argumentTest();

    bool useSSE();
};

#endif
